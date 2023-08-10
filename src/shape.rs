use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;
use crate::types::{Type, ReprSize};

#[derive(Serialize, Deserialize)]
pub struct Column {
  pub typ: Type,
  pub nullable: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Table {
  pub columns: FxHashMap<String, Column>,
  pub fragmentation: Vec<u64>,
  pub row_count: u64,
}

impl ReprSize for Table {
  /// returns byte size of ROW, not entire TABLE
  fn byte_size(&self) -> usize {
    self.columns.values().map(|c| c.typ.byte_size()).sum()
  }
}

#[derive(Serialize, Deserialize, Default)]
pub struct DatabaseShape {
  pub tables: FxHashMap<String, Table>,
}
