mod machine_code;
mod parse;
mod symbol_table;

use std::collections::HashMap;
use std::fmt::Display;

use anyhow::anyhow;
use spdlog::debug;

use crate::expression::calculate_tree_value;
use crate::expression::parse_expr;
use crate::opcode::encoding;
use crate::opcode::EncodingInfo;
use crate::opcode::InstructionInfo;
use crate::opcode::OperandType;

use super::assembler_source::*;
use super::tokens::*;

use anyhow::{Context, Result};

#[derive(Debug, Copy, Clone)]
pub enum Operand {
    None,
    Register(Register),
    Constant(u64),
}

impl Operand {
    pub fn operand_type(&self) -> OperandType {
        match self {
            Operand::Register(reg) => match reg {
                Register::R(_) => OperandType::Reg64,
                Register::W(_) => OperandType::Reg32,
                Register::S(_) => OperandType::Reg16,
                Register::B(_) => OperandType::Reg8,
            },
            Operand::Constant(_) => OperandType::Constant,
            Operand::None => OperandType::None,
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    instruction_info: InstructionInfo,
    encoding_info: EncodingInfo,
    operands: [Operand; 2],
}

pub struct Assembler {
    // The instructions
    instructions: Vec<Instruction>,
    // symbols: SymbolTable,
}

impl Assembler {
    pub fn assemble(source: String) -> Result<Self> {
        let mut assembler = Assembler {
            instructions: Vec::new(),
        };

        let result = assembler.parse_source(source);

        // Pretty bad way of doing it but oh well
        if result {
            for instruction in assembler.instructions.iter() {
                println!("{:?}", instruction.operands);
            }
            Ok(assembler)
        } else {
            Err(anyhow!("Failed to assemble source"))
        }
    }

    fn parse_source(&mut self, source: String) -> bool {
        let source_code = SourceCode::new(source);

        let mut token_iter = source_code.tokens();
        let mut success = true;

        loop {
            let current_line = token_iter.line();
            let token = token_iter.next();
            if let Ok(token) = &token {
                match token {
                    Some(token) => {
                        if let Err(e) = self.parse_token(token, &mut token_iter) {
                            println!("Error line {current_line}: {e}");
                            success = false;
                            token_iter.skip_line();
                        }
                    }
                    // Stop iterating when iterator reaches None
                    None => break,
                }
            } else if let Err(e) = &token {
                println!("Error line {current_line}: {e}");
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
            Token::Keyword(keyword) => self.parse_keyword(keyword, tokens),
            Token::Newline => Ok(()),
            other => Err(anyhow!("Unknown token {other:?}")),
        }
    }

    fn parse_instruction(&mut self, instruction: &Mnemonic, tokens: &mut TokenIter) -> Result<()> {
        debug!("Parsing instruction: {instruction:?}");
        let operands = self.parse_operands(tokens)?;

        let mut types: [OperandType; 2] = [OperandType::None; 2];

        for (a, b) in operands.iter().zip(types.iter_mut()) {
            *b = a.operand_type();
        }

        let instruction_info = InstructionInfo::new(*instruction, types);
        let encoding_info = encoding(&instruction_info).with_context(|| "Invalid instruction")?;

        let instruction = Instruction {
            instruction_info,
            encoding_info,
            operands,
        };

        self.instructions.push(instruction);

        Ok(())
    }

    fn parse_operands(&mut self, tokens: &mut TokenIter) -> Result<[Operand; 2]> {
        let mut operands = [Operand::None; 2];
        let mut cur = 0;

        while let Some(next) = tokens.peek()? {
            let operand = match next {
                Token::Register(reg) => {
                    let _ = tokens.next();
                    Operand::Register(reg)
                }
                Token::Number(_) | Token::Identifier(_) => {
                    Operand::Constant(calculate_tree_value(&*parse_expr(tokens)?))
                }
                _ => return Err(anyhow!("Invalid operand {}", next.to_string())),
            };

            // Used for formatting an error message without annoying the borrow checker
            let len = operands.len();

            *operands
                .get_mut(cur)
                .with_context(|| format!("Too many operands (max operand count is {len})"))? =
                operand;
            cur += 1;

            match tokens.next()? {
                Some(Token::Comma) => {}
                Some(Token::Newline) => break,
                None => break,
                Some(operand) => return Err(anyhow!("Invalid operand {}", operand.to_string())),
            }
        }

        Ok(operands)
    }

    fn parse_keyword(&mut self, keyword: &Keyword, tokens: &mut TokenIter) -> Result<()> {
        match keyword {
            Keyword::Const => self.parse_const(tokens),
        }
    }

    fn parse_const(&mut self, tokens: &mut TokenIter) -> Result<()> {
        unimplemented!("Unimplemented")
        // let name = tokens.next()?.with_context(|| "Expected variable name")?.to_identifier().with_context(|| format!("Expected an identifier"))?;
        // tokens.is_equal_sign()?;
        // let value = tokens.next()?.with_context(|| "Expected variable value")?;
        // tokens.newline_or_eof()?;

        // match value {
        //     Token::Number(number) => self.variables.set(name, number),
        //     Token::Identifier(id) => self.variables.set(name, self.variables.get(&id)?),
        //     _ => Err(anyhow!("Expected number or identifier as value to constant")),
        // }
    }
}
