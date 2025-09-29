use anyhow::{anyhow, bail, Context, Result};
use std::{
    fmt::Display,
    num::{IntErrorKind, ParseIntError},
};
use strum::{AsRefStr, IntoStaticStr};

use super::assembler_source::Lexer;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntoStaticStr, AsRefStr)]
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

/// There are 32 general purpose registers.
/// So the max value stored inside the enum is 31
#[derive(Debug, Clone, Copy)]
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

impl AsRef<str> for Register {
    fn as_ref(&self) -> &str {
        self.clone().into()
    }
}

impl Into<&'static str> for Register {
    fn into(self) -> &'static str {
        let string = match self {
            Self::R(num) => match num {
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
                16 => "r16",
                17 => "r17",
                18 => "r18",
                19 => "r19",
                20 => "r20",
                21 => "r21",
                22 => "r22",
                23 => "r23",
                24 => "r24",
                25 => "r25",
                26 => "r26",
                27 => "r27",
                28 => "r28",
                29 => "r29",
                30 => "r30",
                31 => "r31",
                _ => unimplemented!(),
            },

            Self::W(num) => match num {
                0 => "w0",
                1 => "w1",
                2 => "w2",
                3 => "w3",
                4 => "w4",
                5 => "w5",
                6 => "w6",
                7 => "w7",
                8 => "w8",
                9 => "w9",
                10 => "w10",
                11 => "w11",
                12 => "w12",
                13 => "w13",
                14 => "w14",
                15 => "w15",
                16 => "w16",
                17 => "w17",
                18 => "w18",
                19 => "w19",
                20 => "w20",
                21 => "w21",
                22 => "w22",
                23 => "w23",
                24 => "w24",
                25 => "w25",
                26 => "w26",
                27 => "w27",
                28 => "w28",
                29 => "w29",
                30 => "w30",
                31 => "w31",
                _ => unimplemented!(),
            },

            Self::S(num) => match num {
                0 => "s0",
                1 => "s1",
                2 => "s2",
                3 => "s3",
                4 => "s4",
                5 => "s5",
                6 => "s6",
                7 => "s7",
                8 => "s8",
                9 => "s9",
                10 => "s10",
                11 => "s11",
                12 => "s12",
                13 => "s13",
                14 => "s14",
                15 => "s15",
                16 => "s16",
                17 => "s17",
                18 => "s18",
                19 => "s19",
                20 => "s20",
                21 => "s21",
                22 => "s22",
                23 => "s23",
                24 => "s24",
                25 => "s25",
                26 => "s26",
                27 => "s27",
                28 => "s28",
                29 => "s29",
                30 => "s30",
                31 => "s31",
                _ => unimplemented!(),
            },

            Self::B(num) => match num {
                0 => "b0",
                1 => "b1",
                2 => "b2",
                3 => "b3",
                4 => "b4",
                5 => "b5",
                6 => "b6",
                7 => "b7",
                8 => "b8",
                9 => "b9",
                10 => "b10",
                11 => "b11",
                12 => "b12",
                13 => "b13",
                14 => "b14",
                15 => "b15",
                16 => "b16",
                17 => "b17",
                18 => "b18",
                19 => "b19",
                20 => "b20",
                21 => "b21",
                22 => "b22",
                23 => "b23",
                24 => "b24",
                25 => "b25",
                26 => "b26",
                27 => "b27",
                28 => "b28",
                29 => "b29",
                30 => "b30",
                31 => "b31",
                _ => unimplemented!(),
            },
        };

        string
    }
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

#[derive(Debug, Clone, Copy, AsRefStr, PartialEq, Eq)]
pub enum Keyword {
    Const,
}

#[derive(Debug)]
pub enum Token {
    Mnemonic(Mnemonic),
    Register(Register),
    Identifier(String),
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
    Newline,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Self::Mnemonic(instr) => String::from(instr.as_ref()),
            Self::Register(register) => String::from(register.as_ref()),
            Self::Identifier(id) => id.clone(),
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
        match self {
            Self::Comma => true,
            _ => false,
        }
    }

    pub fn is_equal_sign(&self) -> bool {
        match self {
            Self::Equal => true,
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

        // Prevent things like r01 from being valid registers
        if reg_id < 10 && token.len() != 2 {
            return None;
        } else if reg_id >= 10 && token.len() != 3 {
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
