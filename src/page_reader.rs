use anyhow::{ Ok };
use std::{
    io:: { Read, Seek, SeekFrom },
    sync:: { Arc, Mutex }};

use crate::{
    database::{ self, DatabaseHeader },
    page::{ Page, PageHeader, Cell }};

pub const PAGE_CELLS_COUNT_OFFSET: usize = 3;

#[derive(Debug)]
pub struct PageReader<I: Read + Seek = std::fs::File> {
    db_header: DatabaseHeader,
    file: Arc<Mutex<I>>
}

impl<I: Seek + Read> PageReader<I> {
    pub fn new(db_header: DatabaseHeader, file: I) -> Self {
        Self {
            db_header: db_header,
            file: Arc::new(Mutex::new(file)),
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

        let mut file_guard  = self.file.lock().unwrap();
        file_guard
            .seek(SeekFrom::Start(offset as u64))
            .ok();

        let mut buffer = vec![0; self.db_header.page_size as usize];
        file_guard.read_exact(&mut buffer)?;
        
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
        let mut cells = vec!();
        for cell_pointer in cell_pointers {
            let payload_size = read_varint_at(&data, cell_pointer as usize);
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

pub fn read_varint_at(data: &[u8], mut offset: usize) -> u64 {
    let mut size = 0;
    let mut result = 0;

    while size < 9 {
        let current_byte = data[offset] as u64;
        if size == 8 {
            result = (result << 8) | current_byte;
        } else {
            result = (result << 7) | (current_byte & 0b0111_1111);
        }

        offset += 1;
        size += 1;

        if current_byte & 0b1000_0000 == 0 {
            break;
        }
    }

    result
}