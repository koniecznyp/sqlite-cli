use anyhow::{ Ok };

use crate::page_reader::{ PageReader, read_varint };
use std::{ borrow::Cow };

pub struct Scanner {
    page_reader: PageReader
}

impl Scanner {
    pub fn new(page_reader: PageReader) -> Self {
        Scanner {
            page_reader
        }
    }

    pub fn scan(&self, page_num: usize) -> anyhow::Result<Vec<Record>> {
        let page = self.page_reader.read_page(page_num)?;

        let mut records = vec!();
        for i in 0..page.get_cell_count() {
            let cell = match page.get(i) {
                Some (cell) => cell,
                _ => { anyhow::bail!("cannot read cell") }
            };
            let mut pos = 0;

            let header_size = read_varint(&cell.payload, &mut pos);

            let mut offset = header_size as usize;
            let mut record_fields = vec!();
            for _ in 0..header_size - 1 {
                let varint = read_varint(&cell.payload, &mut pos);

                let (field_type, size) = self.get_field_type(varint)?;
                record_fields.push(RecordField { field_type, size, offset });
                offset += size;
            }

            records.push(Record { 
                header: RecordHeader { fields: record_fields },
                payload: cell.payload.clone() });
        }

        Ok(records)
    }

    fn get_field_type(&self, serial_type_code:u64) -> anyhow::Result<(RecordFieldType, usize)> {
        Ok(match serial_type_code {
            0 => (RecordFieldType::Null, 0),
            1 => (RecordFieldType::I8, 1),
            2 => (RecordFieldType::I16, 2),
            3 => (RecordFieldType::I24, 3),
            4 => (RecordFieldType::I32, 4),
            5 => (RecordFieldType::I48, 6),
            6 => (RecordFieldType::I64, 8),
            7 => (RecordFieldType::Float, 8),
            8 => (RecordFieldType::Zero, 0),
            9 => (RecordFieldType::One, 0),
            n if n >= 12 && n % 2 == 0 => {
                let size = ((n - 12) / 2) as usize;
                (RecordFieldType::Blob(size), size)
            }
            n if n >= 13 && n % 2 == 1 => {
                let size = ((n - 13) / 2) as usize;
                (RecordFieldType::String(size), size)
            }
            n => anyhow::bail!("unsupported field type: {}", n),
        })
    }
}

pub struct Record {
    pub header: RecordHeader,
    pub payload: Vec<u8>
}

impl Record {
    pub fn field(&self, n: usize) -> anyhow::Result<Option<RecordValue>> {
        let Some(record_field) = self.header.fields.get(n) else {
            return Ok(None);
        };

        let value = std::str::from_utf8(
            &self.payload[record_field.offset..record_field.offset + record_field.size])?;

        Ok(Some(RecordValue::String(value.into())))
    }
}

pub struct RecordHeader {
    pub fields: Vec<RecordField>
}

pub struct RecordField {
    pub field_type: RecordFieldType,
    pub size: usize,
    pub offset: usize
}

pub enum RecordValue<'p> {
    Null,
    String(Cow<'p, str>),
    Blob(Cow<'p, [u8]>),
    Int(i64),
    Float(f64)
}

impl<'p> RecordValue<'p> {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RecordValue::String(cow) => Some(cow.as_ref()),
            _ => None,
        }
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