use std::error::Error;

use clap::Parser;
use crate::types::GenerateArgs;

mod types;
mod generator;

fn main() -> Result<(), Box<dyn Error>> {
    let args = GenerateArgs::parse();
    generator::run(&args)?;
    Ok(())
}
