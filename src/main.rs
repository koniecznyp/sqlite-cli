use std::io::{ Write, stdin, stdout };
use anyhow::{ Ok };

use crate::database::Database;

mod database;
mod page_reader;
mod page;
mod scanner;

fn main() -> anyhow::Result<()> {
    let database = database::Database::load_file("test.db")?;
    cli(database)
}

fn cli(mut database: Database) -> anyhow::Result<()> {
    flush_console();

    let mut input_buffer = String::new();

    while stdin().read_line(&mut input_buffer).is_ok() {
        match input_buffer.trim() {
            ".exit" => break,
            ".tables" => list_tables(&mut database)?,
            _ => { println!("not supported dot-command.") }
        }

        flush_console();

        input_buffer.clear();
    }

    Ok(())
}

fn list_tables(database: &mut Database) -> anyhow::Result<()> {
    println!("table count: {}", &database.table_count);
    // todo - list table names
    Ok(())
}

fn flush_console() {
    print!("db> ");
    stdout().flush().ok();
}