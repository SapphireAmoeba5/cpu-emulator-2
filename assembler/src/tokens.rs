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

impl PartialEq for Register {
    fn eq(&self, other: &Self) -> bool {
        if self.is_invalid() || other.is_invalid() {
            false
        } else {
            self.0 == other.0
        }
    }
}

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
    /// The index used to specify that the register is the instruction pointer
    const IP_INDEX: u8 = 127;
    /// A register value that is used to mean there is no register
    const INVALID_INDEX: u8 = 254;
    pub const INVALID_REGISTER: Register = Register(Self::INVALID_INDEX);

    /// If the register is a general purpose register
    pub fn is_gp(&self) -> bool {
        self.0 < Self::NUM_GP_REGISTERS
    }

    pub fn is_sp(&self) -> bool {
        self.0 == Register::SP_INDEX
    }

    pub fn is_ip(&self) -> bool {
        self.0 == Register::IP_INDEX
    }

    pub fn is_valid(&self) -> bool {
        self.0 != Self::INVALID_INDEX
    }

    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }
    // Constructs a new general purpose register (r0 -> r15)
    pub fn new_gp(index: u8) -> Self {
        assert!(index < Self::NUM_GP_REGISTERS);

        Self(index)
    }

    pub fn new_sp() -> Self {
        Self(Self::SP_INDEX)
    }

    pub fn new_ip() -> Self {
        Self(Self::IP_INDEX)
    }

    pub fn none() -> Self {
        Self::INVALID_REGISTER
    }
    /// Returns the index of the register if its a GP
    pub fn get_gp(&self) -> Option<u8> {
        if self.is_gp() { Some(self.0) } else { None }
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
    LSqrBrace,
    RSqrBrace,
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
            Self::LSqrBrace => "[".to_string(),
            Self::RSqrBrace => "]".to_string(),
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
            "[" => Some(Token::LSqrBrace),
            "]" => Some(Token::RSqrBrace),
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
            "halt" => Some(Mnemonic::Halt),
            "mov" => Some(Mnemonic::Mov),
            "mov.u8" | "movb" => Some(Mnemonic::MovU8),
            "mov.u16" | "movq" => Some(Mnemonic::MovU16),
            "mov.u32" | "movh" => Some(Mnemonic::MovU32),
            "mov.u64" => Some(Mnemonic::MovU64),
            "str" | "str.u64" => Some(Mnemonic::Str),
            "str.u8" | "strb" => Some(Mnemonic::StrU8),
            "str.u16" | "strq" => Some(Mnemonic::StrU16),
            "str.u32" | "strh" => Some(Mnemonic::StrU32),
            "lea" | "lea.u8" | "lea.u16" | "lea.u32" | "lea.u64" => Some(Mnemonic::Lea),
            "add" => Some(Mnemonic::Add),
            "add.u8" | "addb" => Some(Mnemonic::AddU8),
            "add.u16" | "addq" => Some(Mnemonic::AddU16),
            "add.u32" | "addh" => Some(Mnemonic::AddU32),
            "add.u64" => Some(Mnemonic::AddU64),
            "sub" => Some(Mnemonic::Sub),
            "sub.u8" | "subb" => Some(Mnemonic::SubU8),
            "sub.u16" | "subq" => Some(Mnemonic::SubU16),
            "sub.u32" | "subh" => Some(Mnemonic::SubU32),
            "sub.u64" => Some(Mnemonic::SubU64),
            "mul" => Some(Mnemonic::Mul),
            "mul.u8" | "mulb" => Some(Mnemonic::MulU8),
            "mul.u16" | "mulq" => Some(Mnemonic::MulU16),
            "mul.u32" | "mulh" => Some(Mnemonic::MulU32),
            "mul.u64" => Some(Mnemonic::MulU64),
            "div" => Some(Mnemonic::Div),
            "div.u8" | "divb" => Some(Mnemonic::DivU8),
            "div.u16" | "divq" => Some(Mnemonic::DivU16),
            "div.u32" | "divh" => Some(Mnemonic::DivU32),
            "div.u64" => Some(Mnemonic::DivU64),
            "idiv" => Some(Mnemonic::Idiv),
            "idiv.u8" | "idivb" => Some(Mnemonic::IdivU8),
            "idiv.u16" | "idivq" => Some(Mnemonic::IdivU16),
            "idiv.u32" | "idivh" => Some(Mnemonic::IdivU32),
            "idiv.u64" => Some(Mnemonic::IdivU64),
            "and" => Some(Mnemonic::And),
            "and.u8" | "andb" => Some(Mnemonic::AndU8),
            "and.u16" | "andq" => Some(Mnemonic::AndU16),
            "and.u32" | "andh" => Some(Mnemonic::AndU32),
            "and.u64" => Some(Mnemonic::AndU64),
            "or" => Some(Mnemonic::Or),
            "or.u8" | "orb" => Some(Mnemonic::OrU8),
            "or.u16" | "orq" => Some(Mnemonic::OrU16),
            "or.u32" | "orh" => Some(Mnemonic::OrU32),
            "or.u64" => Some(Mnemonic::OrU64),
            "xor" => Some(Mnemonic::Xor),
            "xor.u8" | "xorb" => Some(Mnemonic::XorU8),
            "xor.u16" | "xorq" => Some(Mnemonic::XorU16),
            "xor.u32" | "xorh" => Some(Mnemonic::XorU32),
            "xor.u64" => Some(Mnemonic::XorU64),
            "cmp" => Some(Mnemonic::Cmp),
            "cmp.u8" | "cmpb" => Some(Mnemonic::CmpU8),
            "cmp.u16" | "cmpq" => Some(Mnemonic::CmpU16),
            "cmp.u32" | "cmph" => Some(Mnemonic::CmpU32),
            "cmp.u64" => Some(Mnemonic::CmpU64),
            "test" => Some(Mnemonic::Test),
            "test.u8" | "testb" => Some(Mnemonic::TestU8),
            "test.u16" | "testq" => Some(Mnemonic::TestU16),
            "test.u32" | "testh" => Some(Mnemonic::TestU32),
            "test.u64" => Some(Mnemonic::TestU64),

            "push" => Some(Mnemonic::Push),
            "pop" => Some(Mnemonic::Pop),

            "jmp" => Some(Mnemonic::Jmp),

            "jz" | "je" => Some(Mnemonic::Jz),
            "jnz" | "jne" => Some(Mnemonic::Jnz),
            "jc" | "jb" | "jnae" => Some(Mnemonic::Jc),
            "jnc" | "jae" | "jnb" => Some(Mnemonic::Jnc),

            "jo" => Some(Mnemonic::Jo),
            "jno" => Some(Mnemonic::Jno),

            "js" => Some(Mnemonic::Js),
            "jns" => Some(Mnemonic::Jns),

            "ja" | "jnbe" => Some(Mnemonic::Ja),
            "jbe" | "jna" => Some(Mnemonic::Jbe),

            "jg" | "jnle" => Some(Mnemonic::Jg),
            "jle" | "jng" => Some(Mnemonic::Jle),

            "jge" | "jnl" => Some(Mnemonic::Jge),
            "jl" | "jnge" => Some(Mnemonic::Jl),
            
            "call" => Some(Mnemonic::Call),
            "ret" => Some(Mnemonic::Ret),

            "rdt" => Some(Mnemonic::Rdt),

            "stsp" => Some(Mnemonic::Stsp),
            "rdsp" => Some(Mnemonic::Rdsp),

            "rdtf" => Some(Mnemonic::Rdtf),
            "int" => Some(Mnemonic::Int),
            _ => None,
        }
    }

    fn register(token: &str) -> Option<Register> {
        if token == "sp" {
            return Some(Register::new_sp());
        } else if token == "ip" {
            return Some(Register::new_ip());
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
