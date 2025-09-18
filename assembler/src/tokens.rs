use std::{
    error::Error,
    fmt::Display,
    num::{IntErrorKind, ParseIntError},
    ops::Index,
};

use super::assembler_source::Lexer;

#[derive(Debug)]
pub enum Mnemonic {
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    And,
    Or,
    Xor,
    Int,
}

#[derive(Debug, Clone, Copy)]
/// There are 32 general purpose registers.
/// So the max value stored inside the enum is 31
pub enum Register {
    /// 64bit registers
    R(u8),
    /// 32bit registers
    W(u8),
    /// 16bit registers
    S(u8),
    /// 8bit registers
    B(u8),
}

impl Register {
    // Returns the number of bytes stored in the register
    pub fn size(&self) -> usize {
        use Register::*;
        match self {
            R(_) => 8,
            W(_) => 4,
            S(_) => 2,
            B(_) => 1,
        }
    }

    /// Returns the register index
    pub fn index(&self) -> u8 {
        use Register::*;
        match self {
            R(i) | W(i) | S(i) | B(i) => *i,
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Register::*;
        let (reg_type, index) = match self {
            R(i) => ('r', i),
            W(i) => ('w', i),
            S(i) => ('s', i),
            B(i) => ('b', i),
        };
        write!(f, "{reg_type}{index}")
    }
}

#[derive(Debug)]
pub enum Token {
    Mnemonic(Mnemonic),
    Register(Register),
    Identifier(String),
    Number(u64),
    Comma,
    Newline,
}

impl Token {
    pub fn is_comma(&self) -> bool {
        match self {
            Self::Comma => true,
            _ => false,
        }
    }

    pub fn is_newline(&self) -> bool {
        match self {
            Self::Newline => true,
            _ => false,
        }
    }
    
    pub fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Tokens {
    tokens: Vec<Token>,
}

impl Tokens {
    pub fn tokenize(lexer: &mut Lexer) -> Self {
        let mut tokens = Vec::new();
        for token in lexer {
            if let Some(instruction) = Self::instruction(token) {
                tokens.push(Token::Mnemonic(instruction));
            } else if let Some(register) = Self::register(token) {
                tokens.push(Token::Register(register));
            } else if token == "," {
                tokens.push(Token::Comma);
            } else if token == "\n" {
                tokens.push(Token::Newline)
            } else if let Some(number) = Self::number(token) {
                tokens.push(Token::Number(number));
            } else {
                tokens.push(Token::Identifier(token.to_string()));
            }
        }

        Self { tokens }
    }

    pub fn iter(&self) -> TokenIter {
        TokenIter {
            tokens: &self.tokens,
            current: 0,
        }
    }

    /// Returns Token::Instruction if the token is an instruction
    fn instruction(token: &str) -> Option<Mnemonic> {
        match token.to_lowercase().as_str() {
            "mov" => Some(Mnemonic::Mov),
            "add" => Some(Mnemonic::Add),
            "sub" => Some(Mnemonic::Sub),
            "mul" => Some(Mnemonic::Mul),
            "div" => Some(Mnemonic::Div),
            "idiv" => Some(Mnemonic::Idiv),
            "and" => Some(Mnemonic::And),
            "or" => Some(Mnemonic::Or),
            "xor" => Some(Mnemonic::Xor),
            "int" => Some(Mnemonic::Int),
            _ => None,
        }
    }

    fn register(token: &str) -> Option<Register> {
        // The type of register, whether its 8, 16, 32, or 64 bits
        let reg_type = token.chars().next().unwrap();

        let reg_id = &token[1..];

        let reg_id = if let Ok(reg_id) = reg_id.parse::<u8>() {
            reg_id
        } else {
            return None;
        };

        // Only 32 registers are supported
        if reg_id >= 32 {
            return None;
        }

        match reg_type.to_ascii_lowercase() {
            'r' => Some(Register::R(reg_id)),
            'w' => Some(Register::W(reg_id)),
            's' => Some(Register::S(reg_id)),
            'b' => Some(Register::B(reg_id)),
            _ => None,
        }
    }

    /// Tries to parse a number
    fn number(token: &str) -> Option<u64> {
        token.parse::<i64>().map(|value| value as u64).ok()
    }
}

pub struct TokenIter<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> TokenIter<'a> {
    /// Returns Ok(()) if the next token is a newline or None
    pub fn newline_or_eof(&mut self) -> Result<(), ()> {
        self.next().map_or(
            Ok(()),
            |token| if token.is_newline() { Ok(()) } else { Err(()) },
        )
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a Token;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current;
        self.current += 1;
        self.tokens.get(index)
    }
}
