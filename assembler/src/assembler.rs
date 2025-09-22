mod machine_code;
mod parse;

use std::fmt::Display;

use anyhow::anyhow;
use spdlog::debug;

use super::assembler_source::*;
use super::tokens::*;

use anyhow::{Context, Result};

pub enum Instruction {
    /// 3 byte instruction
    MovRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    MovRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    MovRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    MovRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    MovRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    AddRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    AddRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    AddRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    AddRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    AddRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    SubRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    SubRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    SubRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    SubRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    SubRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    MulRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    MulRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    MulRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    MulRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    MulRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    DivRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    DivRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    DivRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    DivRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    DivRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    IdivRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    IdivRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    IdivRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    IdivRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    IdivRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    AndRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    AndRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    AndRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    AndRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    AndRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    OrRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    OrRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    OrRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    OrRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    OrRegImm64 {
        destination: Register,
        value: u64,
    },

    /// 3 byte instruction
    XorRegReg {
        destination: Register,
        source: Register,
    },
    /// 3 byte instruction
    XorRegImm8 {
        destination: Register,
        value: u8,
    },
    /// 4 byte instruction
    XorRegImm16 {
        destination: Register,
        value: u16,
    },
    /// 6 byte instruction
    XorRegImm32 {
        destination: Register,
        value: u32,
    },
    /// 10 byte instruction
    XorRegImm64 {
        destination: Register,
        value: u64,
    },

    // 2 byte instruction
    Int {
        code: u8,
    },
}

impl Instruction {
    /// Returns the number of bytes the instruction takes up
    pub fn size(&self) -> usize {
        use Instruction::*;
        match self {
            MovRegReg { .. }
            | AddRegReg { .. }
            | SubRegReg { .. }
            | MulRegReg { .. }
            | DivRegReg { .. }
            | IdivRegReg { .. }
            | AndRegReg { .. }
            | OrRegReg { .. }
            | XorRegReg { .. } => 3,

            MovRegImm8 { .. }
            | AddRegImm8 { .. }
            | SubRegImm8 { .. }
            | MulRegImm8 { .. }
            | DivRegImm8 { .. }
            | IdivRegImm8 { .. }
            | AndRegImm8 { .. }
            | OrRegImm8 { .. }
            | XorRegImm8 { .. } => 3,

            MovRegImm16 { .. }
            | AddRegImm16 { .. }
            | SubRegImm16 { .. }
            | MulRegImm16 { .. }
            | DivRegImm16 { .. }
            | IdivRegImm16 { .. }
            | AndRegImm16 { .. }
            | OrRegImm16 { .. }
            | XorRegImm16 { .. } => 4,

            MovRegImm32 { .. }
            | AddRegImm32 { .. }
            | SubRegImm32 { .. }
            | MulRegImm32 { .. }
            | DivRegImm32 { .. }
            | IdivRegImm32 { .. }
            | AndRegImm32 { .. }
            | OrRegImm32 { .. }
            | XorRegImm32 { .. } => 6,

            MovRegImm64 { .. }
            | AddRegImm64 { .. }
            | SubRegImm64 { .. }
            | MulRegImm64 { .. }
            | DivRegImm64 { .. }
            | IdivRegImm64 { .. }
            | AndRegImm64 { .. }
            | OrRegImm64 { .. }
            | XorRegImm64 { .. } => 10,
            Int { .. } => 2,
        }
    }

