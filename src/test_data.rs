use crate::{
  database::{Database, RwData},
  operations::{DbOperation, DbColumn, DbRow, DbRowColumnValue},
  types::Type
};

pub fn load_test_data<T: RwData>(db: &mut Database<T>)  {
  db.perform_multiple(vec![
    DbOperation::TableCreate {
      name: "test".into(),
      columns: vec![
        DbColumn {
          name: "test".into(),
          typ: Type::Text(11),
          nullable: false,
        }
      ]
    },
    DbOperation::TableInsert {
      name: "test".into(),
      columns: DbRow::AsPositional { columns: vec![
        DbRowColumnValue::String("Hello world".into()),
      ]}
    },
    DbOperation::TableInsert {
      name: "test".into(),
      columns: DbRow::AsPositional { columns: vec![
        DbRowColumnValue::String("Susceptible".into()),
      ]}
    },
  ]).unwrap();
}
