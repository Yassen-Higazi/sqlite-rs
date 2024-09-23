use crate::core::cell::{ColumnTypes, Record};
use crate::parser::scanner::Scanner;
use crate::parser::statement::Statement;

use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum SchemaTypesTypes {
    Table,
    Index,
    View,
    Trigger,
}

impl Display for SchemaTypesTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SchemaTypesTypes::View => "VIEW",
            SchemaTypesTypes::Table => "TABLE",
            SchemaTypesTypes::Index => "INDEX",
            SchemaTypesTypes::Trigger => "TRIGGER",
        };

        write!(f, "{}", str)
    }
}

impl From<&str> for SchemaTypesTypes {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "VIEW" => SchemaTypesTypes::View,
            "TABLE" => SchemaTypesTypes::Table,
            "INDEX" => SchemaTypesTypes::Index,
            "TRIGGER" => SchemaTypesTypes::Trigger,
            _ => {
                panic!("Invalid SchemaType");
            }
        }
    }
}

impl From<&Record> for SchemaTable {
    fn from(record: &Record) -> Self {
        let mut root_page = 0;

        let mut schema_type = SchemaTypesTypes::Table;

        let mut columns = Vec::with_capacity(3);

        let mut start_index = 0;

        if let ColumnTypes::Text(col_size) = record.column_types[0] {
            let end_index = start_index + col_size as usize;

            let text = std::str::from_utf8(&record.body[start_index..end_index])
                .expect("Could not parse Schema Table");

            schema_type = SchemaTypesTypes::from(text);

            start_index += col_size as usize;
        }

        for i in 1..record.column_types.len() {
            let col_type = &record.column_types[i];

            match col_type {
                ColumnTypes::Be8bitsInt(size) => {
                    let num = u8::from_be_bytes([record.body[start_index]]) as i32;

                    root_page = num;

                    start_index += *size as usize;
                }

                ColumnTypes::Text(col_size) => {
                    let end_index = start_index + *col_size as usize;

                    let text = String::from_utf8(record.body[start_index..end_index].to_vec())
                        .expect("Could not parse Schema Table");

                    columns.push(text);

                    start_index = end_index;
                }
                _ => {}
            }
        }

        let mut scanner = Scanner::new();

        scanner.scan(&columns[2]).expect("Could not scan create statement;");

        let create_statement = Statement::new(scanner.get_tokens()).expect("Could not parse create statement;");

        Self {
            root_page,
            schema_type,
            sql: columns[2].to_owned(),
            name: columns[0].to_owned(),
            statement: create_statement,
            tbl_name: columns[1].to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct SchemaTable {
    pub sql: String,
    pub name: String,
    pub root_page: i32,
    pub tbl_name: String,
    pub statement: Statement,
    pub schema_type: SchemaTypesTypes,
}

impl Display for SchemaTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}",
            self.schema_type, self.name, self.tbl_name, self.root_page, self.sql
        )
    }
}
