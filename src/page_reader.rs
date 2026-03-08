use anyhow::{ Ok };

use std::{
    io:: { Read, Seek, SeekFrom },
    sync:: { Arc, Mutex }
};

use crate::{
    database::{ self, DatabaseHeader },
    page::{ Page, PageHeader },
    scanner::{ RecordField, RecordFieldType }};

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
        let offset = if page_num == 1 { database::HEADER_SIZE } else { 0 };
        let content_offset= &data[offset..];
        
        let header = self.parse_page_header(&content_offset)?; 
        let cell_pointers = self.get_cell_pointers(
            &data[offset + header.size..],
            header.cell_count)?;
        
        for cell_pointer in cell_pointers {
            let mut pos = cell_pointer as usize;
            
            let payload_size = read_varint(&data, &mut pos);
            let rowid = read_varint(&data, &mut pos);
            let header_size = read_varint(&data, &mut pos);

            // println!("payload: {}, rowid: {} header:{}", payload_size, rowid,  header_size);

            let mut record_fields = vec!();
            for _ in 0..header_size - 1 {
                let varint = read_varint(&data, &mut pos);
                record_fields.push(self.get_field_type(varint)?);
            }

            let mut offset = cell_pointer as usize + header_size as usize + 2;
            for record_field in record_fields.iter() {
                let value = std::str::from_utf8(
                    &data[offset..offset + record_field.size])?;
                println!("{}", value);
                offset += record_field.size;
            }
        }

        Ok(Page { header, cells: vec!() })
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

    fn get_cell_pointers(&self, buffer: &[u8], cell_count: u16) -> anyhow::Result<Vec<u16>> {
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

    fn get_field_type(&self, serial_type_code:u64) -> anyhow::Result<RecordField> {
        let a= match serial_type_code {
            0 => RecordField::new(RecordFieldType::Null, 0),
            1 => RecordField::new(RecordFieldType::I8, 1),
            2 => RecordField::new(RecordFieldType::I16, 2),
            3 => RecordField::new(RecordFieldType::I24, 3),
            4 => RecordField::new(RecordFieldType::I32, 4),
            5 => RecordField::new(RecordFieldType::I48, 6),
            6 => RecordField::new(RecordFieldType::I64, 8),
            7 => RecordField::new(RecordFieldType::Float, 8),
            8 => RecordField::new(RecordFieldType::Zero, 0),
            9 => RecordField::new(RecordFieldType::One, 0),
            n if n >= 12 && n % 2 == 0 => {
                let size = ((n - 12) / 2) as usize;
                RecordField::new(RecordFieldType::Blob(size), size)
            }
            n if n >= 13 && n % 2 == 1 => {
                let size = ((n - 13) / 2) as usize;
                RecordField::new(RecordFieldType::String(size), size)
            }
            n => anyhow::bail!("unsupported field type: {}", n),
        };

        Ok(a)
    }
}

fn read_varint(data: &[u8], pos: &mut usize) -> u64 {
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