use std::{ io::Read, fs::File };

use crate::page_reader::PageReader;

pub const HEADER_SIZE: usize = 100;
pub const HEADER_PAGE_SIZE_OFFSET: usize = 16;

pub struct DatabaseHeader {
    pub page_size: u16
}

pub struct Database {
    pub table_count: u16
}

impl Database {
    pub fn load_file(filename: impl AsRef<std::path::Path>) -> anyhow::Result<Database> {
        let mut db_file = File::open(&filename)?;

        let mut bytes = [0; HEADER_SIZE];
        db_file.read_exact(&mut bytes)?;

        let header = parse_header(&bytes)?;

        let page_reader = PageReader::new(header, db_file);

        let table_count = get_table_count(page_reader)?;

        Ok(Database { table_count })
    }
}

fn get_table_count(page_reader: crate::page_reader::PageReader) -> anyhow::Result<u16> {
    let page= page_reader.read_page(1)?;

    let tables = page.get_table_names();

    Ok(page.get_cell_count()?)
}

fn parse_header(bytes: &[u8]) -> anyhow::Result<DatabaseHeader> {
    let page_size = u16::from_be_bytes(
        bytes[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].try_into().unwrap());

    Ok(DatabaseHeader{ page_size })
}
