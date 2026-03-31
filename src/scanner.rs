use anyhow::{ bail };
use std::collections::VecDeque;
use crate::{
    page_reader::{ PageReader, read_varint },
    page::{ Page, Cell },
    ext::RecordFieldTypeExt};

#[derive(Debug, Clone, Copy)]
pub struct Scanner<'a> {
    page_reader: PageReader<'a>
}

impl<'a> Scanner<'a> {
    pub fn new(page_reader: PageReader<'a>) -> Self {
        Scanner {
            page_reader
        }
    }

    pub fn scan(self, root_page: usize) -> anyhow::Result<RecordIter<'a>> {
        let mut pages_to_visit = VecDeque::new();
        pages_to_visit.push_back(root_page);

        Ok(RecordIter {
            page_reader: self.page_reader,
            current_cell: 0,
            pages_to_visit,
            current_page: None
        })
    }
}

pub struct RecordIter<'a> {
    page_reader: PageReader<'a>,
    pages_to_visit: VecDeque<usize>,
    current_page: Option<Page>,
    current_cell: usize,
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = anyhow::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref page) = self.current_page {
                if self.current_cell < page.get_cell_count() {
                    let cell = &page.cells[self.current_cell];
                    self.current_cell += 1;
                    
                    match cell {
                        Cell::TableLeaf { payload } => return Some(parse_record(payload)),
                        Cell::TableInterior { left_child_page } => {
                            self.pages_to_visit.push_back(*left_child_page as usize);
                            continue;
                        }
                    }
                }
                else {
                    if let Some(right_most) = page.header.right_most_pointer {
                        self.pages_to_visit.push_back(right_most as usize);
                    }
                    self.current_page = None;
                }
            }

            match self.pages_to_visit.pop_front() {
                Some(page_num) => {
                    match self.page_reader.read_page(page_num) {
                        Ok(page) => {
                            println!("--- reading page {} ttype: {:?}", page_num, page.header.page_type);
                            self.current_page = Some(page);
                            self.current_cell = 0;
                        }
                        Err(e) => return Some(Err(e))
                    }
                }
                None => return None
            }
        }
    }
}

fn parse_record(payload: &[u8]) -> anyhow::Result<Record> {
    let mut pos = 0;
    let header_size = read_varint(payload, &mut pos) as usize;
    let record_fields = parse_record_fields(payload, header_size, pos)?;

    Ok(Record {
        header: RecordHeader { fields: record_fields },
        payload: payload.to_vec(),
    })
}

fn parse_record_fields(payload: &[u8], header_size: usize, mut pos: usize) -> anyhow::Result<Vec<RecordField>> {
    let mut offset = header_size;
    let mut record_fields = Vec::new();

    for _ in 0..header_size - 1 {
        let type_code = read_varint(payload, &mut pos);
        let (field_type, size) = get_field_type(type_code)?;
        record_fields.push(RecordField { field_type, size, offset });
        offset += size;
    }
    Ok(record_fields)
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

        let payload_slice = &self.payload[record_field.offset..record_field.offset + record_field.size];
        Ok(record_field.field_type.decode(payload_slice)?)
    }

    pub fn to_string(&self) -> anyhow::Result<String> {
        let mut field_values = Vec::new();
        for i in 0..self.header.fields.len() {
            if let Some(value) = self.field(i)? {
                let value = match value {
                    RecordValue::String(s) => s,
                    RecordValue::Int(i) => i.to_string(),
                    RecordValue::Float(i) => i.to_string(),
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
#[allow(dead_code)]
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