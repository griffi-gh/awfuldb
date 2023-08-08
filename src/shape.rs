use crate::types::Type;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive)]
pub struct Column {
  pub name: String,
  pub typ: Type,
  pub nullable: bool,
}

#[derive(Archive)]
pub struct Table {
  pub rows: Vec<Column>,
  pub name: String,
}

#[derive(Archive, Serialize, Deserialize)]
pub struct DatabaseShape {
  pub tables: Vec<Table>,
}
