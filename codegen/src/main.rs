use std::error::Error;

use clap::Parser;
use codegen::{generator, types::GenerateArgs};

fn main() -> Result<(), Box<dyn Error>> {
    let args = GenerateArgs::parse();
    generator::run(&args)?;
    Ok(())
}
