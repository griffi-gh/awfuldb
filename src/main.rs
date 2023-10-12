use std::fs::File;

pub(crate) mod types;
pub(crate) mod shape;
pub(crate) mod database;
pub(crate) mod operations;
pub(crate) mod header;

mod _test_data;

use database::Database;
use _test_data::load_test_data;

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match args[0].as_str() {
    "create" => {
      let data = File::create(&args[1]).unwrap();
      let mut db = Database::new(&data).unwrap();
      db.sync_database().unwrap();
      db.truncate().unwrap();
      db.sync_fs().unwrap();
      println!("created database");
    }
    "optimize" => {
      let data = File::options().read(true).write(true).open(&args[1]).unwrap();
      let len_before = data.metadata().unwrap().len();
      let mut db = Database::new(&data).unwrap();
      db.read_database().unwrap();
      db.optimize().unwrap();
      db.truncate().unwrap();
      db.sync_database().unwrap();
      db.sync_fs().unwrap();
      let len_after = data.metadata().unwrap().len();
      println!("database optimized\nbefore: {len_before}\nafter: {len_after}");
    }
    "test" => {
      let data = File::options().read(true).write(true).open(&args[1]).unwrap();
      let mut db = Database::new(&data).unwrap();
      db.read_database().unwrap();
      println!("database loaded");
      load_test_data(&mut db);
      println!("test data loaded");
      println!("header: {:?}; shape: {:?}", db.header, db.shape);
      db.sync_database().unwrap();
      db.sync_fs().unwrap();
      println!("db synced");
    }
    _ => {
      let data = File::open(&args[0]).unwrap();
      let mut db = Database::new(data).unwrap();
      db.read_database().unwrap();
      println!("database loaded");
    }
  }
}
