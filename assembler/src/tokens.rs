use crate::{
    instruction::{Mnemonic, Operand},
    opcode::OperandFlags,
};
use anyhow::{Context, Result, anyhow, bail};
use std::{
    fmt::Display,
    num::{IntErrorKind, ParseIntError},
};
use strum::{AsRefStr, IntoStaticStr};

use super::assembler_source::Lexer;

/// There are 16 general purpose registers.
/// Garunteed for the register index to be between 0..=15
#[derive(Debug, Clone, Copy)]
pub struct Register(u8);

impl AsRef<str> for Register {
    fn as_ref(&self) -> &str {
        self.clone().into()
    }
}

impl Into<&'static str> for Register {
    fn into(self) -> &'static str {
        let string = match self.0 {
            0 => "r0",
            1 => "r1",
            2 => "r2",
            3 => "r3",
            4 => "r4",
            5 => "r5",
            6 => "r6",
            7 => "r7",
            8 => "r8",
            9 => "r9",
            10 => "r10",
            11 => "r11",
            12 => "r12",
            13 => "r13",
            14 => "r14",
            15 => "r15",
            Self::SP_INDEX => "sp",
            _ => panic!("Can't have more than 32 registers"),
        };

        string
    }
}

impl Register {
    /// The number of indices starting from 0 up to and including `NUM_GP_REGISTERS - 1`
    const NUM_GP_REGISTERS: u8 = 16;
    /// The index used to specify that the register is the stack pointer
    const SP_INDEX: u8 = 255;
    /// A register value used to mean there is no register
    const INVALID_REGISTER: u8 = 254;
    /// If the register is a general purpose register
    pub fn is_gp(&self) -> bool {
        self.0 < Self::NUM_GP_REGISTERS
    }
    // Constructs a new general purpose register (r0 -> r15)
    pub fn new_gp(index: u8) -> Self {
        assert!(index < 16);

        Self(index)
    }

    pub fn new_sp() -> Self {
        Self(Self::SP_INDEX)
    }

    pub fn none() -> Self {
        Self(Self::INVALID_REGISTER)
    }
    /// Returns the index of the register if its a GP
    pub fn get_gp(&self) -> Option<u8> {
        if self.is_gp() {
            Some(self.0)
        } else {
            None
        }
    }

    /// Returns the register type as OperandFlags
    pub fn get_operand_flag(&self) -> OperandFlags {
        let mut flags = OperandFlags::REG;

        if self.is_gp() {
            flags |= OperandFlags::GP_REG;
        }

        flags
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index = self.0;
        write!(f, "r{index}")
    }
}

#[derive(Debug, Clone, Copy, IntoStaticStr, AsRefStr)]
pub enum Directive {
    Section,
    Align,
    Skip,
    Global,
    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone, Copy, AsRefStr, PartialEq, Eq)]
pub enum Keyword {
    Const,
}

#[derive(Debug)]
pub enum Token {
    Mnemonic(Mnemonic),
    Register(Register),
    Identifier(String),
    Directive(Directive),
    Keyword(Keyword),
    Number(u64),
    Equal,
    Comma,
    LBrace,
    RBrace,
    Plus,
    Sub,
    Mul,
    Div,
    Caret,
    Ampersand,
    /// The @ symbol
    AtSign,
    Colon,
    Dollar,
    Newline,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Self::Mnemonic(instr) => String::from(instr.as_ref()),
            Self::Register(register) => String::from(register.as_ref()),
            Self::Identifier(id) => id.clone(),
            Self::Directive(dir) => dir.as_ref().to_string(),
            Self::Keyword(keyword) => keyword.as_ref().to_string(),
            Self::Number(num) => num.to_string(),
            Self::Equal => "=".to_string(),
            Self::Comma => ",".to_string(),
            Self::LBrace => "(".to_string(),
            Self::RBrace => ")".to_string(),
            Self::Plus => "+".to_string(),
            Self::Sub => "-".to_string(),
            Self::Mul => "*".to_string(),
            Self::Div => "/".to_string(),
            Self::Caret => "^".to_string(),
            Self::Ampersand => "&".to_string(),
            Self::AtSign => "@".to_string(),
            Self::Colon => ":".to_string(),
            Self::Dollar => "$".to_string(),
            Self::Newline => "Newline".to_string(),
        }
    }
}

impl Token {
    pub fn to_identifier(self) -> Option<String> {
        match self {
            Self::Identifier(id) => Some(id),
            _ => None,
        }
    }
}

impl Token {
    pub fn is_comma(&self) -> bool {
        matches!(self, Self::Comma)
    }

    pub fn is_equal_sign(&self) -> bool {
        matches!(self, Self::Equal)
    }

    pub fn is_newline(&self) -> bool {
        matches!(self, Self::Newline)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
}

#[derive(Debug)]
pub struct TokenIter<'a> {
    lexer: Lexer<'a>,
}

