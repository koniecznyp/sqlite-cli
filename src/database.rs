use std::{ io::Read, fs::File };
use anyhow::Context;

use crate::{
    page_reader::PageReader,
    scanner::{ Record, Scanner },
    ext::ByteSliceExt};

pub const HEADER_SIZE: usize = 100;
pub const HEADER_PAGE_SIZE_OFFSET: usize = 16;
pub const HEADER_PAGE_COUNT_OFFSET: usize = 28;
pub const HEADER_VERSION_OFFSET: usize = 96;

#[derive(Debug)]
pub struct DatabaseHeader {
    pub page_size: u16,
    pub page_count: u32,
    pub version: u32
}

pub struct Database {
    pub header: DatabaseHeader,
    pub db_file: File,
    pub tables: Vec<Table>
}

impl Database {
    pub fn load_file(filename: impl AsRef<std::path::Path>) -> anyhow::Result<Database> {
        let mut db_file = File::open(&filename)?;

        let mut bytes = [0; HEADER_SIZE];
        db_file.read_exact(&mut bytes)?;

        let header = Database::parse_header(&bytes)?;

        let tables = {
            Database::get_tables(PageReader::new(&header, &db_file))?
        };

        Ok(Database { header, db_file, tables })
    }

    pub fn get_scanner(&self) -> Scanner<'_> {
        Scanner::new(self.get_reader()) 
    }

    fn get_reader(&self) -> PageReader<'_> {
        PageReader::new(&self.header, &self.db_file)
    }

    fn get_tables(page_reader: PageReader<'_>) -> anyhow::Result<Vec<Table>> {
        let scanner = Scanner::new(page_reader);

        let mut tables = Vec::new();
        for record in scanner.scan(1)? {
            if let Some(table) = Table::from_record(&record?)? {
                tables.push(table);
            }
        }

        Ok(tables)
    }

    fn parse_header(bytes: &[u8]) -> anyhow::Result<DatabaseHeader> {
        let page_size = bytes.read_u16_be(HEADER_PAGE_SIZE_OFFSET);
        let page_count = bytes.read_u32_be(HEADER_PAGE_COUNT_OFFSET);
        let version = bytes.read_u32_be(HEADER_VERSION_OFFSET);

        Ok(DatabaseHeader { page_size, page_count, version })
    }
}

pub struct Table {
    pub name: String,
    pub rootpage: usize
}

impl Table {
    pub fn from_record(record: &Record) -> anyhow::Result<Option<Table>> {
        let type_name = record
            .field(0)?
            .context("type")?
            .as_string()
            .context("type must be a string")?;

        if type_name != "table" {
            return Ok(None)
        }

        let tbl_name = record
            .field(2)?
            .context("tbl_name")?
            .as_string()
            .context("tbl_name must be a string")?;

        let rootpage = record
            .field(3)?
            .context("rootpage")?
            .as_int()
            .context("rootpage must be an integer")? as usize;

        Ok(Some(Table { name: tbl_name, rootpage: rootpage }))
    }
}