    /// Gets the opcode
    pub fn opcode(&self) -> u8 {
        use Instruction::*;
        match self {
            MovRegReg { .. } => 0x05,
            MovRegImm8 { .. } => 0x06,
            MovRegImm16 { .. } => 0x07,
            MovRegImm32 { .. } => 0x08,
            MovRegImm64 { .. } => 0x09,

            AddRegReg { .. } => 0x15,
            AddRegImm8 { .. } => 0x16,
            AddRegImm16 { .. } => 0x17,
            AddRegImm32 { .. } => 0x18,
            AddRegImm64 { .. } => 0x19,

            SubRegReg { .. } => 0x25,
            SubRegImm8 { .. } => 0x26,
            SubRegImm16 { .. } => 0x27,
            SubRegImm32 { .. } => 0x28,
            SubRegImm64 { .. } => 0x29,

            MulRegReg { .. } => 0x35,
            MulRegImm8 { .. } => 0x36,
            MulRegImm16 { .. } => 0x37,
            MulRegImm32 { .. } => 0x38,
            MulRegImm64 { .. } => 0x39,

            DivRegReg { .. } => 0x45,
            DivRegImm8 { .. } => 0x46,
            DivRegImm16 { .. } => 0x47,
            DivRegImm32 { .. } => 0x48,
            DivRegImm64 { .. } => 0x49,

            IdivRegReg { .. } => 0x55,
            IdivRegImm8 { .. } => 0x56,
            IdivRegImm16 { .. } => 0x57,
            IdivRegImm32 { .. } => 0x58,
            IdivRegImm64 { .. } => 0x59,

            AndRegReg { .. } => 0x65,
            AndRegImm8 { .. } => 0x66,
            AndRegImm16 { .. } => 0x67,
            AndRegImm32 { .. } => 0x68,
            AndRegImm64 { .. } => 0x69,

            OrRegReg { .. } => 0x75,
            OrRegImm8 { .. } => 0x76,
            OrRegImm16 { .. } => 0x77,
            OrRegImm32 { .. } => 0x78,
            OrRegImm64 { .. } => 0x79,

            XorRegReg { .. } => 0x85,
            XorRegImm8 { .. } => 0x86,
            XorRegImm16 { .. } => 0x87,
            XorRegImm32 { .. } => 0x88,
            XorRegImm64 { .. } => 0x89,

            Int { .. } => 0x01,
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MovRegReg {
                destination,
                source,
            } => write!(f, "mov {destination}, {source}"),
            Instruction::MovRegImm8 { destination, value } => {
                write!(f, "mov {destination}, {value}")
            }
            Instruction::MovRegImm16 { destination, value } => {
                write!(f, "mov {destination}, {value}")
            }
            Instruction::MovRegImm32 { destination, value } => {
                write!(f, "mov {destination}, {value}")
            }
            Instruction::MovRegImm64 { destination, value } => {
                write!(f, "mov {destination}, {value}")
            }
            Instruction::AddRegReg {
                destination,
                source,
            } => write!(f, "add {destination}, {source}"),
            Instruction::AddRegImm8 { destination, value } => {
                write!(f, "add {destination}, {value}")
            }
            Instruction::AddRegImm16 { destination, value } => {
                write!(f, "add {destination}, {value}")
            }
            Instruction::AddRegImm32 { destination, value } => {
                write!(f, "add {destination}, {value}")
            }
            Instruction::AddRegImm64 { destination, value } => {
                write!(f, "add {destination}, {value}")
            }
            Instruction::SubRegReg {
                destination,
                source,
            } => write!(f, "sub {destination}, {source}"),
            Instruction::SubRegImm8 { destination, value } => {
                write!(f, "sub {destination}, {value}")
            }
            Instruction::SubRegImm16 { destination, value } => {
                write!(f, "sub {destination}, {value}")
            }
            Instruction::SubRegImm32 { destination, value } => {
                write!(f, "sub {destination}, {value}")
            }
            Instruction::SubRegImm64 { destination, value } => {
                write!(f, "sub {destination}, {value}")
            }
            Instruction::MulRegReg {
                destination,
                source,
            } => write!(f, "mul {destination}, {source}"),
            Instruction::MulRegImm8 { destination, value } => {
                write!(f, "mul {destination}, {value}")
            }
            Instruction::MulRegImm16 { destination, value } => {
                write!(f, "mul {destination}, {value}")
            }
            Instruction::MulRegImm32 { destination, value } => {
                write!(f, "mul {destination}, {value}")
            }
            Instruction::MulRegImm64 { destination, value } => {
                write!(f, "mul {destination}, {value}")
            }
            Instruction::DivRegReg {
                destination,
                source,
            } => write!(f, "div {destination}, {source}"),
            Instruction::DivRegImm8 { destination, value } => {
                write!(f, "div {destination}, {value}")
            }
            Instruction::DivRegImm16 { destination, value } => {
                write!(f, "div {destination}, {value}")
            }
            Instruction::DivRegImm32 { destination, value } => {
                write!(f, "div {destination}, {value}")
            }
            Instruction::DivRegImm64 { destination, value } => {
                write!(f, "div {destination}, {value}")
            }
            Instruction::IdivRegReg {
                destination,
                source,
            } => write!(f, "idiv {destination}, {source}"),
            Instruction::IdivRegImm8 { destination, value } => {
                write!(f, "idiv {destination}, {value}")
            }
            Instruction::IdivRegImm16 { destination, value } => {
                write!(f, "idiv {destination}, {value}")
            }
            Instruction::IdivRegImm32 { destination, value } => {
                write!(f, "idiv {destination}, {value}")
            }
            Instruction::IdivRegImm64 { destination, value } => {
                write!(f, "idiv {destination}, {value}")
            }

            Instruction::AndRegReg {
                destination,
                source,
            } => write!(f, "and {destination}, {source}"),
            Instruction::AndRegImm8 { destination, value } => {
                write!(f, "and {destination}, {value}")
            }
            Instruction::AndRegImm16 { destination, value } => {
                write!(f, "and {destination}, {value}")
            }
            Instruction::AndRegImm32 { destination, value } => {
                write!(f, "and {destination}, {value}")
            }
            Instruction::AndRegImm64 { destination, value } => {
                write!(f, "and {destination}, {value}")
            }

            Instruction::OrRegReg {
                destination,
                source,
            } => write!(f, "or {destination}, {source}"),
            Instruction::OrRegImm8 { destination, value } => {
                write!(f, "or {destination}, {value}")
            }
            Instruction::OrRegImm16 { destination, value } => {
                write!(f, "or {destination}, {value}")
            }
            Instruction::OrRegImm32 { destination, value } => {
                write!(f, "or {destination}, {value}")
            }
            Instruction::OrRegImm64 { destination, value } => {
                write!(f, "or {destination}, {value}")
            }

            Instruction::XorRegReg {
                destination,
                source,
            } => write!(f, "xor {destination}, {source}"),
            Instruction::XorRegImm8 { destination, value } => {
                write!(f, "xor {destination}, {value}")
            }
            Instruction::XorRegImm16 { destination, value } => {
                write!(f, "xor {destination}, {value}")
            }
            Instruction::XorRegImm32 { destination, value } => {
                write!(f, "xor {destination}, {value}")
            }
            Instruction::XorRegImm64 { destination, value } => {
                write!(f, "xor {destination}, {value}")
            }
            Instruction::Int { code } => {
                write!(f, "int {code}")
            }
        }
    }
}

