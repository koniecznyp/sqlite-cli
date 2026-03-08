use anyhow::{ Ok };

use crate::{
    page_reader::{ PageReader }};

pub struct Scanner {
    page_reader: PageReader,
    page_number: usize
}

pub struct Record {
    header: RecordHeader,
    payload: Vec<u8>
}

pub struct RecordHeader {
    pub fields: Vec<RecordField>
}

pub struct RecordField {
    pub field_type: RecordFieldType,
    pub size: usize
}

impl RecordField {
    pub fn new(field_type: RecordFieldType, size: usize) -> Self {
        Self { field_type, size }
    }
}

impl Scanner {
    pub fn new(page_reader: PageReader, page_number: usize) -> Self {
        Scanner {
            page_reader,
            page_number
        }
    }

    pub fn next_record() -> anyhow::Result<Record> {
        Ok(Record { header: RecordHeader { fields: vec!() }, payload: vec!() })
    }
}

pub enum RecordFieldType {
    Null,
    I8,
    I16,
    I24,
    I32,
    I48,
    I64,
    Float,
    Zero,
    One,
    String(usize),
    Blob(usize)
}