use crate::core::page::{BTreePageSubType, PageTypes};
use std::rc::Rc;

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
    pub fn new(value: u64) -> Result<Self> {
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

    pub fn get_len(&self) -> u64 {
        match self {
            ColumnTypes::Null => 0,
            ColumnTypes::Be8bitsInt(len) => *len as u64,
            ColumnTypes::Be16bitsInt(len) => *len as u64,
            ColumnTypes::Be24bitsInt(len) => *len as u64,
            ColumnTypes::Be32bitsInt(len) => *len as u64,
            ColumnTypes::Be48bitsInt(len) => *len as u64,
            ColumnTypes::Be64bitsInt(len) => *len as u64,
            ColumnTypes::Be64bitsFloat(len) => *len as u64,
            ColumnTypes::Internal(len) => *len,
            ColumnTypes::Blob(len) => *len,
            ColumnTypes::Text(len) => *len,
            _ => 0,
        }
    }

    pub fn print(&self, data: &[u8]) -> Result<()> {
        match self {
            ColumnTypes::Internal(_) => {}
            ColumnTypes::One => {
                print!("1")
            }
            ColumnTypes::Zero => {
                print!("0")
            }
            ColumnTypes::Null => {
                print!("Null")
            }
            ColumnTypes::Blob(_) => {
                print!("{data:?}")
            }
            ColumnTypes::Text(_) => {
                print!("{}", std::str::from_utf8(data)?)
            }
            ColumnTypes::Be8bitsInt(_) => {
                print!("{}", u8::from_be_bytes([data[0]]))
            }
            ColumnTypes::Be16bitsInt(_) => {
                print!("{}", u16::from_be_bytes([data[0], data[1]]))
            }
            ColumnTypes::Be24bitsInt(_) => {
                print!("{}", u32::from_be_bytes([data[0], data[1], data[2], 0]))
            }
            ColumnTypes::Be32bitsInt(_) => {
                print!(
                    "{}",
                    u32::from_be_bytes([data[0], data[1], data[2], data[3]])
                )
            }
            ColumnTypes::Be48bitsInt(_) => {
                print!(
                    "{}",
                    u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], 0, 0, 0])
                )
            }
            ColumnTypes::Be64bitsInt(_) => {
                print!(
                    "{}",
                    u64::from_be_bytes([
                        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]
                    ])
                )
            }
            ColumnTypes::Be64bitsFloat(_) => {
                print!(
                    "{}",
                    f64::from_be_bytes([
                        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]
                    ])
                )
            }
        };

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CellPayload {
    pub size: u32,
    pub body: Vec<u8>,
    pub column_types: Vec<ColumnTypes>,
}

impl CellPayload {
    fn from_table_leaf(buffer: &Vec<u8>, _encoding: &TextEncoding) -> Result<Self> {
        let (header_size, mut bytes, header_size_var_end) =
            parse_varint(buffer).with_context(|| "Could not parse cell size varint")?;

        let mut next_index = header_size_var_end;

        let body = buffer[header_size as usize..].to_vec();

        let mut column_types = Vec::<ColumnTypes>::with_capacity(header_size as usize);

        while next_index <= header_size as usize {
            let (column, column_bytes, column_size) =
                parse_varint(bytes).with_context(|| "Could not decode Record serial Type")?;

            bytes = column_bytes;

            let col_type = ColumnTypes::new(column)?;

            column_types.push(col_type);

            next_index += column_size;
        }

        Ok(Self {
            body,
            column_types,
            size: header_size as u32,
        })
    }
}

