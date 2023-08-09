use rkyv::{Archive, Deserialize, Serialize};
use rustc_hash::FxHashMap;
use crate::types::Type;

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Column {
  pub name: String,
  pub typ: Type,
  pub nullable: bool,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Table {
  pub columns: FxHashMap<String, Column>,
  pub name: String,
}

pub struct DbFragmentation {

}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct DatabaseShape {
  pub tables: FxHashMap<String, Table>,
}
