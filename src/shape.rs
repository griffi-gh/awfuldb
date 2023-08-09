use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;
use crate::types::Type;

#[derive(Serialize, Deserialize)]
pub struct Column {
  pub name: String,
  pub typ: Type,
  pub nullable: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Table {
  pub columns: FxHashMap<String, Column>,
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct DbFragmentation {

}

#[derive(Serialize, Deserialize)]
pub struct DatabaseShape {
  pub tables: FxHashMap<String, Table>,
}
