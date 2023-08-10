use rustc_hash::FxHashMap;
use crate::{
  database::{Database, RwData},
  shape::{Table, Column},
  types::{Type, TextType}
};

pub fn load_test_data<T: RwData>(db: &mut Database<T>)  {
  db.shape.tables.insert("test".to_string(), Table {
    columns: {
      let mut columns = FxHashMap::default();
      columns.insert("name".to_string(), Column {
        typ: Type::Text(TextType { size: 11 }),
        nullable: false,
      });
      columns
    },
    fragmentation: Vec::new(),
    row_count: 0,
  });
  db.write_shape().unwrap();
  db.table_insert("test", "Hello world".as_bytes()).unwrap();
  db.table_insert("test", "Goodbye wld".as_bytes()).unwrap();
}
