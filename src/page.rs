use anyhow::Result;

use crate::header::DBHeader;
use crate::page::BTreePageSubType::{IndexInterior, IndexLeaf, TableInterior, TableLeaf};
use crate::page::PageTypes::BTree;

#[derive(Debug, Clone, Copy)]
pub enum BTreePageSubType {
    IndexInterior,
    IndexLeaf,
    TableInterior,
    TableLeaf,
}

#[derive(Debug, Clone, Copy)]
pub enum PageTypes {
    Lock,
    FreeList,
    PointerMap,
    PayloadOverflow,
    BTree(BTreePageSubType),

}

impl From<&u8> for PageTypes {
    fn from(value: &u8) -> Self {
        let page_type = match value {
            2 => BTree(IndexInterior),
            5 => BTree(IndexLeaf),
            10 => BTree(TableInterior),
            13 => BTree(TableLeaf),
            _ => {
                panic!("Invalid Page Type")
            }
        };


        page_type
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub header: DBHeader,
    page_type: PageTypes,
    page_size: u16,
    free_block_start: u16,
    num_of_cells: u16,
    content_area_start: u16,
    num_of_fragmented_free_bytes: u8,
    right_most_pointer: Option<u32>,

    cell_pointer: Vec<u32>,
}

impl Page {
    pub fn new(buffer: &Vec<u8>, page_size: u16) -> Result<Self> {
        let header = DBHeader::new(&buffer)?;

        let page_type = PageTypes::from(&buffer[0]);

        let free_block_start = u16::from_be_bytes([buffer[1], buffer[2]]);

        let num_of_cells = u16::from_be_bytes([buffer[3], buffer[4]]);

        let content_area_start = u16::from_be_bytes([buffer[5], buffer[6]]);

        let num_of_fragmented_free_bytes = u8::from_be_bytes([buffer[7]]);

        let right_most_pointer_value = u32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);

        let right_most_pointer = if right_most_pointer_value == 0 { None } else { Some(right_most_pointer_value) };

        Ok(Self {
            header,
            page_size,
            page_type,
            num_of_cells,
            free_block_start,
            content_area_start,
            right_most_pointer,
            num_of_fragmented_free_bytes,

            cell_pointer: vec![],
        })
    }
}