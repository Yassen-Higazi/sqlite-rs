use crate::parser::scanner::Scanner;

use crate::core::header::DBHeader;
use crate::core::page::Page;
use crate::core::schema::{SchemaTable, SchemaTypesTypes};

use crate::parser::statement::{Statement, StatementType};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Read;
use std::os::unix::prelude::FileExt;

pub struct Database {
    file: File,
    root_page: Page,
    header: DBHeader,
    scanner: Scanner,
    db_file_name: String,
}

impl Database {
    pub fn new(db_file_name: &String) -> Result<Self> {
        let mut file = File::open(&db_file_name)?;

        let mut size_buf = vec![0u8; 2];

        file.read_at(&mut size_buf, 16).with_context(|| "Could not read page size")?;

        let page_size = u16::from_le_bytes([size_buf[0], size_buf[1]]) * 256;

        let mut page_buffer = vec![0u8; page_size as usize];

        file.read_exact(&mut page_buffer)?;


        let root_page = Page::new(&page_buffer, page_size, 1)?;

        // println!("Root Page: {root_page:?}");

        let db = Self {
            file,
            scanner: Scanner::new(),
            header: root_page.header.clone(),
            db_file_name: db_file_name.clone(),
            root_page,
        };

        Ok(db)
    }

    pub fn get_table_schemas(&self) -> Vec<SchemaTable> {
        let mut tables = Vec::with_capacity(self.root_page.cells.len());

        for cell in &self.root_page.cells {
            let schema = SchemaTable::from(&cell.record);

            if schema.schema_type == SchemaTypesTypes::Table { tables.push(schema); }
        }

        tables
    }

    pub fn get_table_schema(&self, table_name: &String) -> Option<SchemaTable> {
        let mut table: Option<SchemaTable> = None;

        for cell in &self.root_page.cells {
            let schema = SchemaTable::from(&cell.record);

            if schema.schema_type == SchemaTypesTypes::Table && &schema.tbl_name == table_name { table = Some(schema) }
        }

        table
    }

    pub fn count_records(&self, table_name: &String) -> Result<u16> {
        let table = self.get_table_schema(table_name).with_context(|| format!("No such table: {table_name}"))?;

        let mut page_buff = vec![0u8; self.header.page_size as usize];

        let page_offset = ((table.root_page - 1) * self.header.page_size as i32) as u64;

        self.file.read_exact_at(&mut page_buff, page_offset)?;

        let table_root_page = Page::new(&page_buff, self.header.page_size, table.root_page as u64)?;

        Ok(table_root_page.num_of_cells)
    }

    pub fn execute_command(&self, command: &String) -> Result<()> {
        let mut scanner = Scanner::from(command.clone());

        scanner.scan(command)?;

        let tokens = scanner.get_tokens();

        let statement = Statement::new(tokens)?;

        match statement.statement_type {
            StatementType::SELECT => {
                println!("Table: {:?}", statement.tables);
                println!("Columns: {:?}", statement.columns);
                println!("Where: {:?}", statement.where_conditions);
                println!("Limit: {:?}", statement.limit);
                let table_name = statement.tables.first().unwrap().lexeme.clone();

                let schema_table = self.get_table_schema(&table_name).with_context(|| format!("relation {table_name} does not exist"))?;

                println!("Schema: {schema_table:?}");
            }

            _ => {
                match command.as_str() {
                    ".dbinfo" => {
                        println!("{}", self.header, );
                        println!("number of tables:    {}", self.get_table_schemas().len());
                    }

                    ".tables" => {
                        let tables = self.get_table_schemas();

                        for t in tables {
                            print!("{} ", t.tbl_name);
                        }
                    }

                    ".count" => {
                        let tables = self.get_table_schemas();

                        for table in tables {
                            let count = self.count_records(&table.tbl_name)?;

                            println!("{}: {}", table.tbl_name, count);
                        }
                    }

                    sql_text => {
                        let split = sql_text.split_whitespace().collect::<Vec<&str>>();

                        if split.len() == 4 {
                            let table_name = split.last().unwrap();

                            println!("{}", self.count_records(&table_name.to_string())?)
                        } else {}
                        bail!("Missing or invalid command passed: {}", command)
                    }
                }
            }
        };


        Ok(())
    }
}