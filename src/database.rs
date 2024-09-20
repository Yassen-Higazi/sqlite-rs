use crate::header::DBHeader;
use crate::page::Page;

use crate::schema::{SchemaTable, SchemaTypesTypes};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Read;
use std::os::unix::prelude::FileExt;

pub struct Database {
    header: DBHeader,
    root_page: Page,
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


        let root_page = Page::new(&page_buffer, page_size)?;

        // println!("Root Page: {root_page:?}");

        let db = Self {
            header: root_page.header.clone(),
            db_file_name: db_file_name.clone(),
            root_page,
        };

        Ok(db)
    }

    pub fn get_table_names(&self) -> Vec<String> {
        let mut tables = Vec::with_capacity(self.root_page.cells.len());

        for cell in &self.root_page.cells {
            let schema = SchemaTable::from(&cell.record);

            if schema.schema_type == SchemaTypesTypes::Table { tables.push(schema.tbl_name); }
        }

        tables.to_owned()
    }

    pub fn execute_command(&self, command: &String) -> Result<()> {
        match command.as_str() {
            ".dbinfo" => {
                println!("{}", self.header, );
                println!("number of tables:    {}", self.get_table_names().len());
            }

            ".tables" => {
                let tables = self.get_table_names();

                for t in tables {
                    print!("{t} ");
                }
            }

            _ => bail!("Missing or invalid command passed: {}", command),
        }

        Ok(())
    }
}