pub struct Assembler {
    // The instructions
    instructions: Vec<Instruction>,
}

impl Assembler {
    pub fn assemble(source: String) -> Result<Self, ()> {
        let mut assembler = Assembler {
            instructions: Vec::new(),
        };

        let result = assembler.parse_source(source);

        if result {
            Ok(assembler)
        } else {
            Err(())
        }
    }

    /// Links the generated code and returns the machine code for it
    pub fn link(&mut self) -> Vec<u8> {
        use Instruction::*;
        // The machine code
        let mut mc = Vec::new();

        // for instruction in self.instructions.iter() {
        for i in 0..self.instructions.len() {
            let instruction = &self.instructions[i];
            // Push the opcode right away. It's the same for every instruction
            mc.push(instruction.opcode());

            match instruction {
                MovRegReg {
                    source,
                    destination,
                }
                | AddRegReg {
                    destination,
                    source,
                }
                | SubRegReg {
                    destination,
                    source,
                }
                | MulRegReg {
                    destination,
                    source,
                }
                | DivRegReg {
                    destination,
                    source,
                }
                | IdivRegReg {
                    destination,
                    source,
                }
                | AndRegReg {
                    destination,
                    source,
                }
                | OrRegReg {
                    destination,
                    source,
                }
                | XorRegReg {
                    destination,
                    source,
                } => self.assemble_mov_reg_reg(*destination, *source, &mut mc),
                MovRegImm8 { destination, value }
                | AddRegImm8 { destination, value }
                | SubRegImm8 { destination, value }
                | MulRegImm8 { destination, value }
                | DivRegImm8 { destination, value }
                | IdivRegImm8 { destination, value }
                | AndRegImm8 { destination, value }
                | OrRegImm8 { destination, value }
                | XorRegImm8 { destination, value } => {
                    self.assemble_mov_reg_imm8(*destination, *value, &mut mc)
                }

                MovRegImm16 { destination, value }
                | AddRegImm16 { destination, value }
                | SubRegImm16 { destination, value }
                | MulRegImm16 { destination, value }
                | DivRegImm16 { destination, value }
                | IdivRegImm16 { destination, value }
                | AndRegImm16 { destination, value }
                | OrRegImm16 { destination, value }
                | XorRegImm16 { destination, value } => {
                    self.assemble_mov_reg_imm16(*destination, *value, &mut mc)
                }

                MovRegImm32 { destination, value }
                | AddRegImm32 { destination, value }
                | SubRegImm32 { destination, value }
                | MulRegImm32 { destination, value }
                | DivRegImm32 { destination, value }
                | IdivRegImm32 { destination, value }
                | AndRegImm32 { destination, value }
                | OrRegImm32 { destination, value }
                | XorRegImm32 { destination, value } => {
                    self.assemble_mov_reg_imm32(*destination, *value, &mut mc)
                }

                MovRegImm64 { destination, value }
                | AddRegImm64 { destination, value }
                | SubRegImm64 { destination, value }
                | MulRegImm64 { destination, value }
                | DivRegImm64 { destination, value }
                | IdivRegImm64 { destination, value }
                | AndRegImm64 { destination, value }
                | OrRegImm64 { destination, value }
                | XorRegImm64 { destination, value } => {
                    self.assemble_mov_reg_imm64(*destination, *value, &mut mc)
                }

                Instruction::Int { code } => self.assemble_int(*code, &mut mc),
            }
        }

        mc
    }