impl<'a> TokenIter<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self { lexer }
    }

    /// Skips all tokens until the next newline or None
    pub fn skip_line(&mut self) {
        while let Some(next) = self.lexer.next() {
            if next == "\n" {
                break;
            }
        }
        // Loop until the next token is Ok(None)
        // loop {
        //     if let Ok(next) = self.next() {
        //         if let Some(next) = next {
        //             if next.is_newline() {
        //                 break;
        //             }
        //         } else {
        //             break;
        //         }
        //     }
        // }
    }
    /// Returns Ok(()) if the next token is a newline or None
    pub fn newline_or_eof(&mut self) -> Result<()> {
        self.next()?.map_or(Ok(()), |token| {
            if token.is_newline() {
                Ok(())
            } else {
                Err(anyhow!("Expected newline or end of file"))
            }
        })
    }

    pub fn is_comma(&mut self) -> Result<()> {
        self.next()?.map_or(Ok(()), |token| {
            if token.is_comma() {
                Ok(())
            } else {
                Err(anyhow!("Expected comma"))
            }
        })
    }

    pub fn is_equal_sign(&mut self) -> Result<()> {
        self.next()?.map_or(Ok(()), |token| {
            if token.is_equal_sign() {
                Ok(())
            } else {
                Err(anyhow!("Expected equal sign"))
            }
        })
    }

    pub fn next(&mut self) -> Result<Option<Token>> {
        if let Some(token) = self.lexer.next() {
            Self::parse_token(token)
        } else {
            Ok(None)
        }
    }

    pub fn peek(&mut self) -> Result<Option<Token>> {
        if let Some(token) = self.lexer.peek() {
            Self::parse_token(token)
        } else {
            Ok(None)
        }
    }

    /// Returns the line of the last token gotten from next()
    pub fn line(&self) -> usize {
        self.lexer.line()
    }

    fn parse_token(token: &str) -> Result<Option<Token>> {
        let token = if let Some(instruction) = Self::instruction(token) {
            Token::Mnemonic(instruction)
        } else if let Some(register) = Self::register(token) {
            Token::Register(register)
        } else if let Some(keyword) = Self::keyword(token) {
            Token::Keyword(keyword)
        } else if let Some(token) = Self::directive(token) {
            Token::Directive(token)
        } else if let Some(token) = Self::special_character(token) {
            token
        } else if token == "\n" {
            Token::Newline
        } else if let Some(number) = Self::number(token)? {
            Token::Number(number)
        } else {
            Token::Identifier(token.to_string())
        };

        Ok(Some(token))
    }

    fn special_character(token: &str) -> Option<Token> {
        match token {
            "=" => Some(Token::Equal),
            "," => Some(Token::Comma),
            "(" => Some(Token::LBrace),
            ")" => Some(Token::RBrace),
            "+" => Some(Token::Plus),
            "-" => Some(Token::Sub),
            "*" => Some(Token::Mul),
            "/" => Some(Token::Div),
            "^" => Some(Token::Caret),
            "&" => Some(Token::Ampersand),
            "@" => Some(Token::AtSign),
            ":" => Some(Token::Colon),
            "$" => Some(Token::Dollar),
            _ => None,
        }
    }

    fn directive(token: &str) -> Option<Directive> {
        match token {
            ".section" => Some(Directive::Section),
            ".align" => Some(Directive::Align),
            ".skip" => Some(Directive::Skip),
            ".global" => Some(Directive::Global),
            ".u8" => Some(Directive::U8),
            ".u16" => Some(Directive::U16),
            ".u32" => Some(Directive::U32),
            ".u64" => Some(Directive::U64),
            _ => None,
        }
    }

    fn keyword(token: &str) -> Option<Keyword> {
        match token {
            "const" => Some(Keyword::Const),
            _ => None,
        }
    }

    /// Returns Token::Instruction if the token is an instruction
    fn instruction(token: &str) -> Option<Mnemonic> {
        match token.to_lowercase().as_str() {
            "mov" => Some(Mnemonic::Mov),
            "str" => Some(Mnemonic::Str),
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
        if token == "sp" {
            return Some(Register::new_sp());
        }

        // The type of register, whether its 8, 16, 32, or 64 bits
        let reg_type = token.chars().next().unwrap();

        let reg_id = &token[1..];

        let reg_id = if let Ok(reg_id) = reg_id.parse::<u8>() {
            reg_id
        } else {
            return None;
        };

        // Only 16 registers are supported
        if reg_id >= 16 {
            return None;
        }

        // Prevent things like r01 from being valid registers
        if reg_id < 10 && token.len() != 2 {
            return None;
        } else if reg_id >= 10 && token.len() != 3 {
            return None;
        }

        match reg_type.to_ascii_lowercase() {
            'r' => Some(Register::new_gp(reg_id)),
            _ => None,
        }
    }

    /// Tries to parse a number
    fn number(token: &str) -> Result<Option<u64>> {
        if !token.starts_with(|ch: char| ch.is_ascii_digit()) {
            Ok(None)
        } else if token.starts_with("0x") && token.len() >= 3 {
            match u64::from_str_radix(&token[2..], 16) {
                Ok(num) => Ok(Some(num)),
                Err(e) => match e.kind() {
                    IntErrorKind::PosOverflow => Err(anyhow!("Number {token} is too large")),
                    IntErrorKind::NegOverflow => Err(anyhow!("Number {token} is too small")),
                    IntErrorKind::InvalidDigit => {
                        Err(anyhow!("Number {token} contains an invalid digit"))
                    }
                    _ => Err(anyhow!("Invalid number {token}")),
                },
            }
        } else {
            match token.parse::<i64>().map(|value| value as u64) {
                Ok(num) => Ok(Some(num)),
                Err(_) => Err(anyhow!("Invalid number {token}")),
            }
        }
    }
}
