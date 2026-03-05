use std::{
    io:: { Read, Seek, SeekFrom },
    sync:: { Arc, Mutex }
};

use crate::{
    database::{ self, DatabaseHeader },
    page::{ Page, PageHeader, Cell }};

use anyhow::Ok;

pub const PAGE_CELLS_COUNT_OFFSET: usize = 3;

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
        let offset = if page_num == 1 { database::HEADER_SIZE as u16 } else { 0 };
        let content_offset= &data[offset as usize..];
        
        let header = self.parse_page_header(&content_offset)?; 
        let cells = self.parse_cells(
            &data[offset as usize + header.size as usize..],
            header.cell_count as usize)?;

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

    fn parse_page_header(&self, buffer: &[u8]) -> anyhow::Result<PageHeader> {
        let size = 8;
        let cell_count = u16::from_be_bytes(
            buffer[PAGE_CELLS_COUNT_OFFSET..PAGE_CELLS_COUNT_OFFSET + 2]
            .try_into()
            .unwrap());

        Ok(PageHeader { size, cell_count })
    }

    fn parse_cells(&self, buffer: &[u8], cell_count: usize) -> anyhow::Result<Vec<Cell>> {
        let mut cells = Vec::new();
        for i in 0..cell_count {
            let offset = i * 2;
            let pointer = u16::from_be_bytes(buffer[offset..offset + 2].try_into().unwrap());
            cells.push(Cell { pointer });
        }

        Ok(cells)
    }
}