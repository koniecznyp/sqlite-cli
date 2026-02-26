use std::io::{ Read };

use anyhow::Context;

pub const HEADER_SIZE: usize = 100;
pub const HEADER_PAGE_SIZE_OFFSET: usize = 16;

pub struct DatabaseHeader {
    pub page_size: u16
}

pub struct Database {
    pub header: DatabaseHeader
}

impl Database {
    pub fn load_file(filename: impl AsRef<std::path::Path>) -> anyhow::Result<Database> {
        let mut db_file = std::fs::File::open(filename.as_ref())
            .context("open database file")?;

        let mut bytes = [0; HEADER_SIZE];
        db_file .read_exact(&mut bytes)
            .context("read db header")?;

        let header = parse_header(&bytes)
            .context("parse db header")?;

        Ok(Database { header })
    }
}

fn parse_header(bytes: &[u8]) -> anyhow::Result<DatabaseHeader> {
    let page_size = u16::from_be_bytes(
        bytes[HEADER_PAGE_SIZE_OFFSET..HEADER_PAGE_SIZE_OFFSET + 2].try_into().unwrap());

    Ok(DatabaseHeader{ page_size })
}
