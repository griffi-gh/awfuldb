//! public json api to the database

use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;
use anyhow::{Result, Context, ensure, bail};
use crate::{
  database::{Database, RwData},
  shape::{Table, Column},
  types::{Type, ReprSize},
};

#[derive(Serialize, Deserialize)]
pub struct DbColumn {
  pub name: String,

  #[serde(rename = "type")]
  pub typ: Type,

  #[serde(default)]
  pub nullable: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DbRowColumnValue {
  String(String),
  Blob(Vec<u8>),
  Integer(u64),
  Float(f64),
}

impl DbRowColumnValue {
  pub fn to_bytes_as(&self, typ: Type) -> Result<Box<[u8]>> {
    match typ {
      Type::Text(size) => {
        let Self::String(s) = self else { bail!("expected string") };
        if s.len() != size { bail!("invalid string size, must match exactly") };
        Ok(s.as_bytes().into())
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
  }
}

#[derive(Serialize, Deserialize)]
pub enum DbOperationResult {
  NoResult,
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
        self.shape.tables.insert(name, Table {
          columns: columns.iter().map(|c| Column {
            typ: c.typ,
            nullable: c.nullable,
          }).collect(),
          column_map: {
            let mut map = FxHashMap::default();
            for (idx, column) in columns.iter().enumerate() {
              map.insert(column.name.clone(), idx);
            }
            map
          },
          fragmentation: Vec::new(),
          row_count: 0,
        });
        Ok(DbOperationResult::NoResult)
      },
      DbOperation::TableInsert { name, columns } => {
        let table = self.shape.tables.get_mut(&name).context("table not found")?;

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
          let value_buf = value.to_bytes_as(column.typ)?;
          ensure!(value_buf.len() == value_len, "invalid length");
          row_buffer[value_range].copy_from_slice(&value_buf[..]);
          position += value_len;
        }

        self.table_insert(&name, &row_buffer)?;

        Ok(DbOperationResult::NoResult)
      },
    }
  }
}
