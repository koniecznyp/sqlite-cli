use anyhow::{ Context, Ok, bail };
use crate::page_reader::{ PageReader, read_varint };
use crate::page::Page;

#[derive(Debug)]
pub struct Scanner<'a> {
    page_reader: &'a PageReader
}

impl<'a> Scanner<'a> {
    pub fn new(page_reader: &'a PageReader) -> Self {
        Scanner {
            page_reader
        }
    }

    pub fn scan(&self, page_num: usize) -> anyhow::Result<RecordIter> {
        let page = self.page_reader.read_page(page_num)?;
        Ok(RecordIter {
            page,
            current_cell: 0,
        })
    }
}

pub struct RecordIter {
    page: Page,
    current_cell: usize,
}

impl Iterator for RecordIter {
    type Item = anyhow::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_cell >= self.page.header.cell_count as usize {
            return None;
        }

        let cell = match self.page.cells.get(self.current_cell) {
            Some(cell) => cell,
            None => {
                self.current_cell += 1;
                return Some(Err(anyhow::anyhow!("cannot read cell")));
            }
        };

        self.current_cell += 1;

        let result = parse_record(&cell.payload);
        Some(result)
    }
}

fn parse_record(payload: &[u8]) -> anyhow::Result<Record> {
    let mut pos = 0;
    let header_size = read_varint(payload, &mut pos) as usize;

    let mut offset = header_size;
    let mut record_fields = Vec::new();
    
    for _ in 0..header_size - 1 {
        let type_code = read_varint(payload, &mut pos);
        let (field_type, size) = get_field_type(type_code)?;
        record_fields.push(RecordField { field_type, size, offset });
        offset += size;
    }

    Ok(Record {
        header: RecordHeader { fields: record_fields },
        payload: payload.to_vec(),
    })
}

fn get_field_type(serial_type_code:u64) -> anyhow::Result<(RecordFieldType, usize)> {
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

pub struct Record {
    pub header: RecordHeader,
    pub payload: Vec<u8>
}

impl Record {
    pub fn field(&self, n: usize) -> anyhow::Result<Option<RecordValue>> {
        let Some(record_field) = self.header.fields.get(n) else {
            return Ok(None);
        };

        match record_field.field_type {
            RecordFieldType::Null => Ok(Some(RecordValue::Null)),
            RecordFieldType::String(_) => {
                let s = std::str::from_utf8(
                    &self.payload[record_field.offset..record_field.offset + record_field.size])?;
                Ok(Some(RecordValue::String(s.to_string())))
            }
            RecordFieldType::Blob(_) => {
                let blob = self.payload[record_field.offset..record_field.offset + record_field.size].to_vec();
                Ok(Some(RecordValue::Blob(blob)))
            }
            RecordFieldType::I64 => {
                let bytes = &self.payload[record_field.offset..record_field.offset + record_field.size];
                let val = i64::from_be_bytes(bytes.try_into().context("Invalid i64 bytes")?);
                Ok(Some(RecordValue::Int(val)))
            }
            RecordFieldType::Float => {
                let bytes = &self.payload[record_field.offset..record_field.offset + record_field.size];
                let val = f64::from_be_bytes(bytes.try_into().context("Invalid f64 bytes")?);
                Ok(Some(RecordValue::Float(val)))
            }
            RecordFieldType::I32 => {
                let bytes = &self.payload[record_field.offset..record_field.offset + record_field.size];
                let val = i32::from_be_bytes(bytes.try_into().context("Invalid i32 bytes")?);
                Ok(Some(RecordValue::Int(val as i64)))
            }
            RecordFieldType::I16 => {
                let bytes = &self.payload[record_field.offset..record_field.offset + record_field.size];
                let val = i16::from_be_bytes(bytes.try_into().context("Invalid i16 bytes")?);
                Ok(Some(RecordValue::Int(val as i64)))
            }
            RecordFieldType::I8 => {
                let bytes = &self.payload[record_field.offset..record_field.offset + record_field.size];
                let val = i8::from_be_bytes(bytes.try_into().context("Invalid i8 bytes")?);
                Ok(Some(RecordValue::Int(val as i64)))
            }
            RecordFieldType::Zero => Ok(Some(RecordValue::Int(0))),
            RecordFieldType::One => Ok(Some(RecordValue::Int(1))),
            _ => anyhow::bail!("Unsupported field type: {:?}", record_field.field_type),
        }
    }

    pub fn to_string(&self) -> anyhow::Result<String> {
        let mut field_values = Vec::new();
        for i in 0..self.header.fields.len() {
            if let Some(value) = self.field(i)? {
                let value = match value {
                    RecordValue::String(s) => s,
                    RecordValue::Int(i) => i.to_string(),
                    _ => bail!("unsupported value type for to_string")
                };
                field_values.push(value);
            }
        }
        Ok(field_values.join("|"))
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

pub enum RecordValue {
    Null,
    String(String),
    Blob(Vec<u8>),
    Int(i64),
    Float(f64)
}

impl RecordValue {
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None
        }
    }
}

#[derive(Debug)]
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