mod header;

use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;
use crate::header::DBHeader;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();

    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];

    let mut file = File::open(&args[1])?;

    match command.as_str() {
        ".dbinfo" => {

            let mut header = [0; 100];

            file.read_exact(&mut header)?;

            let db_headers = DBHeader::new(&header);

            println!("{}", db_headers);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
