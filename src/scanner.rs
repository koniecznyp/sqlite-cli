use anyhow::{ Ok, bail };

use crate::page_reader::{ PageReader, read_varint };
use std::borrow::Cow;
pub struct Scanner {
    page_reader: PageReader,
    page_number: usize
}

pub struct Record {
    pub header: RecordHeader,
    pub payload: Vec<u8>
}

impl Record {
    pub fn field(&self, n: usize) -> anyhow::Result<Option<RecordValue>> {
        // todo: calculate value from record
        Ok(Some(RecordValue::String("tabelka".into())))
    }
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

impl Scanner {
    pub fn new(page_reader: PageReader, page_number: usize) -> Self {
        Scanner {
            page_reader,
            page_number
        }
    }

    pub fn scan(&self) -> anyhow::Result<Vec<Record>> {
        let page = self.page_reader.read_page(1)?;

        let mut records = vec!();
        for i in 0..page.get_cell_count() {
            let cell = match page.get(i) {
                Some (cell) => cell,
                _ => { anyhow::bail!("cannot read cell") }
            };
            let mut pos = 0;

            let header_size = read_varint(&cell.payload, &mut pos);

            let mut record_fields = vec!();
            for _ in 0..header_size - 1 {
                let varint = read_varint(&cell.payload, &mut pos);
                record_fields.push(self.get_field_type(varint)?);
            }

            let mut offset = header_size as usize;
            for record_field in record_fields.iter() {
                let value = std::str::from_utf8(
                    &cell.payload[offset..offset + record_field.size])?;
                println!("offset {} | {}", offset, value);
                offset += record_field.size;
            }

            records.push(Record { 
                header: RecordHeader { fields: record_fields },
                payload: cell.payload.clone() });
        }

        Ok(records)
    }

    fn get_field_type(&self, serial_type_code:u64) -> anyhow::Result<RecordField> {
        Ok(match serial_type_code {
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
        })
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