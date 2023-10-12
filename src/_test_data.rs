use crate::{
  database::{Database, RwData},
  operations::{DbOperation, DbColumn, DbRow, DbRowColumnValue},
  types::Type
};

pub fn load_test_data<T: RwData>(db: &mut Database<T>)  {
  //"test" table
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
      columns: DbRow::AsPositional(vec![
        DbRowColumnValue::String("Hello world".into()),
      ])
    },
    DbOperation::TableInsert {
      name: "test".into(),
      columns: DbRow::AsPositional(vec![
        DbRowColumnValue::String("Susceptible".into()),
      ])
    },
  ]).unwrap();

  //"spam" table
  db.perform(DbOperation::TableCreate {
    name: "spam".into(),
    columns: vec![
      DbColumn {
        name: "spam_column".into(),
        typ: Type::Text(11),
        nullable: false,
      }
    ]
  }).unwrap();
  for i in 0..1000000 {
    db.perform(DbOperation::TableInsert {
      name: "spam".into(),
      columns: DbRow::AsPositional(vec![
        DbRowColumnValue::String(format!("{: >11}", i)),
      ])
    }).unwrap();
  }

  //"customers" table
  db.perform(DbOperation::TableCreate {
    name: "customers".into(),
    columns: vec![
      DbColumn {
        name: "spam_id".into(),
        typ: Type::Pointer(1),
        nullable: false,
      }
    ]
  }).unwrap();
}
