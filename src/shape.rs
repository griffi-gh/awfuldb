use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;
use crate::types::{Type, ReprSize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Column {
  pub typ: Type,
  pub nullable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
  pub name: String,
  pub columns: Vec<Column>,
  pub column_map: FxHashMap<String, usize>,
  pub fragmentation: Vec<u64>,
  pub row_count: u64,
}

impl ReprSize for Table {
  /// returns byte size of ROW, not entire TABLE
  fn byte_size(&self) -> usize {
    self.columns.iter().map(|c| c.typ.into_type_tree().byte_size()).sum()
  }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DbShape {
  pub reclaim: VecDeque<u64>,
  pub table_map: FxHashMap<String, usize>,
  pub tables: Vec<Table>,
}

impl DbShape {
  pub fn insert_table(&mut self, name: &str, table: Table) {
    self.table_map.insert(name.to_string(), self.tables.len());
    self.tables.push(table);
  }

  pub fn get_table(&self, name: &str) -> Option<&Table> {
    self.tables.get(*self.table_map.get(name)?)
  }

  pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
    self.tables.get_mut(*self.table_map.get(name)?)
  }
}