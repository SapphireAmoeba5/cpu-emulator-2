mod assembler;
mod assembler_source;
mod expression;
mod opcode;
mod tokens;

use std::{
    collections::HashMap, fmt::Binary, fs::OpenOptions, io::Write, ptr::NonNull, time::Instant,
};

use anyhow::{anyhow, bail, Result};
use assembler_source::*;
use expression::*;

use crate::{
    assembler::Assembler,
    tokens::{Token, TokenIter},
};
use spdlog::prelude::*;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[clap()]
    input: String,
    #[arg(short, long)]
    output: String,
}

// #[derive(Debug)]
// pub enum Node {
//     Add(Box<Self>, Box<Self>),
//     Mul(Box<Self>, Box<Self>),
//     Number(u64),
// }

fn output_expression(node: &Node) {
    match node {
        Node::Constant(num) => print!("{}", *num as i64),
        Node::BinaryOp { op, left, right } => {
            output_expression(left);
            let op = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Xor => "^",
            };

            print!(" {op} ");

            output_expression(right);
        }
        Node::Expression(expr) => {
            print!("(");
            output_expression(expr);
            print!(")");
        }
        Node::UnaryOp { op, expr } => match op {
            UnaryOp::Neg => {
                print!("-");
                output_expression(expr);
            }
        },
    }
}

fn main() -> Result<()> {
    spdlog::default_logger().set_level_filter(spdlog::LevelFilter::All);

    let args = Args::parse();

    let filename = args.input;

    let text = match std::fs::read_to_string(filename) {
        Ok(text) => text,
        Err(e) => {
            // eprintln!("Error reading file: {e}");
            return Err(anyhow!("Reading file: {e}"));
        }
    };

    let start = Instant::now();

    let assembler = Assembler::assemble(text)?;

    Ok(())

    // if let Ok(mut assembler) = assembler {
    //     let mc = assembler.link();
    //
    //     let end = Instant::now();
    //     let elapsed = (end - start).as_secs_f64();
    //
    //     println!("Time taken: {elapsed} seconds");
    //
    //     let mut file = OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .truncate(true)
    //         .open(args.output)
    //         .unwrap();
    //     file.write_all(&mc).unwrap();
    //     Ok(())
    // } else {
    //     bail!("Assembly failed")
    // }
}
