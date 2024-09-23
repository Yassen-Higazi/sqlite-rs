#![allow(warnings)]

mod core;
mod parser;
mod utils;

use core::database::Database;
use std::fs::File;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();

    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        3 => {}
        l => bail!("Expected 2 arguments but got {}", l),
    }

    // Parse command and act accordingly
    let db_file_name = &args[1];

    let command = &args[2];

    let mut file = File::open(&db_file_name)?;

    let db = Database::new(&mut file)?;

    db.execute_command(command)?;

    Ok(())
}
