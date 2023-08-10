use std::fs::File;

pub(crate) mod types;
pub(crate) mod shape;
pub(crate) mod database;
pub(crate) mod test_data;

use database::Database;
use test_data::load_test_data;

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
    "test" => {
      let data = File::options().read(true).write(true).open(&args[1]).unwrap();
      let mut db = Database::new(data).unwrap();
      db.read_database().unwrap();
      println!("database loaded");
      load_test_data(&mut db);
      println!("test data loaded");
    }
    _ => {
      let data = File::open(&args[0]).unwrap();
      let mut db = Database::new(data).unwrap();
      db.read_database().unwrap();
      println!("database loaded");
      println!("{:#?}", &db.shape);
    }
  }
}
