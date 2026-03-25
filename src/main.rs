use std::{io::{ Write, stdin, stdout}};
use anyhow::{Ok};

use crate::database::Database;

mod database;
mod page_reader;
mod page;
mod scanner;
mod parser;
mod tokenizer;
mod planner;
mod query_plan;
mod executor;

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
            query => process_query(&mut database, query)?
        }

        flush_console();

        input_buffer.clear();
    }

    Ok(())
}

fn list_tables(database: &Database) -> anyhow::Result<()> {
    for table in &database.tables {
        println!("{}", &table.name);
    }
    Ok(())
}

fn process_query(database: &Database, query: &str) -> anyhow::Result<()> {
    let statement = parser::parse_sql(query)?;
    let query_plan = planner::Planner::new(database).compile(&statement)?;
    let mut executor = executor::Executor::new(&query_plan)?;

    while let Some(record) = executor.get_next_row()? {
        println!("{}", record.to_string()?);
    }
    Ok(())
}

fn flush_console() {
    print!("db> ");
    stdout().flush().ok();
}