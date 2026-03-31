use anyhow::{Context, Result, anyhow};

use crate::core::scanner::{RecordFieldType, RecordValue};

pub trait RecordFieldTypeExt {
    fn decode(&self, data: &[u8]) -> Result<Option<RecordValue>>;
}

impl RecordFieldTypeExt for RecordFieldType {
    fn decode(&self, data: &[u8]) -> Result<Option<RecordValue>> {
        match self {
            RecordFieldType::Null => Ok(Some(RecordValue::Null)),
            RecordFieldType::String(_) => {
                let s = std::str::from_utf8(data).context("invalid string")?;
                Ok(Some(RecordValue::String(s.to_string())))
            }
            RecordFieldType::Blob(_) => Ok(Some(RecordValue::Blob(data.to_vec()))),
            RecordFieldType::I64 => {
                let val = i64::from_be_bytes(data.try_into().context("invalid i64 bytes")?);
                Ok(Some(RecordValue::Int(val)))
            }
            RecordFieldType::Float => {
                let val = f64::from_be_bytes(data.try_into().context("invalid f64 bytes")?);
                Ok(Some(RecordValue::Float(val)))
            }
            RecordFieldType::I32 => {
                let val = i32::from_be_bytes(data.try_into().context("invalid i32 bytes")?);
                Ok(Some(RecordValue::Int(val as i64)))
            }
            RecordFieldType::I16 => {
                let val = i16::from_be_bytes(data.try_into().context("invalid i16 bytes")?);
                Ok(Some(RecordValue::Int(val as i64)))
            }
            RecordFieldType::I8 => {
                let val = i8::from_be_bytes(data.try_into().context("invalid i8 bytes")?);
                Ok(Some(RecordValue::Int(val as i64)))
            }
            RecordFieldType::Zero => Ok(Some(RecordValue::Int(0))),
            RecordFieldType::One => Ok(Some(RecordValue::Int(1))),
            _ => Err(anyhow!("unsupported field type: {:?}", self)),
        }
    }
}
