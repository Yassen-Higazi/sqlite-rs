use crate::header::DBHeader;
use anyhow::{bail, Result};
use std::fs::File;
use std::io::Read;

pub struct Database {
    header: DBHeader,
    db_file_name: String,
}

impl Database {
    pub fn new(db_file_name: &String) -> Result<Self> {
        let mut file = File::open(&db_file_name)?;

        let mut header = [0; 100];

        file.read_exact(&mut header)?;

        let db_headers = DBHeader::new(&header);

        let db = Self { header: db_headers, db_file_name: db_file_name.clone() };

        Ok(db)
    }

    pub fn execute_command(&self, command: &String) -> Result<()> {
        match command.as_str() {
            ".dbinfo" => {
                println!("{}", self.header);
            }
            _ => bail!("Missing or invalid command passed: {}", command),
        }

        Ok(())
    }
}