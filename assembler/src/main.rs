mod assembler;
mod tokens;
mod assembler_source;

use std::{fs::OpenOptions, io::Write, time::Instant};

use assembler_source::*;

use crate::assembler::Assembler;
use spdlog::prelude::*;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[clap()]
    input: String,
   #[arg(short, long)] 
    output: String,
}

fn main() {
    spdlog::default_logger().set_level_filter(spdlog::LevelFilter::All);

    let args = Args::parse();

    let filename = args.input;

    let text = match std::fs::read_to_string(filename) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error reading file: {e}");
            return;
        }
    };

    let start = Instant::now();

    let mut assembler = Assembler::assemble(text); 
    let mc = assembler.link();

    let end = Instant::now();
    let elapsed = (end - start).as_secs_f64();

    println!("Time taken: {elapsed} seconds");

    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(args.output).unwrap();
    file.write_all(&mc).unwrap();
}
