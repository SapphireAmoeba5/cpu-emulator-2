use crate::{
    assembler::{Assembler, Instruction},
    tokens::{Register, Token, TokenIter},
};
use anyhow::{anyhow, bail, Context, Result};
use spdlog::prelude::*;

/// There are a bunch of instructions that are essentially just a mov but with different opcodes
enum Movlike {
    RegReg {
        destination: Register,
        source: Register,
    },

    RegImm8 {
        destination: Register,
        value: u8,
    },
    RegImm16 {
        destination: Register,
        value: u16,
    },
    RegImm32 {
        destination: Register,
        value: u32,
    },
    RegImm64 {
        destination: Register,
        value: u64,
    },
}

fn parse_movlike(tokens: &mut TokenIter) -> Result<Movlike> {
    let next = tokens.next()?.with_context(|| "Expected a token")?;
    match next {
        Token::Register(reg) => parse_movlike_reg(reg, tokens),
        token => bail!("Expected register but got {}", token.to_string()),
    }
}

fn parse_movlike_reg(register: Register, tokens: &mut TokenIter) -> Result<Movlike> {
    // Consume the comma
    let _ = tokens.is_comma()?;

    let next = tokens.next()?.with_context(|| "Expected token")?;

    match next {
        Token::Register(right) => parse_movlike_reg_reg(register, right, tokens),
        Token::Number(number) => parse_movlike_reg_imm(register, number, tokens),
        other => bail!("Unexpected token {}", other.to_string()),
    }
}

fn parse_movlike_reg_reg(
    left: Register,
    right: Register,
    tokens: &mut TokenIter,
) -> Result<Movlike> {
    // Make sure the next token is either None (reached end of file) or a newline
    tokens.newline_or_eof()?;

    if left.size() != right.size() {
        panic!("Mismatched register sizes");
    }

    Ok(Movlike::RegReg {
        source: right,
        destination: left,
    })
}

fn parse_movlike_reg_imm(
    destination: Register,
    number: u64,
    tokens: &mut TokenIter,
) -> Result<Movlike> {
    tokens.newline_or_eof()?;

    match destination.size() {
        1 => Ok(Movlike::RegImm8 {
            destination,
            value: number as u8,
        }),
        2 => Ok(Movlike::RegImm16 {
            destination,
            value: number as u16,
        }),
        4 => Ok(Movlike::RegImm32 {
            destination,
            value: number as u32,
        }),
        8 => Ok(Movlike::RegImm64 {
            destination,
            value: number,
        }),
        _ => unreachable!(),
    }
}

impl Assembler {
    pub fn parse_mov(&mut self, tokens: &mut TokenIter) -> Result<()> {
        debug!("Parsing mov");
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::MovRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::MovRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::MovRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::MovRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::MovRegImm64 { destination, value }),
        }

        Ok(())
    }
    pub fn parse_add(&mut self, tokens: &mut TokenIter) -> Result<()> {
        debug!("Parsing add");
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::AddRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::AddRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::AddRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::AddRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::AddRegImm64 { destination, value }),
        }

        Ok(())
    }

    pub fn parse_sub(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::SubRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::SubRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::SubRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::SubRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::SubRegImm64 { destination, value }),
        }

        Ok(())
    }

    pub fn parse_mul(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::MulRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::MulRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::MulRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::MulRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::MulRegImm64 { destination, value }),
        }

        Ok(())
    }

    pub fn parse_div(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::DivRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::DivRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::DivRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::DivRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::DivRegImm64 { destination, value }),
        }

        Ok(())
    }

    pub fn parse_idiv(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::IdivRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::IdivRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::IdivRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::IdivRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::IdivRegImm64 { destination, value }),
        }

        Ok(())
    }

    pub fn parse_and(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::AndRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::AndRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::AndRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::AndRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::AndRegImm64 { destination, value }),
        }

        Ok(())
    }

    pub fn parse_or(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::OrRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::OrRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::OrRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::OrRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::OrRegImm64 { destination, value }),
        }

        Ok(())
    }

    pub fn parse_xor(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let movlike = parse_movlike(tokens)?;

        match movlike {
            Movlike::RegReg {
                destination,
                source,
            } => self.instructions.push(Instruction::XorRegReg {
                destination,
                source,
            }),
            Movlike::RegImm8 { destination, value } => self
                .instructions
                .push(Instruction::XorRegImm8 { destination, value }),
            Movlike::RegImm16 { destination, value } => self
                .instructions
                .push(Instruction::XorRegImm16 { destination, value }),
            Movlike::RegImm32 { destination, value } => self
                .instructions
                .push(Instruction::XorRegImm32 { destination, value }),
            Movlike::RegImm64 { destination, value } => self
                .instructions
                .push(Instruction::XorRegImm64 { destination, value }),
        }

        Ok(())
    }

    /// Parse the int instruction
    pub fn parse_int(&mut self, tokens: &mut TokenIter) -> Result<()> {
        match tokens.next()?.with_context(|| "Expected a token")? {
            Token::Number(value) => {
                tokens.newline_or_eof()?;
                self.instructions
                    .push(Instruction::Int { code: value as u8 });
                Ok(())
            }
            other => Err(anyhow!("Expected a number but got {}", other.to_string())),
        }
    }
}
