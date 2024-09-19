mod header;
mod schema;
mod database;
mod page;
mod cell;

use crate::database::Database;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();

    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        3 => {}
        l => bail!("Expected 2 arguments but got {}", l)
    }

    // Parse command and act accordingly
    let db_file_name = &args[1];

    let command = &args[2];

    let db = Database::new(db_file_name)?;

    db.execute_command(command)?;

    Ok(())
}
