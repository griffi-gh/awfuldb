use anyhow::{Result, Context};
use clap::{Parser, Args, Subcommand};
use colored::*;
use std::{
  fs::File,
  sync::{Arc, Mutex},
  io::{Seek, SeekFrom, self},
  path::PathBuf, net::IpAddr
};
use rouille::{Request, Response};

pub(crate) mod types;
pub(crate) mod shape;
pub(crate) mod database;
pub(crate) mod operations;
pub(crate) mod header;

use database::Database;

#[derive(Parser)]
#[command(author, version, arg_required_else_help = true)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  Create(CreateCommand),
  Run(RunCommand),
}

#[derive(Args)]
struct CreateCommand {
  #[clap(help = "The path to the database file")]
  path: PathBuf,
  #[clap(short = 'f')]
  force: bool,
}

#[derive(Args)]
struct RunCommand {
  #[clap(help = "The path to the database file")]
  path: PathBuf,
  #[clap(short = 'c', help = "Automatically create the database if it doesn't exist")]
  create: bool,
  #[clap(short = 'a', default_value = "127.0.0.1", help = "The address to bind to")]
  addr: IpAddr,
  #[clap(short = 'p', default_value = "12012", help = "The port to bind to")]
  port: u16,
}

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

fn txt_opening(path: &PathBuf) {
  #[allow(clippy::print_literal)] {
  println!(
    "{}{}🗃️ {}",
    path
      .canonicalize()
      .unwrap_or_else(|_| path.clone())
      .as_os_str()
      .to_string_lossy()
      .dimmed(),
    "\n",
    "Opening database file...".bold(),
  );}
}

fn main() {
  println!(
    "{}",
    r#"
         __          ________ _    _ _      _____  ____
        /\ \        / /  ____| |  | | |    |  __ \|  _ \
       /  \ \  /\  / /| |__  | |  | | |    | |  | | |_) |
      / /\ \ \/  \/ / |  __| | |  | | |    | |  | |  _ <
     / ____ \  /\  /  | |    | |__| | |____| |__| | |_) |
    /_/    \_\/  \/   |_|     \____/|______|_____/|____/
    "#.bold().dimmed()
  );
  println!();

  let cli = Cli::parse();
  match &cli.command {
    Some(Commands::Create(args)) => {
      txt_opening(&args.path);
      let data = match File::options().create(args.force).create_new(!args.force).write(true).open(&args.path) {
        Ok(x) => x,
        Err(err) => match err.kind() {
          io::ErrorKind::AlreadyExists => {
            println!(
              "❌ {}\n{}",
              "File already exists".red().bold(),
              "(Use --force to erase/overwrite an existing database)".dimmed()
            );
            return
          }
          _ => panic!("{:?}", err),
        }
      };
      let mut db = Database::new(data).unwrap();
      db.sync_database().unwrap();
      db.truncate().unwrap();
      db.sync_fs().unwrap();
      println!("🐤 {}", "Created new database".bold().green());
    },
    Some(Commands::Run(args)) => {
      txt_opening(&args.path);
      let mut data = match File::options().read(true).write(true).create(args.create).open(&args.path) {
        Ok(x) => x,
        Err(err) => match err.kind() {
          io::ErrorKind::NotFound => {
            println!("❌ {}", "File not found".red().bold());
            return
          }
          _ => panic!("{:?}", err),
        }
      };
      let size = data.seek(SeekFrom::End(0)).unwrap();

      let db = Arc::new(Mutex::new(Database::new(data).unwrap()));
      let mut dblock = db.lock().unwrap();

      if args.create && size == 0 {
        println!("🐤 {}", "Creating new database...".bold());
        dblock.sync_database().unwrap();
        dblock.sync_fs().unwrap();
      } else {
        if args.create {
          println!(
            "⚠️  {} {}",
            "Database already exists".bright_yellow().bold(),
            "(but --create was specified)".dimmed()
          );
        }
        dblock.read_database().unwrap();
      }

      drop(dblock);

      println!(
        "📡 {} `{}:{}`",
        "Running on".green().bold(),
        args.addr, args.port
      );

      rouille::start_server((args.addr, args.port), move |request| {
        handle_error(handle_req(request, &mut db.lock().unwrap_or_else(|poison| {
          poison.into_inner()
        })))
      });
    }
    _ => ()
  }
}
