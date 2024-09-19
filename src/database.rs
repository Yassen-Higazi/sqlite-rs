use crate::header::DBHeader;
use crate::page::Page;
use anyhow::{bail, Result};
use std::fs::File;
use std::io::Read;

pub struct Database {
    header: DBHeader,
    root_page: Page,
    db_file_name: String,
}

impl Database {
    pub fn new(db_file_name: &String) -> Result<Self> {
        let mut file = File::open(&db_file_name)?;

        let mut page_buffer = vec![0u8; 6000];

        file.read_exact(&mut page_buffer)?;

        let page_size = u16::from_le_bytes([page_buffer[16], page_buffer[17]]) * 256;

        let root_page = Page::new(&page_buffer, page_size)?;

        let db = Self {
            header: root_page.header.clone(),
            db_file_name: db_file_name.clone(),
            root_page,
        };

        Ok(db)
    }

    pub fn execute_command(&self, command: &String) -> Result<()> {
        match command.as_str() {
            ".dbinfo" => {
                println!("{}\nnumber of tables:    {}", self.header, self.root_page.num_of_cells);
            }
            _ => bail!("Missing or invalid command passed: {}", command),
        }

        Ok(())
    }
}