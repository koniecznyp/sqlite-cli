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

fn cli(database: Database) -> anyhow::Result<()> {
    flush_console();

    let mut input_buffer = String::new();

    while stdin().read_line(&mut input_buffer).is_ok() {
        match input_buffer.trim() {
            ".exit" => break,
            ".dbinfo" => show_dbinfo(&database)?,
            ".tables" => list_tables(&database)?,
            query => process_query(&database, query)?
        }

        flush_console();

        input_buffer.clear();
    }

    Ok(())
}

fn show_dbinfo(database: &Database) -> anyhow::Result<()> {
    println!(
        "database page size:\t{}\n\
         database page count:\t{}\n\
         software version:\t{}\n\
         ...other metadata",
        database.header.page_size,
        database.header.page_count,
        database.header.version);
    Ok(())
}

fn list_tables(database: &Database) -> anyhow::Result<()> {
    let mut tables = database.tables.iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<_>>();
    tables.sort();
    print!("{}\n", tables.join("\t"));
    Ok(())
}

fn process_query(database: &Database, query: &str) -> anyhow::Result<()> {
    let statement = parser::parse_sql(query)?;
    let planner = planner::Planner::new(database);
    let query_plan = planner.compile(&statement)?;
    let mut executor = executor::Executor::new(&query_plan)?;

    while let Some(record) = executor.get_next_row()? {
        println!("{}", record.to_string()?);
    }
    Ok(())
}

fn flush_console() {
    print!("db> ");
    let _ = stdout().flush();
}