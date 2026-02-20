mod assembler;
mod assembler_source;
mod expression;
mod instruction;
mod linker;
mod module;
mod opcode;
mod section;
mod tokens;

use std::{
    collections::{HashMap, btree_map::Entry},
    fs::{File, OpenOptions},
    hash::Hash,
    io::Write,
    process::ExitCode,
    time::Instant,
};

use crate::{
    assembler::{emit::PREFIX_BYTE, Assembler},
    instruction::Mnemonic,
    linker::{link, Instr},
    module::Module,
    opcode::EncodingFlags,
    tokens::TokenIter,
};

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[clap(required_unless_present = "map")]
    input: Vec<String>,
    #[arg(short, long, required_unless_present = "map", default_value_t = String::new())]
    output: String,

    #[clap(long, default_value_t = false)]
    map: bool,
}

fn output_opcode_map() {
    use assembler::emit::EXTENSION_BYTE;
    use opcode::encodings;
    use std::collections::BTreeMap;

    let mut opcodes: BTreeMap<u16, Vec<Mnemonic>> = BTreeMap::new();

    for (mnemonic, encodings) in encodings() {
        let mut encoding_opcodes: HashMap<u16, ()> = HashMap::new();

        for encoding in encodings {
            let mut opcode: u16 = 0;
            if encoding.extension {
                opcode = u16::from(EXTENSION_BYTE) << 8;
            }
            opcode |= u16::from(encoding.opcode);

            if encoding.options.intersects(EncodingFlags::OPCODE_REG) {
                if (opcode & 0x0f != 0) {
                    panic!("OPCODE_REG encoding must have the lowest 4 bits set to zero");
                }

                // Encodings with OPCODE_REG set encodes the register operand in the lowest 4 bits
                // of the opcode
                for i in 0..16 {
                    let opcode = opcode | i;
                    if !encoding_opcodes.contains_key(&opcode) {
                        match opcodes.entry(opcode) {
                            Entry::Occupied(mut entry) => entry.get_mut().push(mnemonic),

                            Entry::Vacant(entry) => {
                                entry.insert(vec![mnemonic]);
                            }
                        }
                    }
                    encoding_opcodes.insert(opcode, ());
                }
            } else {
                if !encoding_opcodes.contains_key(&opcode) {
                    match opcodes.entry(opcode) {
                        Entry::Occupied(mut entry) => entry.get_mut().push(mnemonic),

                        Entry::Vacant(entry) => {
                            entry.insert(vec![mnemonic]);
                        }
                    }
                }
                encoding_opcodes.insert(opcode, ());
            }
        }
    }

    println!("==============================================");
    println!("All opcodes");
    for (opcode, mnemonics) in &opcodes {
        let width = if *opcode > 0xff { 6 } else { 4 };
        println!("{opcode:#0width$x} = {mnemonics:?}");
    }
    println!("==============================================");

    println!("==============================================");
    println!("Collisions");
    for (opcode, mnemonics) in &opcodes {
        if mnemonics.len() > 1 {
            let width = if *opcode > 0xff { 6 } else { 4 };
            println!("{opcode:#0width$x} = {mnemonics:?}");
        }
    }
    println!("==============================================");

    println!("================================================================");
    println!("Base opcode map:");

    print!("     ");
    for i in 0..16 {
        print!("{i:02x} ");
    }
    println!("\n");

    for h in 0..16 {
        print!("{:02x}   ", h << 4);
        for l in 0..16 {
            let opcode: u16 = (h << 4) | l;

            if opcodes.contains_key(&opcode) {
                print!("{opcode:02x} ");
            } else if opcode == u16::from(EXTENSION_BYTE) || opcode & 0xfff0 == u16::from(PREFIX_BYTE) {
                print!("xx ");
            } else {
                print!("-- ");
            }
        }
        println!();
    }
    println!("================================================================");

    println!("Extended opcode map:");

    print!("     ");
    for i in 0..16 {
        print!("{i:02x} ");
    }
    println!("\n");

    for h in 0..16 {
        print!("{:02x}   ", h << 4);
        for l in 0..16 {
            let opcode: u16 = u16::from(EXTENSION_BYTE) << 8 | (h << 4) | l;

            if opcodes.contains_key(&opcode) {
                // Clear the extension byte
                let opcode = opcode & 0xff;
                print!("{opcode:02x} ");
            } else {
                print!("-- ");
            }
        }
        println!();
    }

    println!("================================================================");
}

fn main() -> ExitCode {
    spdlog::default_logger().set_level_filter(spdlog::LevelFilter::All);

    let args = Args::parse();

    if args.map {
        output_opcode_map();
        return ExitCode::SUCCESS;
    }

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
        Instr::Section(".text".to_string()),
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

    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.output)
    {
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
