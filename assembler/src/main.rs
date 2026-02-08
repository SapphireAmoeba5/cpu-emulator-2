mod assembler;
mod assembler_source;
mod expression;
mod instruction;
mod linker;
mod module;
mod opcode;
mod section;
mod tokens;

use std::{fs::{File, OpenOptions}, io::Write, process::ExitCode, time::Instant};

use crate::{
    assembler::Assembler,
    linker::{Instr, link},
    module::Module,
    tokens::{TokenIter},
};

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[clap()]
    input: Vec<String>,
    #[arg(short, long)]
    output: String,
}

fn main() -> ExitCode {
    spdlog::default_logger().set_level_filter(spdlog::LevelFilter::All);

    let args = Args::parse();

    let mut modules = Vec::with_capacity(args.input.len());

    let start = Instant::now();
    for filename in &args.input {
        let text = match std::fs::read_to_string(filename) {
            Ok(text) => text,
            Err(e) => {
                println!("Error opening file for reading: {e}");
                return ExitCode::FAILURE;
            }
        };

        let assembler = match Assembler::assemble(filename.clone(), text) {
            Ok(assembler) => assembler,
            Err(e) => {
                println!("{e}");
                return ExitCode::FAILURE;
            }
        };

        let module = match Module::try_from(assembler) {
            Ok(module) => module,
            Err(e) => {
                println!("{e}");
                return ExitCode::FAILURE;
            }
        };

        modules.push(module);
    }

    let script = vec![
        Instr::Section(".entry".to_string()),
        Instr::Section("*".to_string()),
    ];
    let program = match link(modules, script) {
        Ok(program) => program,
        Err(_) => {
            return ExitCode::FAILURE;
        }
    };

    let end = Instant::now();
    let elapsed = (end - start).as_secs_f64();

    println!("Time taken: {elapsed}s");

    let mut file = match OpenOptions::new().write(true).create(true).truncate(true).open(&args.output) {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening file for writing: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = file.write_all(&program.linked) {
        println!("Error writing file: {e}");
        return ExitCode::FAILURE;
    }

    println!("Wrote {} bytes", program.linked.len());

    return ExitCode::SUCCESS;
}
