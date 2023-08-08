use crate::types::Type;

pub struct Column {
  pub name: String,
  pub typ: Type,
  pub nullable: bool,
}

pub struct Table {
  pub rows: Vec<Column>,
  pub name: String,
}

pub struct DatabaseShape {
  pub tables: Vec<Table>,
}
