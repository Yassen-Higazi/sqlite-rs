use crate::core::header::DBHeader;
use crate::core::page::Page;
use crate::core::schema::{SchemaTable, SchemaTypesTypes};
use crate::parser::scanner::Scanner;
use std::collections::HashMap;

use crate::core::cell::ColumnTypes;
use crate::parser::statement::{Statement, StatementType};
use crate::parser::token::TokenType;
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Read;
use std::os::unix::prelude::FileExt;

pub type Row = HashMap<String, (ColumnTypes, Vec<u8>)>;

pub struct Database<'file> {
    file: &'file File,
    root_page: Page<'file>,
    header: DBHeader,
    scanner: Scanner,
}

impl<'file> Database<'file> {
    pub fn new(file: &'file mut File) -> Result<Database<'file>> {
        let mut size_buf = vec![0u8; 2];

        file.read_at(&mut size_buf, 16)
            .with_context(|| "Could not read page size")?;

        let page_size = u16::from_le_bytes([size_buf[0], size_buf[1]]) * 256;

        let mut page_buffer = vec![0u8; page_size as usize];

        file.read_exact(&mut page_buffer)?;

        let root_page = Page::new(file, page_size, 1)?;

        let db = Self {
            file,
            scanner: Scanner::new(),
            header: root_page.header.clone(),
            root_page,
        };

        Ok(db)
    }

    pub fn get_table_schemas(&self) -> Result<Vec<SchemaTable>> {
        let mut tables = Vec::with_capacity(self.root_page.cells.len());

        for (_, payload) in &self.root_page.get_payloads()? {
            let schema = SchemaTable::from(payload);

            if schema.schema_type == SchemaTypesTypes::Table {
                tables.push(schema);
            }
        }

        Ok(tables)
    }

    pub fn get_table_schema(&self, table_name: &String) -> Result<Option<SchemaTable>> {
        let mut table: Option<SchemaTable> = None;

        for (_, payload) in &self.root_page.get_payloads()? {
            let schema = SchemaTable::from(payload);

            if schema.schema_type == SchemaTypesTypes::Table && &schema.tbl_name == table_name {
                table = Some(schema);
                break;
            }
        }

        Ok(table)
    }

    fn read_page(&self, page_number: i32) -> Result<Page> {
        let table_root_page = Page::new(&self.file, self.header.page_size, page_number as u64)?;

        Ok(table_root_page)
    }

    pub fn count_records(&self, table_name: &String) -> Result<u16> {
        let table = self
            .get_table_schema(table_name)?
            .with_context(|| format!("No such table: {table_name}"))?;

        let page = self.read_page(table.root_page).with_context(|| {
            format!(
                "Could not read Page number {}, for table: {}",
                table.root_page, table.tbl_name
            )
        })?;

        Ok(page.num_of_cells)
    }

    pub fn get_data(&self, schema: &SchemaTable) -> Result<Vec<Row>> {
        let page = self.read_page(schema.root_page)?;

        let create_statement = &schema.statement;

        let mut column_vec = Vec::with_capacity(page.cells.len());

        let payloads = page.get_payloads()?;

        // println!("Rows len: {}", payloads.len());

        for (row_id, payload) in payloads {
            let mut index = 0;

            let mut meta = Row::new();

            if payload.column_types.len() == 0 {
                return Ok(column_vec);
            }

            // println!("RowId: {row_id}");

            for j in 0..create_statement.columns.len() {
                let column_name = &create_statement.columns[j];

                let cell_type = &payload.column_types[j];

                // if row_id == 1 {
                //     // println!("Column: Type -> {cell_type:?}, Name -> {}, Payload -> {payload:?}", column_name.lexeme);
                // }

                let len = cell_type.get_len() as usize;

                let data = &payload.body[index..index + len];

                if column_name.lexeme == "id" {
                    let bytes = row_id.to_be_bytes().to_vec();
                    let data = (ColumnTypes::new(bytes.len() as u64)?, bytes);

                    meta.insert(column_name.lexeme.to_string(), data);
                } else {
                    meta.insert(
                        column_name.lexeme.to_string(),
                        (cell_type.clone(), data.to_vec()),
                    );
                }


                index += len;
            }
            // println!("Data: {meta:?}");

            column_vec.push(meta);
        }

        Ok(column_vec)
    }

    pub fn execute_command(&self, command: &String) -> Result<()> {
        let mut scanner = Scanner::from(command.clone());

        scanner.scan(command)?;

        let tokens = scanner.get_tokens();

        let statement = Statement::new(tokens)?;

        match statement.statement_type {
            StatementType::SELECT => self.handle_select(&statement),

            _ => {
                match command.as_str() {
                    ".dbinfo" => {
                        println!("{}", self.header, );
                        println!("number of tables:    {}", self.get_table_schemas()?.len());
                    }

                    ".tables" => {
                        let tables = self.get_table_schemas()?;

                        for t in tables {
                            print!("{} ", t.tbl_name);
                        }
                    }

                    _ => bail!("Missing or invalid command passed: {}", command)
                };

                Ok(())
            }
        }
    }

    fn handle_select(&self, statement: &Statement) -> Result<()> {
        let table = statement.tables.first().with_context(|| "No table selected")?;

        let table_name = &table.lexeme;

        let is_count = statement
            .columns
            .iter()
            .any(|token| token.token_type == TokenType::COUNT);

        if is_count {
            println!("{}", self.count_records(&table_name)?);

            return Ok(());
        }

        let schema = self
            .get_table_schema(table_name)?
            .with_context(|| format!("Could not Get Table Schema: {}", table_name))?;

        let rows = self.get_data(&schema)?;

        if rows.len() == 0 {
            return Ok(());
        }

        let mut selected_columns = &statement.columns;

        for col_index in 0..statement.columns.len() {
            let col = &statement.columns[col_index];

            if col.token_type == TokenType::STAR {
                selected_columns = &schema.statement.columns;
                break;
            } else if rows[0].get(&col.lexeme).is_none() {
                bail!("No such Column: {}", col.lexeme)
            }
        }

        let mut limit: usize = 0;

        if let Some(lim) = statement.limit {
            limit = std::cmp::min(lim as usize, rows.len());
        } else {
            limit = rows.len();
        }

        for row_index in 0..limit {
            let row = &rows[row_index];

            if !statement.evaluate_where(&row)? {
                for col_index in 0..selected_columns.len() {
                    let col = &selected_columns[col_index];

                    match row.get(&col.lexeme) {
                        None => continue,

                        Some((col_type, data)) => {
                            col_type.print(data).with_context(|| {
                                format!("Could not print column Type: {col_type:?}")
                            })?;
                        }
                    }

                    if col_index != selected_columns.len() - 1 {
                        print!("|")
                    }
                }

                print!("\n");
            }
        }

        Ok(())
    }
}
