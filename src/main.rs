use anyhow::{Result, Context};
use std::{fs::File, sync::{Arc, Mutex}, io::{SeekFrom, Seek}};
use rouille::{Request, Response};

pub(crate) mod types;
pub(crate) mod shape;
pub(crate) mod database;
pub(crate) mod operations;
pub(crate) mod header;

use database::Database;

mod _test_data;
use _test_data::load_test_data;

fn handle_req(request: &Request, db: &mut Database<File>) -> Result<Response> {
  let req = serde_json::from_reader(request.data().context("no request body")?)?;
  let res = db.perform_multiple(req)?;
  db.sync_database()?;
  // if let Err(err) = db.sync_fs() {
  //   eprint!("failed to sync db to fs: {}", err);
  // }
  Ok(Response::json(&res))
}

fn handle_error(request: Result<Response>) -> Response {
  request.unwrap_or_else(|err| {
    Response::json(&err.to_string()).with_status_code(500)
  })
}

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match args[0].as_str() {
    "create" => {
      let data = File::create(&args[1]).unwrap();
      let mut db = Database::new(data).unwrap();
      db.sync_database().unwrap();
      db.truncate().unwrap();
      db.sync_fs().unwrap();
      println!("created database");
    }
    // "optimize" => {
    //   let data = File::options().read(true).write(true).open(&args[1]).unwrap();
    //   let len_before = data.metadata().unwrap().len();
    //   let mut db = Database::new(data).unwrap();
    //   db.read_database().unwrap();
    //   db.optimize().unwrap();
    //   db.truncate().unwrap();
    //   db.sync_database().unwrap();
    //   db.sync_fs().unwrap();
    //   let len_after = data.metadata().unwrap().len();
    //   println!("database optimized\nbefore: {len_before}\nafter: {len_after}");
    // }
    "test" => {
      let data = File::options().read(true).write(true).open(&args[1]).unwrap();
      let mut db = Database::new(data).unwrap();
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
      let mut data = File::options().read(true).write(true).create(true).open(&args[0]).unwrap();
      //Check if the file is empty and needs init
      let size = data.seek(SeekFrom::End(0)).unwrap();
      let db = Arc::new(Mutex::new(Database::new(data).unwrap()));
      let mut dblock = db.lock().unwrap();
      if size == 0 {
        println!("file empty, creating new database");
        dblock.sync_database().unwrap();
        dblock.sync_fs().unwrap();
      }
      dblock.read_database().unwrap();
      drop(dblock);
      println!("database loaded, starting the server");
      rouille::start_server("127.0.0.1:12012", move |request| {
        handle_error(handle_req(request, &mut db.lock().unwrap_or_else(|poison| {
          poison.into_inner()
        })))
      });
    }
  }
}
