use anyhow::{ Context, Ok };
use std::{
    cell::RefCell,
    io:: { Read, Seek, SeekFrom }};

use crate::{
    database::{ self, DatabaseHeader },
    page::{ Page, PageHeader, Cell }};

pub const PAGE_CELLS_COUNT_OFFSET: usize = 3;

#[derive(Debug)]
pub struct PageReader<I: Read + Seek = std::fs::File> {
    db_header: DatabaseHeader,
    file: RefCell<I>,
}

impl<I: Seek + Read> PageReader<I> {
    pub fn new(db_header: DatabaseHeader, file: I) -> Self {
        Self {
            db_header: db_header,
            file: RefCell::new(file),
        }
    }

    pub fn read_page(&self, page_num: usize) -> anyhow::Result<Page> {
        let data = self.read_file_content(page_num)?;
        let offset = if page_num == 1 { database::HEADER_SIZE } else { 0 };
        let content_offset= &data[offset..];
        
        let header = Self::parse_page_header(&content_offset)?; 
        let cell_pointers = Self::get_cell_pointers(
            &data[offset + header.size..],
            header.cell_count)?;
        let cells = Self::get_cells(&data, cell_pointers)?;

        Ok(Page { header, cells })
    }

    fn read_file_content(&self, page_num: usize) -> anyhow::Result<Vec<u8>> {
        let offset = page_num.saturating_sub(1) * self.db_header.page_size as usize;

        let mut file = self.file.borrow_mut();

        file.seek(SeekFrom::Start(offset as u64))?;

        let mut buffer = vec![0; self.db_header.page_size as usize];
        file.read_exact(&mut buffer)?;
        
        Ok(buffer)
    }

    fn parse_page_header(buffer: &[u8]) -> anyhow::Result<PageHeader> {
        let size = 8;
        let cell_count = u16::from_be_bytes(
            buffer[PAGE_CELLS_COUNT_OFFSET..PAGE_CELLS_COUNT_OFFSET + 2]
            .try_into()
            .unwrap());

        Ok(PageHeader { size, cell_count })
    }

    fn get_cell_pointers(buffer: &[u8], cell_count: u16) -> anyhow::Result<Vec<u16>> {
        let mut cell_pointers = Vec::new();
        for i in 0..cell_count {
            let offset = (i * 2) as usize;
            cell_pointers.push(u16::from_be_bytes(
                buffer[offset..offset + 2]
                .try_into()
                .unwrap()));
        }
        Ok(cell_pointers)
    }

    fn get_cells(data: &[u8], cell_pointers: Vec<u16>) -> anyhow::Result<Vec<Cell>> {
        let mut cells = Vec::new();
        for cell_pointer in cell_pointers {
            let mut pos = cell_pointer as usize;
            let payload_size = read_varint(&data, &mut pos);
            let offset = cell_pointer as usize + 2;
            cells.push(Cell { payload: data[offset..offset + payload_size as usize].to_vec()});
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
}