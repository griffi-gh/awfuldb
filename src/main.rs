pub(crate) mod types;
pub(crate) mod shape;
pub(crate) mod database;

use std::fs::File;
use database::Database;

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match args[0].as_str() {
    "create" => {
      let data = File::create(&args[1]).unwrap();
      let mut db = Database::new(&data).unwrap();
      db.write_shape().unwrap();
      data.sync_all().unwrap();
      println!("created database");
    }
    _ => {
      let data = File::open(&args[0]).unwrap();
      let mut db = Database::new(data).unwrap();
      db.read_shape().unwrap();
      println!("parsed shape, starting service");
    }
  }
}
