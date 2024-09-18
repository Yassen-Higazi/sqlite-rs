use std::fmt::{Display, Formatter};

pub enum SchemaTypes {
    Table,
    Index,
    View,
    Trigger,
}

impl Display for SchemaTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SchemaTypes::View => "VIEW",
            SchemaTypes::Table => "TABLE",
            SchemaTypes::Index => "INDEX",
            SchemaTypes::Trigger => "TRIGGER",
        };

        write!(f, "{}", str)
    }
}

impl From<&str> for SchemaTypes {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "VIEW" => SchemaTypes::View,
            "TABLE" => SchemaTypes::Table,
            "INDEX" => SchemaTypes::Index,
            "TRIGGER" => SchemaTypes::Trigger,
            _ => {
                panic!("Invalid SchemaType");
            }
        }
    }
}

struct Schema {
    sql: String,
    name: String,
    root_page: i32,
    tbl_name: String,
    schema_type: SchemaTypes,
}

impl Display for Schema {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}|{}|{}", self.schema_type, self.name, self.tbl_name, self.root_page, self.sql)
    }
}