use crate::core::cell::{CellPayload, PageCell};
use crate::core::header::DBHeader;
use crate::core::page::BTreePageSubType::{Interior, Leaf};
use crate::core::page::PageTypes::{IndexBTree, TableBTree};
use anyhow::Result;
use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BTreePageSubType {
    Leaf,
    Interior,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageTypes {
    Lock,
    FreeList,
    PointerMap,
    PayloadOverflow,
    IndexBTree(BTreePageSubType),
    TableBTree(BTreePageSubType),
}

impl From<&u8> for PageTypes {
    fn from(value: &u8) -> Self {
        let page_type = match value {
            2 => IndexBTree(Interior),
            5 => TableBTree(Interior),
            10 => IndexBTree(Leaf),
            13 => TableBTree(Leaf),
            _ => {
                panic!("Invalid Page Type")
            }
        };

        page_type
    }
}

#[derive(Debug)]
pub struct Page<'file> {
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

    file: &'file File,
}

impl<'file> Page<'file> {
    pub fn new(file: &'file File, page_size: u16, page_number: u64) -> Result<Self> {
        let mut buffer = vec![0u8; page_size as usize];

        let page_offset = ((page_number as u16 - 1) * page_size) as u64;

        file.read_exact_at(&mut buffer, page_offset)?;

        let mut start_index = 0;

        let header = DBHeader::new(&buffer)?;

        if page_number == 1 || page_offset == 0 {
            start_index = 100;
        }

        let page_type = PageTypes::from(&buffer[start_index]);

        let free_block_start =
            u16::from_be_bytes([buffer[start_index + 1], buffer[start_index + 2]]);

        let num_of_cells = u16::from_be_bytes([buffer[start_index + 3], buffer[start_index + 4]]);

        let content_area_start =
            u16::from_be_bytes([buffer[start_index + 5], buffer[start_index + 6]]);

        let num_of_fragmented_free_bytes = u8::from_be_bytes([buffer[start_index + 7]]);

        let right_most_pointer_value = u32::from_be_bytes([
            buffer[start_index + 8],
            buffer[start_index + 9],
            buffer[start_index + 10],
            buffer[start_index + 11],
        ]);

        let right_most_pointer = if right_most_pointer_value == 0 {
            None
        } else {
            Some(right_most_pointer_value)
        };

        let mut cells: Vec<PageCell> = Vec::with_capacity(num_of_cells as usize);
        let mut cell_pointers = Vec::with_capacity(num_of_cells as usize);

        let cell_pointers_start_index = match page_type {
            IndexBTree(btree_type) | TableBTree(btree_type) => match btree_type {
                BTreePageSubType::Leaf => start_index + 8,
                BTreePageSubType::Interior => start_index + 12,
            },
            _ => start_index + 8,
        } as u16;

        // each cell is 2 bytes
        let cell_pointers_end_index = cell_pointers_start_index + num_of_cells * 2;

        for i in (cell_pointers_start_index..cell_pointers_end_index).step_by(2) {
            let i: usize = i as usize;

            let pointer = u16::from_be_bytes([buffer[i], buffer[i + 1]]);

            cell_pointers.push(pointer);

            if pointer != 0 {
                let cell_vec = &buffer[(pointer as usize)..].to_vec();

                let cell = PageCell::new(&cell_vec.to_vec(), page_type, &header.text_encoding)?;

                cells.push(cell);
            }
        }

        Ok(Self {
            file,
            cells,
            header,
            page_size,
            page_type,
            num_of_cells,
            cell_pointers,
            free_block_start,
            content_area_start,
            right_most_pointer,
            num_of_fragmented_free_bytes,
        })
    }

    fn cell_content_area_offset(&self) -> u16 {
        // the offset to the cell content area will equal the page size minus the bytes of reserved space
        let (value, is_overflowing) = self
            .page_size
            .overflowing_sub(self.header.reserved_bytes_per_page);

        if is_overflowing {
            self.page_size
        } else {
            value
        }
    }

    pub fn get_payloads(&self, pointers: &mut Vec<u32>) -> Result<Vec<(u32, Rc<CellPayload>)>> {
        let mut pages_len = 0;
        let mut result: Vec<(u32, Rc<CellPayload>)> = vec![];

        for cell in &self.cells {
            match cell.left_pointer {
                None => {
                    result.push((cell.row_id, Rc::clone(&cell.payload)))
                }

                Some(pointer) => {
                    let mut page = Page::new(self.file, self.page_size, pointer as u64)?;

                    pages_len += 1;
                    let mut i = 0;

                    while i < pages_len {
                        match page.page_type {
                            TableBTree(Interior) => {
                                for cell in &page.cells {
                                    match cell.left_pointer {
                                        None => {
                                            result.push((cell.row_id, Rc::clone(&cell.payload)))
                                        }

                                        Some(pointer) => {
                                            if !pointers.contains(&pointer) {
                                                println!("Pointer: {pointer}");
                                                pointers.push(pointer);

                                                let page = Page::new(self.file, self.page_size, pointer as u64)?;

                                                result.append(&mut page.get_payloads(pointers)?);

                                                pages_len += 1;
                                            }
                                        }
                                    }
                                }
                            }

                            TableBTree(Leaf) => {
                                let _ = page.cells.iter().map(|c| result.push((c.row_id, Rc::clone(&c.payload))));
                                break;
                            }

                            otherwise => {
                                println!("PageType: {otherwise:?}");
                            }
                        }

                        i += 1;
                    }
                }
            }
        }

        Ok(result)
    }
}
