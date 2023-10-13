//! public json api to the database

use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;
use anyhow::{Result, Context, ensure, bail};
use crate::{
  database::{Database, RwData, SECTOR_SIZE},
  shape::{Table, Column, DbShape},
  types::{Type, ReprSize, TypeTree, TextType, IntegerType, IntegerSize, FloatType, FloatSize},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DbTypeExt {
  #[serde(rename = "Pointer")]
  UnresolvedPointer(String),

  #[serde(untagged)]
  Type(Type),
}
impl DbTypeExt {
  pub fn resolve(&self, shape: &DbShape) -> Option<Type> {
    Some(match self {
      DbTypeExt::UnresolvedPointer(name) => Type::Pointer(*shape.table_map.get(name)? as u32),
      DbTypeExt::Type(t) => *t,
    })
  }
}

#[derive(Serialize, Deserialize)]
pub struct DbColumn {
  pub name: String,

  #[serde(rename = "type")]
  pub typ: DbTypeExt,

  #[serde(default)]
  pub nullable: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DbRowColumnValue {
  String(String),
  Blob(Vec<u8>),
  Integer(i128),
  Float(f64),
}

macro_rules! impl_to_bytes_as_num {
  // (_0 $self:ident f32) => {
  //   { let DbRowColumnValue::Float(i) = ($self) else { bail!("expected float") }; i }
  // };
  // (_0 $self:ident f64) => {
  //   { let DbRowColumnValue::Float(i) = ($self) else { bail!("expected float") }; i }
  // };
  // (_0 $self:ident $ty:ty) => {
  //   { let DbRowColumnValue::Integer(i) = ($self) else { bail!("expected integer") }; i }
  // };
  ($self: expr, $typ: ident) => {
    {
      // let self_ = $self;
      // let i = impl_to_bytes_as_num!(_0 self_ $typ);
      let DbRowColumnValue::Integer(i) = ($self) else { bail!("expected integer") };
      ensure!($typ::try_from(*i).is_ok(), "integer out of range");
      Ok(Box::new((*i as $typ).to_le_bytes()))
    }
  };
}

impl DbRowColumnValue {
  pub fn serialize_as_type(&self, typ: Type) -> Result<Box<[u8]>> {
    match typ.into_type_tree() {
      TypeTree::Number(nt) => match nt {
        crate::types::NumberType::Integer(it) => match it {
          IntegerType { size: IntegerSize::Int8, is_signed: false } => impl_to_bytes_as_num!(self, u8),
          IntegerType { size: IntegerSize::Int8, is_signed: true } => impl_to_bytes_as_num!(self, i8),
          IntegerType { size: IntegerSize::Int16, is_signed: false } => impl_to_bytes_as_num!(self, u16),
          IntegerType { size: IntegerSize::Int16, is_signed: true } => impl_to_bytes_as_num!(self, i16),
          IntegerType { size: IntegerSize::Int32, is_signed: false } => impl_to_bytes_as_num!(self, u32),
          IntegerType { size: IntegerSize::Int32, is_signed: true } => impl_to_bytes_as_num!(self, i32),
          IntegerType { size: IntegerSize::Int64, is_signed: false } => impl_to_bytes_as_num!(self, u64),
          IntegerType { size: IntegerSize::Int64, is_signed: true } => impl_to_bytes_as_num!(self, i64),
        },
        crate::types::NumberType::Float(FloatType { size }) => {
          let DbRowColumnValue::Float(f) = self else { bail!("expected float") };
          match size {
            FloatSize::Float32 => Ok(Box::new((*f as f32).to_le_bytes())),
            FloatSize::Float64 => Ok(Box::new(f.to_le_bytes())),
          }
        }
      },
      TypeTree::Text(TextType { size }) => {
        let Self::String(s) = self else { bail!("expected string") };
        if s.len() > size { bail!("string is too long") };
        Ok(
          (s.len() as u32).to_le_bytes().iter()
            .chain(s.as_bytes().iter())
            .copied()
            .chain(std::iter::repeat(0).take(size - s.len()))
            .collect()
        )
      },
      _ => todo!("parse other types")
    }
  }

  fn deserialize_as_type(typ: Type, data: &[u8]) -> Result<Self> {
    match typ.into_type_tree() {
      TypeTree::Text(_) => {
        let len = u32::from_le_bytes(data[..4].try_into().context("invalid data length")?) as usize;
        let s = String::from_utf8(data[4..(4 + len)].to_vec()).context("invalid utf8")?;
        Ok(Self::String(s))
      },
      _ => todo!("parse other types")
    }
  }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DbRow {
  AsPositional(Vec<DbRowColumnValue>),
  AsNamed(FxHashMap<String, DbRowColumnValue>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum DbQueryKey {
  ///Simple key, for example `name`
  Simple(String),
  ///Pointer key, for example `["customer", "name"]`
  Pointer(Vec<String>),
}

#[allow(clippy::enum_variant_names)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DbOperation {
  TableCreate {
    name: String,
    columns: Vec<DbColumn>
  },
  TableInsert {
    name: String,
    columns: DbRow,
  },
  TableQuery {
    name: String,
    columns: Vec<DbQueryKey>,
    _rowid: u64
  }
}

#[derive(Serialize, Deserialize)]
pub enum DbOperationResult {
  NoResult,
  TableQuery(Vec<Vec<DbRowColumnValue>>),
}

impl<T: RwData> Database<T> {
  pub fn perform_multiple(&mut self, ops: Vec<DbOperation>) -> Result<Vec<DbOperationResult>> {
    let mut results = vec![];
    for op in ops {
      results.push(self.perform(op)?);
    }
    Ok(results)
  }

  pub fn perform(&mut self, op: DbOperation) -> Result<DbOperationResult> {
    match op {
      DbOperation::TableCreate { name, columns } => {
        if self.shape.get_table(&name).is_some() {
          bail!("table already exists");
        }
        let table = Table {
          name: name.clone(),
          columns: columns.iter().map(|c| -> Result<Column> {
            Ok(Column {
              typ: c.typ.resolve(&self.shape).context("Failed to resolve pointer or type")?,
              nullable: c.nullable,
            })
          }).collect::<Result<Vec<Column>>>()?,
          column_map: {
            let mut map = FxHashMap::default();
            for (idx, column) in columns.iter().enumerate() {
              map.insert(column.name.clone(), idx);
            }
            map
          },
          fragmentation: Vec::new(),
          row_count: 0,
        };
        if table.byte_size() > SECTOR_SIZE {
          bail!("row size is too big. compile with larger sector size or reduce row size");
        }
        self.shape.insert_table(&name, table);
        Ok(DbOperationResult::NoResult)
      },
      DbOperation::TableInsert { name, columns } => {
        let table = self.shape.get_table_mut(&name).context("table not found")?;

        //Get sorted list of values
        //TODO allow omitting nullable in AsNamed
        let values = match columns {
          DbRow::AsNamed(columns) => todo!("handle DbRow::AsNamed"),
          DbRow::AsPositional(columns) => columns,
        };
        ensure!(values.len() == table.columns.len());

        //Create buffer to write
        let mut row_buffer = vec![0; table.byte_size()].into_boxed_slice();
        let mut position = 0;
        for (idx, value) in values.iter().enumerate() {
          let column = &table.columns[idx];
          let value_len = column.typ.into_type_tree().byte_size();
          let value_range = position..(position + value_len);
          let value_buf = value.serialize_as_type(column.typ)?;
          ensure!(value_buf.len() == value_len, "invalid length");
          row_buffer[value_range].copy_from_slice(&value_buf[..]);
          position += value_len;
        }

        self.table_insert(&name, &row_buffer)?;

        Ok(DbOperationResult::NoResult)
      },
      DbOperation::TableQuery { name, columns, _rowid } => {
        let table_idx = *self.shape.table_map.get(&name).context("table not found")?;
        let mut res = Vec::with_capacity(1);
        for key in columns.iter() {
          match key {
            DbQueryKey::Simple(key_name) => {
              let table = &self.shape.tables[table_idx];
              let Some(&col_idx) = table.column_map.get(key_name) else {
                bail!("column not found");
              };
              let column = &table.columns[col_idx];
              let column_type = column.typ;
              match column_type.into_type_tree() {
                TypeTree::Text(_) => {
                  let roco_data = self.table_read_row_column(&name, _rowid, col_idx)?;
                  let value = DbRowColumnValue::deserialize_as_type(column_type, &roco_data)?;
                  res.push(value);
                }
                _ => todo!("handle other types"),
              }
            },
            DbQueryKey::Pointer(_) => todo!("handle DbQueryKey::Pointer"),
          }
        }
        Ok(DbOperationResult::TableQuery(vec![res]))
      },
    }
  }
}
