use std::{ io::Read, fs::File };

use anyhow::Context;

use crate::{
    page_reader::PageReader,
    scanner::{ Record, RecordValue, Scanner }};

pub const HEADER_SIZE: usize = 100;
pub const HEADER_PAGE_SIZE_OFFSET: usize = 16;

#[derive(Debug, Copy, Clone)]
pub struct DatabaseHeader {
    pub page_size: u16
}

pub struct Database {
    page_reader: PageReader,
    pub tables: Vec<Table>
}

impl Database {
    pub fn load_file(filename: impl AsRef<std::path::Path>) -> anyhow::Result<Database> {
        let mut db_file = File::open(&filename)?;

        let mut bytes = [0; HEADER_SIZE];
        db_file.read_exact(&mut bytes)?;

        let header = Database::parse_header(&bytes)?;

        let page_reader = PageReader::new(header, db_file);

        let tables = Database::get_tables(page_reader.clone())?;

        Ok(Database { page_reader, tables })
    }

    fn get_tables(page_reader: PageReader) -> anyhow::Result<Vec<Table>> {
        let scanner = Scanner::new(page_reader);

        let mut tables = vec!();
        for record in scanner.scan(1)? {
            tables.push(Table::from_record(&record)?);
        }
        
        Ok(tables)
    }

    fn parse_header(bytes: &[u8]) -> anyhow::Result<DatabaseHeader> {
        let page_size = u16::from_be_bytes(
            bytes[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].try_into().unwrap());

        Ok(DatabaseHeader{ page_size })
    }
}

pub struct Table {
    pub name: String
}

impl Table {
    pub fn from_record(record: &Record) -> anyhow::Result<Table> {
        let tbl_name = record
            .field(2)?
            .context("tbl_name")?;

        let tbl_name_value: String = match tbl_name {
            RecordValue::String(cow) => cow.as_ref().to_string(),
            _ => panic!("string expected")
        };

        Ok(Table { name: tbl_name_value })
    }
}