impl CellPayload {
    pub fn new(buffer: &Vec<u8>, value: PageTypes, encoding: &TextEncoding) -> Result<Self> {
        // println!("B-tree Type: {value:?}");
        match value {
            PageTypes::TableBTree(b_tee_type) | PageTypes::IndexBTree(b_tee_type) => match b_tee_type {
                BTreePageSubType::Leaf => CellPayload::from_table_leaf(buffer, encoding),

                _ => Ok(Self {
                    size: 0,
                    body: Vec::with_capacity(0),
                    column_types: Vec::with_capacity(0),
                }),
            },

            _ => {
                bail!("Invalid Btree Type")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PageCell {
    pub cell_size: u64,
    pub row_id: u64,
    pub payload: Rc<CellPayload>,
    pub left_pointer: Option<u32>,

    pub overflow_pointers: u32,
}

impl PageCell {
    pub fn new(
        buffer: &Vec<u8>,
        btree_type: PageTypes,
        usable_page_size: u64,
        encoding: &TextEncoding,
    ) -> Result<PageCell> {
        let (size, size_var_end) =
            u64::decode_var(buffer).with_context(|| "Could not parse cell size varint")?;

        let (is_overflowing, payload_size, overflow_size) = PageCell::is_overflowing(btree_type, size, usable_page_size);

        let mut end_index = buffer.len();

        if is_overflowing {
            // end_index = (end_index - overflow_size as usize);
            println!("Is overflowing: {is_overflowing}, Size: {size}, Buffer len: {end_index}, Payload Size: {payload_size}, Overflow Size: {overflow_size}");
        }

        let mut next_index = std::cmp::min(payload_size as usize, size_var_end);

        let left_pointer: Option<u32> = match btree_type {
            PageTypes::IndexBTree(sub_type) | PageTypes::TableBTree(sub_type) => match sub_type {
                BTreePageSubType::Leaf => None,

                BTreePageSubType::Interior => {
                    let res = u32::from_be_bytes([
                        buffer[next_index],
                        buffer[next_index + 1],
                        buffer[next_index + 2],
                        buffer[next_index + 3],
                    ]);

                    next_index += 4;

                    Some(res)
                }
            },

            _ => None,
        };

        let (rowid, rowid_var_end) = u64::decode_var(&buffer[next_index..end_index])
            .with_context(|| "Could not parse cell rowid varint")?;

        next_index += std::cmp::min(rowid as usize, rowid_var_end);

        let record_buffer = buffer[next_index..end_index].to_vec();

        let payload = CellPayload::new(&record_buffer, btree_type, encoding)?;

        let mut overflow = 0;

        if is_overflowing {
            let len = end_index;

            let overflow_page_pointer = u32::from_be_bytes([
                buffer[len - 4],
                buffer[len - 3],
                buffer[len - 2],
                buffer[len - 1],
            ]);

            println!("Buffer: {:?}", buffer[len - 20..].to_vec());

            overflow = overflow_page_pointer;

            println!("Overflow page Pointer: {overflow_page_pointer}");
        }

        let cell = Self {
            left_pointer,
            row_id: rowid,
            cell_size: size,
            payload: Rc::new(payload),
            overflow_pointers: overflow,
        };

        Ok(cell)
    }

    fn is_overflowing(page_type: PageTypes, payload_size: u64, page_size: u64) -> (bool, u64, u64) {
        // TODO: pass usable page from db header
        let usable_size = page_size - 64;

        let max_payload_size = match page_type {
            PageTypes::TableBTree(BTreePageSubType::Leaf) => {
                usable_size - 35
            }

            PageTypes::IndexBTree(_) => {
                ((usable_size - 12) * 64 / 255) - 23
            }

            _ => payload_size
        };

        if payload_size <= max_payload_size {
            (false, payload_size, 0)
        } else {
            let min_payload_size = ((usable_size - 12) * 32 / 255) - 23;
            let in_page_payload_size = min_payload_size + ((payload_size - min_payload_size) % (usable_size - 4));

            if in_page_payload_size <= max_payload_size {
                (true, in_page_payload_size, payload_size - in_page_payload_size)
            } else {
                (true, min_payload_size, payload_size - min_payload_size)
            }
        }
    }
}

