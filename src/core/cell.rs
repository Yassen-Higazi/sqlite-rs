use crate::core::page::{BTreePageSubType, PageTypes};

use crate::core::header::TextEncoding;
use crate::utils::parse_varint;
use anyhow::{bail, Context, Result};
use integer_encoding::VarInt;

#[derive(Clone, Debug)]
pub enum ColumnTypes {
    Null,
    Be8bitsInt(u8),
    Be24bitsInt(u8),
    Be16bitsInt(u8),
    Be32bitsInt(u8),
    Be48bitsInt(u8),
    Be64bitsInt(u8),
    Be64bitsFloat(u8),
    Zero,
    One,
    Internal(u64),
    Blob(u64),
    Text(u64),
}

impl ColumnTypes {
    fn new(value: u64) -> Result<Self> {
        let col_type = match value {
            0 => ColumnTypes::Null,
            1 => ColumnTypes::Be8bitsInt(1),
            2 => ColumnTypes::Be16bitsInt(2),
            3 => ColumnTypes::Be24bitsInt(3),
            4 => ColumnTypes::Be32bitsInt(4),
            5 => ColumnTypes::Be48bitsInt(5),
            6 => ColumnTypes::Be64bitsInt(8),
            7 => ColumnTypes::Be64bitsFloat(8),
            8 => ColumnTypes::Zero,
            9 => ColumnTypes::One,
            10 | 11 => ColumnTypes::Internal(value),
            _ => {
                if value >= 12 && value % 2 == 0 {
                    ColumnTypes::Blob((value - 12) / 2)
                } else if value >= 13 && value % 2 == 1 {
                    ColumnTypes::Text((value - 12) / 2)
                } else {
                    bail!("Invalid Column Type, {value}")
                }
            }
        };

        Ok(col_type)
    }
}

#[derive(Clone, Debug)]
pub struct Record {
    pub size: u32,
    pub body: Vec<u8>,
    pub column_types: Vec<ColumnTypes>,
}

impl Record {
    fn from_table_leaf(buffer: &Vec<u8>, _encoding: &TextEncoding) -> Result<Self> {
        let (header_size, size_var_end) = u32::decode_var(buffer).with_context(|| "Could not parse cell size varint")?;

        let mut next_index = size_var_end;

        let body = buffer[header_size as usize..].to_vec();

        let mut column_types = Vec::<ColumnTypes>::with_capacity(header_size as usize);

        let mut bytes = &buffer[next_index..];

        while next_index < header_size as usize {
            let (column, column_bytes, column_size) = parse_varint(bytes).with_context(|| "Could not decode Record serial Type")?;

            bytes = column_bytes;

            let col_type = ColumnTypes::new(column)?;

            column_types.push(col_type);

            next_index += column_size;
        }

        Ok(Self {
            body,
            column_types,
            size: header_size,
        })
    }
}

impl Record {
    pub fn new(buffer: &Vec<u8>, value: PageTypes, encoding: &TextEncoding) -> Result<Self> {
        match value {
            PageTypes::TableBTree(b_tee_type) => {
                match b_tee_type {
                    BTreePageSubType::Leaf => Record::from_table_leaf(buffer, encoding),

                    _ => {
                        Ok(Self {
                            size: 0,
                            body: vec![],
                            column_types: vec![],

                        })
                    }
                }
            }

            _ => {
                bail!("Invalid Btree Type")
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct PageCell {
    pub cell_size: u32,
    pub row_id: u32,
    pub record: Record,
}

impl PageCell {
    pub fn new(buffer: &Vec<u8>, btree_type: PageTypes, encoding: &TextEncoding) -> Result<PageCell> {
        let (size, size_var_end) = u32::decode_var(buffer).with_context(|| "Could not parse cell size varint")?;

        let mut next_index = size_var_end;

        let (rowid, rowid_var_end) = u32::decode_var(&buffer[next_index..buffer.len()])
            .with_context(|| "Could not parse cell rowid varint")?;

        next_index += rowid_var_end;

        let record_buffer = buffer[next_index..].to_vec();

        let cell = Self {
            row_id: rowid,
            cell_size: size,
            record: Record::new(&record_buffer, btree_type, encoding)?,
        };

        Ok(cell)
    }
}