    fn parse_source(&mut self, source: String) -> bool {
        let source_code = SourceCode::new(source);

        let mut token_iter = source_code.tokens();
        let mut success = true;

        loop {
            let token = token_iter.next();
            if let Ok(token) = &token {
                match token {
                    Some(token) => {
                        if let Err(e) = self.parse_token(token, &mut token_iter) {
                            println!("Error: {e}");
                            success = false;
                            token_iter.skip_line();
                        }
                    }
                    // Stop iterating when iterator reaches None
                    None => break,
                }
            } else if let Err(e) = &token {
                println!("Error: {e}");
                success = false;
                token_iter.skip_line();
            }
        }

        success
    }

    fn parse_token(&mut self, token: &Token, tokens: &mut TokenIter) -> Result<()> {
        debug!("Parsing token: {token:?}");
        match token {
            Token::Mnemonic(instruction) => self.parse_instruction(instruction, tokens),
            Token::Newline => Ok(()),
            other => Err(anyhow!("Unknown token {other:?}")),
        }
    }

    fn parse_instruction(&mut self, instruction: &Mnemonic, tokens: &mut TokenIter) -> Result<()> {
        debug!("Parsing instruction: {instruction:?}");
        match instruction {
            Mnemonic::Mov => self.parse_mov(tokens),
            Mnemonic::Add => self.parse_add(tokens),
            Mnemonic::Sub => self.parse_sub(tokens),
            Mnemonic::Mul => self.parse_mul(tokens),
            Mnemonic::Div => self.parse_div(tokens),
            Mnemonic::Idiv => self.parse_idiv(tokens),
            Mnemonic::And => self.parse_and(tokens),
            Mnemonic::Or => self.parse_or(tokens),
            Mnemonic::Xor => self.parse_xor(tokens),
            Mnemonic::Int => self.parse_int(tokens),
        }
    }
}
