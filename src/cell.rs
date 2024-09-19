use crate::page::{BTreePageSubType, PageTypes};
use anyhow::{bail, Context, Result};
use integer_encoding::VarInt;

#[derive(Clone, Debug)]
pub struct Record {
    pub size: u32,
    pub column_types: Vec<u32>,
    pub body: Vec<Vec<u8>>,
}

impl Record {
    pub fn new(buffer: &Vec<u8>, value: PageTypes) -> Result<Self> {
        match value {
            PageTypes::BTree(bTee_type) => {
                match bTee_type {
                    BTreePageSubType::TableLeaf => {
                        Ok(Self {
                            size: 0,
                            column_types: vec![],
                            body: vec![],
                        })
                    }

                    _ => {
                        Ok(Self {
                            size: 0,
                            column_types: vec![],
                            body: vec![],
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
    pub fn new(buffer: &Vec<u8>, btree_type: PageTypes) -> Result<PageCell> {
        let (size, size_var_end) = u32::decode_var(buffer).with_context(|| "Could not parse cell size varint")?;

        let mut next_index = size_var_end + 1;

        let (rowid, rowid_var_end) = u32::decode_var(&buffer[next_index..buffer.len()])
            .with_context(|| "Could not parse cell rowid varint")?;

        next_index += rowid_var_end;

        let record_buffer = buffer[next_index..size as usize].to_vec();

        println!("{size} -> {size_var_end} \n {rowid} -> {rowid_var_end} \n {next_index} -> {record_buffer:?}");

        Ok(Self {
            row_id: rowid,
            cell_size: size,
            record: Record::new(&record_buffer, btree_type)?,
        })
    }

    fn parse_varint(buffer: &Vec<u8>) -> Vec<u8> {
        let mut num_vec = vec![];

        for i in 0..buffer.len() {
            let rest = buffer[i] & 0b01111111;
            let first_bit = buffer[i] & 0b10000000;

            if first_bit == 0 {
                num_vec.push(rest);

                break;
            } else {
                num_vec.push(rest);
            }
        }

        num_vec.reverse();

        num_vec
    }
}
