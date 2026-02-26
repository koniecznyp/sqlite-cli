use std::io::{ Write, stdin, stdout };
use anyhow::{ Ok };

mod database;

fn main() -> anyhow::Result<()> {

    let database = database::Database::load_file("test.db")?;
    println!("database page size: {}", database.header.page_size);

    flush_console();


    let mut input_buffer = String::new();

    while stdin().read_line(&mut input_buffer).is_ok() {
        match 
        input_buffer.trim() {
            ".version" => println!("v2137"),
            ".tables" => println!("todo"),
            ".exit" => break,
            _ => { println!("not supported dot-command.") }
        }

        flush_console();

        input_buffer.clear();
    }

    Ok(())
}

fn flush_console() {
    print!("db> ");
    stdout().flush().ok();
}