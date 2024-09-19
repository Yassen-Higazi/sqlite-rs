use crate::cell::PageCell;
use crate::header::DBHeader;
use crate::page::BTreePageSubType::{IndexInterior, IndexLeaf, TableInterior, TableLeaf};
use crate::page::PageTypes::BTree;
use anyhow::{Context, Result};
use std::fmt::Display;

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
    pub page_type: PageTypes,
    pub page_size: u16,
    pub free_block_start: u16,
    pub num_of_cells: u16,
    pub content_area_start: u16,
    pub num_of_fragmented_free_bytes: u8,
    pub right_most_pointer: Option<u32>,
    pub cell_pointers: Vec<u16>,
    pub cells: Vec<PageCell>,

    page_buffer: Vec<u8>,
}

impl Page {
    pub fn new(buffer: &Vec<u8>, page_size: u16) -> Result<Self> {
        let header = DBHeader::new(&buffer)?;

        let page_type = PageTypes::from(&buffer[100]);

        let free_block_start = u16::from_be_bytes([buffer[101], buffer[102]]);

        let num_of_cells = u16::from_be_bytes([buffer[103], buffer[104]]);

        let content_area_start = u16::from_be_bytes([buffer[105], buffer[106]]);

        let num_of_fragmented_free_bytes = u8::from_be_bytes([buffer[107]]);

        let right_most_pointer_value = u32::from_be_bytes([buffer[108], buffer[109], buffer[110], buffer[111]]);

        let right_most_pointer = if right_most_pointer_value == 0 { None } else { Some(right_most_pointer_value) };

        let mut cell: Vec<PageCell> = Vec::with_capacity(num_of_cells as usize);
        let mut cell_pointers = Vec::with_capacity(num_of_cells as usize);

        let cell_pointers_start_index = 112;

        // each cell is 2 bytes
        let cell_pointers_end_index = cell_pointers_start_index + num_of_cells * 2;

        for i in (cell_pointers_start_index..cell_pointers_end_index).step_by(2) {
            let i: usize = i as usize;

            let pointer = u16::from_be_bytes([buffer[i], buffer[i + 1]]);

            cell_pointers.push(pointer);

            PageCell::new(&buffer[(pointer as usize)..buffer.len()].to_vec(), page_type)
                .with_context(|| format!("could not initiate page Cell as: {}", pointer))?;

            // cell.push();
        }

        Ok(Self {
            header,
            page_size,
            page_type,
            num_of_cells,
            cell_pointers,
            free_block_start,
            content_area_start,
            right_most_pointer,
            cells: cell,
            num_of_fragmented_free_bytes,
            page_buffer: buffer.clone(),
        })
    }

    fn cell_content_area_offset(&self) -> u16 {
        // the offset to the cell content area will equal the page size minus the bytes of reserved space
        let (value, is_overflowing) = self.page_size.overflowing_sub(self.header.reserved_bytes_per_page);


        if is_overflowing { self.page_size } else { value }
    }
}