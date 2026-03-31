use std::{
    io::{ Read, Seek, SeekFrom },
    fs::File
};

use crate::{
    database::{ self, DatabaseHeader },
    page::{ Page, PageType, PageHeader, Cell },
    ext::ByteSliceExt};

pub const PAGE_CELLS_COUNT_OFFSET: usize = 3;
pub const PAGE_TYPE_TABLE_LEAF: u8 = 13;
pub const PAGE_TYPE_TABLE_INTERIOR: u8 = 5;
pub const PAGE_RIGHT_MOST_POINTER_OFFSET: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct PageReader<'a> {
    db_header: &'a DatabaseHeader,
    file: &'a File
}

impl<'a> PageReader<'a> {
    pub fn new(db_header: &'a DatabaseHeader, file: &'a File) -> Self {
        Self {
            db_header,
            file,
        }
    }

    pub fn read_page(&self, page_num: usize) -> anyhow::Result<Page> {
        let data = self.read_file_content(page_num)?;
        let offset = if page_num == 1 { database::HEADER_SIZE } else { 0 };
        let content_offset = &data[offset..];
        
        let header = Self::parse_page_header(&content_offset)?; 
        let cell_pointers = Self::get_cell_pointers(
            &data[offset + header.size..],
            header.cell_count)?;
        let cells = Self::get_cells(&data, cell_pointers, &header.page_type)?;

        Ok(Page { header, cells })
    }

    fn read_file_content(&self, page_num: usize) -> anyhow::Result<Vec<u8>> {
        let offset = page_num.saturating_sub(1) * self.db_header.page_size as usize;
        let mut file = self.file; 
        file.seek(SeekFrom::Start(offset as u64))?;
        let mut buffer = vec![0; self.db_header.page_size as usize];
        file.read_exact(&mut buffer)?;
        
        Ok(buffer)
    }

    fn parse_page_header(buffer: &[u8]) -> anyhow::Result<PageHeader> {
        let (page_type, page_size) = Self::parse_page_type(&buffer)?;
        let cell_count = buffer.read_u16_be(PAGE_CELLS_COUNT_OFFSET);
        let right_most_pointer = Self::get_right_most_ptr(&buffer, &page_type);

        Ok(PageHeader { page_type, size: page_size, cell_count, right_most_pointer})
    }

    fn get_right_most_ptr(buffer: &[u8], page_type: &PageType) -> Option<u32> {
        match page_type {
            PageType::TableInterior => Some(buffer.read_u32_be(PAGE_RIGHT_MOST_POINTER_OFFSET)),
            _ => None
        }
    }

    fn parse_page_type(buffer: &[u8]) -> anyhow::Result<(PageType, usize)> {
        Ok(match buffer[0] {
            PAGE_TYPE_TABLE_LEAF => (PageType::TableLeaf, 8),
            PAGE_TYPE_TABLE_INTERIOR => (PageType::TableInterior, 12),
            _ => anyhow::bail!("unknown page type")
        })
    }

    fn get_cell_pointers(buffer: &[u8], cell_count: u16) -> anyhow::Result<Vec<u16>> {
        let mut cell_pointers = Vec::new();
        for i in 0..cell_count {
            let offset = (i * 2) as usize;
            cell_pointers.push(buffer.read_u16_be(offset));
        }
        Ok(cell_pointers)
    }

    fn get_cells(data: &[u8], cell_pointers: Vec<u16>, page_type: &PageType) -> anyhow::Result<Vec<Cell>> {
        let mut cells = Vec::new();
        for cell_pointer in cell_pointers {
            let mut pos = cell_pointer as usize;
            match page_type {
                PageType::TableLeaf => {
                    let payload_size = read_varint(&data, &mut pos);
                    let _rowid = read_varint(&data, &mut pos);
                    
                    cells.push(Cell::TableLeaf { 
                        payload: data[pos..pos + payload_size as usize].to_vec() 
                    });
                },
                PageType::TableInterior => {
                    let left_child_page = data.read_u32_be(pos);
                    cells.push(Cell::TableInterior { left_child_page });
                }
            }
        }
        Ok(cells)
    }
}

pub fn read_varint(data: &[u8], pos: &mut usize) -> u64 {
    let mut value = 0u64;
    let mut shift = 0u64;

    loop {
        if *pos >= data.len() {
            panic!("Out of bounds");
        }

        let byte = data[*pos];
        *pos += 1;

        value |= ((byte & 0x7F) as u64) << shift;
   
        if byte & 0x80 == 0 {
            return value;
        }
  
        shift += 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&[0x00], 0)]
    #[test_case(&[0x01], 1)]
    #[test_case(&[0x64], 100)]
    #[test_case(&[0xE8, 0x07], 1000)]
    #[test_case(&[0xA0, 0x8D, 0x06], 100_000)]
    #[test_case(&[0xC0, 0x84, 0x3D], 1_000_000)]
    fn read_varint_tests(data: &[u8], expected: u64) {
        let mut pos = 0;
        let value = read_varint(data, &mut pos);
        assert_eq!(value, expected);
    }

    #[test]
    fn get_table_leaf_cell_test_() {
        let data: [u8; 5] = [0x03, 0x01, 0x03, 0x04, 0x05];
        let cell_pointers = vec![0];

        let cells = PageReader::get_cells(&data, cell_pointers, &PageType::TableLeaf).unwrap();

        assert_eq!(
            cells,
            vec![Cell::TableLeaf { payload: vec![0x03, 0x04, 0x05] }]);
    }

    #[test]
    fn get_table_interior_cell_test_() {
        let data: [u8; 4] = [0x00, 0x00, 0x00, 0x11];
        let cell_pointers = vec![0];

        let cells = PageReader::get_cells(&data, cell_pointers, &PageType::TableInterior).unwrap();

        assert_eq!(
            cells,
            vec![Cell::TableInterior { left_child_page: 17u32 } ]);
    }
}