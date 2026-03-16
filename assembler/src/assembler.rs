mod directive;
pub(super) mod emit;
mod parse;
pub mod symbol_table;
use bitflags::Flag;
use itertools::izip;

use std::collections::HashMap;
use std::f32::consts::E;
use std::iter::Peekable;
use std::ops::{Deref, DerefMut};
use std::panic::resume_unwind;
use std::{mem, usize};

use anyhow::{anyhow, bail};
use spdlog::debug;

use crate::assembler::symbol_table::{SymbolTable, Type};
use crate::expression::{BinaryOp, Mode, Node, parse_expr};
use crate::instruction::Mnemonic;
use crate::opcode::{
    EncodingFlags, InstEncoding, MAX_OPERANDS, OperandFlags, Relocation, get_encodings,
};
use crate::operand;
use crate::section::{self, Section, SectionMap};
pub use emit::calculate_disp32_offset;

use super::lexer::*;
use super::tokens::*;

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct AssemblerToken {
    pub token: Token,
    pub line: usize,
}

pub trait AsmTokenIter<'a>: Iterator<Item = &'a AssemblerToken> {
    fn is_equal_sign(&mut self) -> bool {
        self.next()
            .map(|a| matches!(a.token, Token::Equal))
            .unwrap_or(false)
    }

    fn is_newline_or_eof(&mut self) -> bool {
        self.next()
            .map(|a| matches!(a.token, Token::Newline))
            .unwrap_or(true)
    }
}
impl<'a, T: Iterator<Item = &'a AssemblerToken>> AsmTokenIter<'a> for T {}

fn get_operand_from_expr_result(is_label: bool, _relocatable: bool) -> OperandFlags {
    let operand = if is_label {
        operand!(DISP)
    } else {
        operand!(IMM | ADDR64 | DISP)
    };

    operand
}

#[derive(Debug, Copy, Clone)]
pub enum Operand {
    None,
    Constant(u64),
    Register(Register),
}

/// The memory address is calculated as Base + (Index * scale) + displacement
#[derive(Debug, Clone, Copy)]
pub struct MemoryIndex {
    /// The constant offset for the memory index
    pub disp: u64,
    /// The base register to use in the address calculation
    pub base: Register,
    /// The index register to use in the address calculation. If a register is supplied here, then
    /// `base` must also be valid register too
    pub index: Register,
    /// Either 1, 2, 4, or 8. Multiplied with the index register
    pub scale: u64,
    /// If the displacement value is supposed to be a label
    pub is_label: bool,
}

impl MemoryIndex {
    /// Returns true if this memory index expression is just a register
    pub fn is_register(&self) -> bool {
        self.disp == 0
            && self.base.is_valid()
            && self.index.is_invalid()
            && self.scale == 1
            && self.is_label == false
    }

    /// Returns true if this memory index expression has a register component to it
    pub fn has_register(&self) -> bool {
        self.base.is_valid() || self.index.is_valid()
    }

    /// Returns tru of this memory index expression is just a number
    pub fn is_number(&self) -> bool {
        self.base.is_invalid() && self.index.is_invalid() && self.scale == 1
    }
}

impl MemoryIndex {
    pub fn disp(disp: u64) -> Self {
        Self {
            disp,
            base: Register::none(),
            index: Register::none(),
            scale: 1,
            is_label: false,
        }
    }

    pub fn label(value: u64) -> Self {
        Self {
            disp: value,
            base: Register::none(),
            index: Register::none(),
            scale: 1,
            is_label: true,
        }
    }

    pub fn register(register: Register) -> Self {
        Self {
            disp: 0,
            base: register,
            index: Register::none(),
            scale: 1,
            is_label: false,
        }
    }
}

impl Default for MemoryIndex {
    fn default() -> Self {
        Self {
            disp: 0,
            base: Register::none(),
            index: Register::none(),
            scale: 1,
            is_label: false,
        }
    }
}

impl Operand {
    /// Panics if `self` isn't a constant
    #[track_caller]
    pub fn constant(&self) -> u64 {
        match self {
            Self::Constant(num) => *num,
            _ => panic!("Not a constant"),
        }
    }

    /// Panics if `self` isn't a register
    #[track_caller]
    pub fn register(&self) -> Register {
        match self {
            Self::Register(reg) => *reg,
            _ => panic!("Not a register"),
        }
    }
}

#[derive(Debug, Clone)]
struct Instruction {
    encoding: InstEncoding,
    /// In `types`, `operands`, `exprs`, and `reloc` the values up to `< operand_count` should be
    /// populated with proper values based off what should be emitted
    operand_count: usize,
    types: [OperandFlags; MAX_OPERANDS],
    /// The instruction operand's
    operands: [Operand; MAX_OPERANDS],
    /// The expression that produced each operand. Must contain `Some`
    exprs: [Option<Box<Node>>; MAX_OPERANDS],
    /// The memory indexing operations of this instruction
    indexes: [MemoryIndex; MAX_OPERANDS],
    /// Whether a relocation has been requested
    reloc: [bool; MAX_OPERANDS],
    // /// Reloaction per operand
    // reloc: [Relocation; MAX_OPERANDS],
}

#[derive(Debug, Clone)]
pub struct ForwardReferenceEntry {
    pub relocation: Relocation,
    pub section: usize,
    pub offset: usize,
    pub expr: Box<Node>,

    /// The line number the relocation was emitted on
    pub line_number: usize,
}

impl ForwardReferenceEntry {
    pub fn new(
        relocation: Relocation,
        section: usize,
        offset: usize,
        expr: Box<Node>,
        line_number: usize,
    ) -> Self {
        Self {
            relocation,
            section,
            offset,
            expr,
            line_number,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExprResult {
    Constant {
        constant: u64,
        is_label: bool,
        relocation: bool,
    },
    Register(Register),
}

impl ExprResult {
    pub fn new_imm(immediate: u64) -> Self {
        Self::Constant {
            constant: immediate,
            is_label: false,
            relocation: false,
        }
    }

    pub fn new_reloc() -> Self {
        Self::Constant {
            constant: 0,
            is_label: false,
            relocation: true,
        }
    }
}

#[derive(Debug)]
pub struct Assembler {
    pub filename: String,
    pub symbols: SymbolTable,
    pub global_symbols: Vec<String>,

    pub forward_references: Vec<ForwardReferenceEntry>,

    pub sections: SectionMap,

    /// The current line number being parsed
    current_line: usize,
}

impl Assembler {
    const NO_SECTION: usize = usize::MAX;
}

impl Assembler {
    fn evaluate_expression(&self, expr: &Box<Node>, current_section: usize) -> Result<ExprResult> {
        match &**expr {
            Node::Constant(value) => Ok(ExprResult::new_imm(*value)),
            Node::Register(register) => Ok(ExprResult::Register(*register)),
            Node::Identifier(id) => {
                // If the symbol isn't defined
                let Some(symbol) = self.symbols.get_symbol(id) else {
                    return Ok(ExprResult::new_reloc());
                };

                // The symbol is a label
                if let Some(section) = symbol.section_index {
                    if section == current_section {
                        Ok(ExprResult::Constant {
                            constant: symbol.value,
                            is_label: true,
                            relocation: false,
                        })
                    }
                    // The label exists in a different section
                    else {
                        Ok(ExprResult::Constant {
                            constant: 0,
                            is_label: true,
                            relocation: true,
                        })
                    }
                } else {
                    Ok(ExprResult::new_imm(symbol.value))
                }
            }
            Node::Expression(expr) => self.evaluate_expression(expr, current_section),
            Node::BinaryOp { op, left, right } => {
                let left = self.evaluate_expression(left, current_section)?;
                let right = self.evaluate_expression(right, current_section)?;

                let ExprResult::Constant {
                    constant: lhs_constant,
                    is_label: lhs_is_label,
                    relocation: lhs_relocation,
                } = left
                else {
                    bail!("Invalid operation on a register")
                };

                let ExprResult::Constant {
                    constant: rhs_constant,
                    is_label: rhs_is_label,
                    relocation: rhs_relocation,
                } = right
                else {
                    bail!("Invalid operation on a register")
                };

                if *op == BinaryOp::Div && rhs_constant == 0 {
                    bail!("Division by zero");
                }

                let result = op.calculate(lhs_constant, rhs_constant);

                Ok(ExprResult::Constant {
                    constant: result,
                    is_label: lhs_is_label | rhs_is_label,
                    relocation: lhs_relocation | rhs_relocation,
                })
            }

            Node::UnaryOp { op, expr } => {
                let result = self.evaluate_expression(expr, current_section)?;

                let ExprResult::Constant {
                    constant,
                    is_label,
                    relocation,
                } = result
                else {
                    bail!("Invalid operation on a register");
                };

                if is_label {
                    bail!("Cannot perform this operation on a label");
                }

                let constant = op.calculate(constant);

                Ok(ExprResult::Constant {
                    constant,
                    is_label,
                    relocation,
                })
            }
        }
    }

    /// Returns a tuple of the result of the expression, and whether a relocation needs to be
    /// emitted
    fn evaluate_non_operand_expression(&self, expr: &Box<Node>) -> Result<(u64, bool)> {
        let result = self.evaluate_expression(expr, Self::NO_SECTION)?;

        match result {
            ExprResult::Register(_) => bail!("Invalid use of register"),
            ExprResult::Constant {
                constant,
                is_label,
                relocation,
            } => {
                if is_label {
                    bail!("Invalid use of label");
                } else {
                    Ok((constant, relocation))
                }
            }
        }
    }

    /// Does not check if `scale` is valid
    fn evaluate_memory_index(
        &self,
        expr: &Box<Node>,
        current_section: usize,
    ) -> Result<(MemoryIndex, bool)> {
        let (index, relocation) = match &**expr {
            Node::Constant(constant) => (MemoryIndex::disp(*constant), false),
            Node::Register(register) => (MemoryIndex::register(*register), false),
            Node::Identifier(identifier) => {
                let Some(symbol) = self.symbols.get_symbol(&identifier) else {
                    return Ok((MemoryIndex::disp(0), true));
                };

                if let Some(section_index) = symbol.section_index {
                    if current_section == section_index {
                        (MemoryIndex::label(symbol.value), false)
                    } else {
                        (MemoryIndex::label(0), true)
                    }
                } else {
                    (MemoryIndex::disp(symbol.value), false)
                }
            }
            Node::BinaryOp { op, left, right } => {
                // We will do some normalization very shortly so it needs to be mutable
                let mut op = *op;

                let (mut left_result, left_relocation) =
                    self.evaluate_memory_index(left, current_section)?;
                let (mut right_result, right_relocation) =
                    self.evaluate_memory_index(right, current_section)?;

                let relocation = left_relocation | right_relocation;

                // Normalize subtracting a constant displacement from a register value by negating
                // the constant displacement and switching the operation into addition
                if op == BinaryOp::Sub && left_result.has_register() && right_result.is_number() {
                    op = BinaryOp::Add;
                    right_result.disp = right_result.disp.wrapping_neg();
                }

                if left_result.is_number() && right_result.is_number() {
                    let new_value = op.calculate(left_result.disp, right_result.disp);

                    let mut index = MemoryIndex::disp(new_value);
                    index.is_label = left_result.is_label | right_result.is_label;
                    (index, relocation)
                } else if op == BinaryOp::Add {
                    left_result.disp = left_result.disp.wrapping_add(right_result.disp);

                    if right_result.base.is_valid() {
                        if left_result.base.is_invalid() {
                            left_result.base = right_result.base;
                        } else if left_result.index.is_invalid() {
                            left_result.index = right_result.base;
                        } else {
                            return Err(anyhow!(
                                "Attempting to add too many registers in index expression"
                            ));
                        }
                    }
                    if right_result.index.is_valid() {
                        // We only check the left index and not the left base because if the code is proper, then once a
                        // register goes into the index slot there isn't a reason it should go back
                        // to the base
                        if left_result.index.is_invalid() {
                            left_result.index = right_result.index;
                            left_result.scale = right_result.scale;
                        } else {
                            return Err(anyhow!(
                                "Attempting to add too many registers in index expression"
                            ));
                        }
                    }

                    left_result.is_label |= right_result.is_label;

                    (left_result, relocation)
                } else if op == BinaryOp::Mul {
                    if left_result.is_label || right_result.is_label {
                        return Err(anyhow!("Cannot use label as scalar"));
                    }

                    if relocation {
                        return Err(anyhow!("Cannot relocate a scalar value"));
                    }

                    let (scale, index) = if left_result.disp != 0 && right_result.disp == 0 {
                        // The left result is the scalar, it cannot have any registers associated
                        // with it
                        if left_result.base.is_valid() || left_result.index.is_valid() {
                            return Err(anyhow!("Invalid memory index expression"));
                        }
                        // The right result can't have both registers associated with it, just one
                        if right_result.base.is_valid() && right_result.index.is_valid() {
                            return Err(anyhow!("Invalid memory index expression"));
                        }

                        if right_result.base.is_valid() {
                            (left_result.disp, right_result.base)
                        } else {
                            let right_scale: u64 = right_result.scale.into();
                            (left_result.disp * right_scale, right_result.index)
                        }
                    } else if right_result.disp != 0 && left_result.disp == 0 {
                        if right_result.base.is_valid() || right_result.index.is_valid() {
                            return Err(anyhow!("Invalid memory index expression"));
                        } else if left_result.base.is_valid() && left_result.index.is_valid() {
                            return Err(anyhow!("Invalid memory index expression"));
                        }

                        if left_result.base.is_valid() {
                            (right_result.disp, left_result.base)
                        } else {
                            let left_scale: u64 = left_result.scale.into();
                            (right_result.disp * left_scale, left_result.index)
                        }
                    } else {
                        return Err(anyhow!("Invalid memory index expression"));
                    };

                    let index = MemoryIndex {
                        disp: 0,
                        base: Register::none(),
                        index,
                        scale,
                        is_label: false,
                    };

                    (index, false)
                } else if op == BinaryOp::Div {
                    if relocation {
                        return Err(anyhow!("Cannot relocate a scalar value"));
                    }

                    if left_result.is_label | right_result.is_label {
                        return Err(anyhow!("Cannot use label as a scalar value"));
                    }
                    if right_result.base.is_valid() || right_result.index.is_valid() {
                        return Err(anyhow!("Cannot divide by a register value"));
                    }

                    if left_result.base.is_valid() && left_result.index.is_valid() {
                        return Err(anyhow!("Invalid memory index expression"));
                    }

                    if right_result.disp == 0 {
                        return Err(anyhow!("Cannot divide by 0"));
                    }

                    let (scale, index) = if left_result.base.is_valid() {
                        (1 / right_result.disp, left_result.base)
                    } else {
                        (left_result.scale / right_result.disp, left_result.index)
                    };

                    let index = MemoryIndex {
                        disp: 0,
                        base: Register::none(),
                        index,
                        scale,
                        is_label: false,
                    };

                    (index, false)
                } else {
                    return Err(anyhow!("Invalid memory index expression"));
                }
            }
            Node::UnaryOp { op, expr } => {
                let (mut result, relocation) = self.evaluate_memory_index(expr, current_section)?;

                if result.base.is_valid() || result.index.is_valid() {
                    return Err(anyhow!("Cannot perform unary op on a register value"));
                }

                let value = op.calculate(result.disp);
                result.disp = value;

                (result, relocation)
            }
            Node::Expression(expr) => self.evaluate_memory_index(expr, current_section)?,
        };

        Ok((index, relocation))
    }
}

impl Assembler {
    pub fn assemble(filename: String, source: String) -> Result<Self> {
        debug!("Assembling file {filename}");
        let mut assembler = Assembler {
            filename,
            symbols: SymbolTable::new(),
            global_symbols: Vec::new(),
            forward_references: Vec::new(),
            sections: SectionMap::new(),
            current_line: 0,
        };

        let lexer = Lexer::new(&source);
        let iter = TokenIter::new(lexer);

        let mut current_line = 1usize;
        let tokens: Vec<AssemblerToken> = match iter
            .map(|token| match token {
                Ok(token) => {
                    if let Token::Newline = token {
                        current_line += 1;
                    }

                    Ok(AssemblerToken {
                        token,
                        line: current_line,
                    })
                }
                Err(e) => Err(e),
            })
            .collect::<Result<_>>()
        {
            Ok(vec) => vec,
            Err(e) => bail!("Error {}:{current_line}\n\t{e}", assembler.filename),
        };

        let mut token_iter = tokens.iter().peekable();

        let result = assembler.parse_source(&mut token_iter);

        result
            .then(|| assembler)
            .with_context(|| "Failed to assemble source")
    }

    fn parse_source<'a>(&mut self, token_iter: &mut Peekable<impl AsmTokenIter<'a>>) -> bool {
        let mut success = true;

        while let Some(token) = token_iter.next() {
            self.current_line = token.line;
            if let Err(e) = self.parse_token(&token, token_iter) {
                println!("Error {}:{}:\n\t{e}", self.filename, token.line);
                success = false;
                while let Some(token) = token_iter.next() {
                    if let Token::Newline = token.token {
                        break;
                    }
                }
            }
        }

        success
    }

    fn parse_token<'a>(
        &mut self,
        token: &AssemblerToken,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        match &token.token {
            Token::Mnemonic(instruction) => self.parse_instruction(&instruction, tokens),
            Token::Keyword(keyword) => self.parse_keyword(keyword, tokens),
            Token::Directive(directive) => self.parse_directive(*directive, tokens),
            Token::Identifier(id) => self.parse_label(id.clone(), tokens),
            Token::Newline => Ok(()),
            other => Err(anyhow!("Unknown token {other:?}")),
        }
    }

    fn parse_instruction<'a>(
        &mut self,
        instruction: &Mnemonic,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        // All possible instruction encodings of the current mnemonic
        let encodings = get_encodings(*instruction);

        let mut operands = [Operand::None; MAX_OPERANDS];
        let mut index_addresses = [MemoryIndex::default(); MAX_OPERANDS];
        let mut reloc_needed = [false; MAX_OPERANDS];
        let mut types = [OperandFlags::empty(); MAX_OPERANDS];

        // This is the expression for each operand
        let mut operand_exprs = std::array::from_fn(|_| None);

        let operand_count = self.parse_operands(
            tokens,
            &mut operands,
            &mut reloc_needed,
            &mut types,
            &mut index_addresses,
            &mut operand_exprs,
        )?;

        let mut chosen_encoding: Option<InstEncoding> = None;

        for (_, encoding) in encodings.iter().enumerate() {
            // Skip this if the operand counts don't match since this shows right away that this
            // encoding isn't the correct one
            if encoding.operand_count() != operand_count {
                continue;
            }

            // If the parsed instruction is the same as the instruction template
            let mut matches = true;
            // Iterate over the current instruction's operands and the instruction encoding's
            // operands to determine if they all match
            for (encoding_type, type_) in izip!(&encoding.operands, &mut types).take(operand_count)
            {
                // Set matches to false an break from the loop if the two instruction types don't
                // match
                if !encoding_type.intersects(*type_) {
                    matches = false;
                    break;
                }
            }

            // We found the right instruction encoding
            if matches {
                for (type_, encoding_type) in
                    izip!(&mut types, &encoding.operands).take(operand_count)
                {
                    *type_ &= *encoding_type;
                    // There should only be one type set
                    assert_eq!(
                        type_.bits().count_ones(),
                        1,
                        "There should only be one operand type set"
                    );
                }

                chosen_encoding = Some(*encoding);
                break;
            }
        }

        let Some(encoding) = chosen_encoding else {
            return Err(anyhow!("Invalid instruction"));
        };

        debug!(
            "Chosen encoding: {chosen_encoding:?} {}:{}",
            self.filename, self.current_line
        );

        let instruction = Instruction {
            encoding,
            operand_count,
            indexes: index_addresses,
            types,
            operands,
            exprs: operand_exprs,
            reloc: reloc_needed,
        };

        let _ = self.emit_instruction(instruction)?;
        Ok(())
    }

    /// TODO: Documentation
    fn parse_operands<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
        operands: &mut [Operand; MAX_OPERANDS],
        reloc_needed: &mut [bool; MAX_OPERANDS],
        types: &mut [OperandFlags; MAX_OPERANDS],
        index_addresses: &mut [MemoryIndex; MAX_OPERANDS],
        operand_exprs: &mut [Option<Box<Node>>; MAX_OPERANDS],
    ) -> Result<usize> {
        assert!(
            operands.len() == types.len() && operand_exprs.len() == types.len(),
            "Arrays not the same size"
        );

        if let Some(Token::Newline) = tokens.peek().map(|a| &a.token) {
            return Ok(0);
        }

        let mut num_operands = 0;

        let mut expecting_comma = false;

        while let Some(token) = tokens.peek() {
            let token = &token.token;
            if expecting_comma {
                if let Token::Comma = token {
                    let _ = tokens.next();
                    expecting_comma = false;
                } else if let Token::Newline = token {
                    // Don't consume the newline
                    expecting_comma = false;
                    break;
                } else {
                    return Err(anyhow!("Expected comma"));
                }
            } else {
                expecting_comma = true;
                let (current_section, _) = self.sections.get_section()?;

                // The operand is a memory index, otherwise it's an expression/register
                if let Token::LSqrBrace = tokens.peek().context("Expected token")?.token {
                    let _ = tokens.next();
                    let expr = parse_expr(tokens)?;

                    let Token::RSqrBrace = tokens
                        .next()
                        .context("Expected closing square bracket")?
                        .token
                    else {
                        return Err(anyhow!("Expected closing square bracket"));
                    };

                    // This function doesn't check if the scalar value is valid, and we won't
                    // either. The emit function will check it.
                    let (index, relocation) = self.evaluate_memory_index(&expr, current_section)?;

                    if let Some(memory_index) = index_addresses.get_mut(num_operands)
                        && let Some(reloc_needed) = reloc_needed.get_mut(num_operands)
                        && let Some(op_type) = types.get_mut(num_operands)
                        && let Some(operand_expr) = operand_exprs.get_mut(num_operands)
                    {
                        num_operands += 1;
                        *memory_index = index;
                        *reloc_needed = relocation;
                        *op_type = OperandFlags::INDEX;
                        *operand_expr = Some(expr);
                    } else {
                        return Err(anyhow!("Too many operands. Max is {MAX_OPERANDS}"));
                    }
                } else {
                    #[derive(Debug, Eq, PartialEq)]
                    enum FlagOverride {
                        None,
                        /// Always set the flags associated with immediate values
                        Constant,
                        /// Always set every flags associated with reading from memory
                        Memory,
                        /// Always set the flags associated with hardcoded addresses
                        Addr,
                        /// Always set the flags associated with reading from memory with PC relative
                        /// displacements
                        Offset,
                    }

                    let flag_override = match tokens.peek().context("Expected token")?.token {
                        Token::Dollar => {
                            let _ = tokens.next();
                            FlagOverride::Constant
                        }
                        Token::Mul => {
                            let _ = tokens.next();
                            FlagOverride::Memory
                        }
                        Token::AtSign => {
                            let _ = tokens.next();
                            FlagOverride::Addr
                        }
                        Token::Ampersand => {
                            let _ = tokens.next();
                            FlagOverride::Offset
                        }
                        _ => FlagOverride::None,
                    };

                    let expr = parse_expr(tokens)?;
                    let result = self.evaluate_expression(&expr, current_section)?;

                    if let Some(operand) = operands.get_mut(num_operands)
                        && let Some(reloc_needed) = reloc_needed.get_mut(num_operands)
                        && let Some(op_type) = types.get_mut(num_operands)
                        && let Some(operand_expr) = operand_exprs.get_mut(num_operands)
                    {
                        *operand_expr = Some(expr);
                        num_operands += 1;
                        *reloc_needed = if let ExprResult::Constant { relocation, .. } = result {
                            relocation
                        } else {
                            false
                        };

                        (*operand, *op_type) = match result {
                            ExprResult::Register(register) => {
                                if flag_override != FlagOverride::None {
                                    bail!("Cannot use operand type specifiers with registers");
                                } else {
                                    (Operand::Register(register), register.get_operand_flag())
                                }
                            }
                            // If it is an ExprResult::Constant we return a tuple containing
                            // (Operand::Constant(constant), {OPERAND_FLAGS})
                            ExprResult::Constant {
                                constant,
                                is_label,
                                relocation,
                            } => (
                                Operand::Constant(constant),
                                match flag_override {
                                    FlagOverride::None => {
                                        get_operand_from_expr_result(is_label, relocation)
                                    }
                                    FlagOverride::Constant => operand!(IMM),
                                    FlagOverride::Memory => operand!(ADDR | DISP),
                                    FlagOverride::Addr => operand!(ADDR),
                                    FlagOverride::Offset => operand!(DISP),
                                },
                            ),
                        };
                    } else {
                        return Err(anyhow!("Too many operands. Max is {MAX_OPERANDS}"));
                    }
                }
            }
        }

        if !expecting_comma {
            Ok(num_operands)
        } else {
            if tokens.peek().is_none() {
                Ok(num_operands)
            } else {
                Err(anyhow!("Expected comma"))
            }
        }
    }

    fn parse_keyword<'a>(
        &mut self,
        keyword: &Keyword,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        match keyword {
            Keyword::Const => self.parse_const(tokens),
        }
    }

    fn parse_label<'a>(
        &mut self,
        name: String,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let token = &tokens.next().context("Expected token but got EOF")?.token;

        let Token::Colon = token else {
            bail!("Expected colon after identifier but got {token}");
        };

        let (current_section, section) = self.sections.get_section()?;
        let position = section.cursor();

        debug!("Label at {}+{position:#x}", section.name);
        self.symbols
            .insert_symbol(name, position as u64, Type::Label, Some(current_section))?;

        Ok(())
    }

    fn parse_const<'a>(&mut self, tokens: &mut Peekable<impl AsmTokenIter<'a>>) -> Result<()> {
        let name = tokens
            .next()
            .with_context(|| "Expected variable name")?
            .token
            .as_identifier()
            .with_context(|| format!("Expected an identifier"))?
            .to_string();
        if !tokens.is_equal_sign() {
            bail!("Expected equal sign");
        }

        let expr = parse_expr(tokens)?;
        let (value, relocation) = self.evaluate_non_operand_expression(&expr)?;

        if !tokens.is_newline_or_eof() {
            bail!("Expected a newline or EOF");
        }

        self.symbols
            .insert_symbol(name, value, Type::Constant, None)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    /// Easily turn an &str into an owned string without much typing
    fn s(source: &str) -> String {
        source.to_string()
    }

    use super::*;

    fn default_assembler() -> Assembler {
        Assembler {
            filename: "test.asm".to_string(),
            symbols: SymbolTable::new(),
            global_symbols: Vec::new(),
            forward_references: Vec::new(),
            sections: SectionMap::new(),
            current_line: 0,
        }
    }

    #[test]
    fn test_assembler() {
        let source = s("
        .section .entry
        const value = 10
        mov r0, value
        ");

        let _ = Assembler::assemble(s("test"), source).unwrap();

        let source = s("
        const value = 10
        mov r0, value
        ");

        let _ = Assembler::assemble(s("test"), source).unwrap_err();
    }

    #[test]
    fn test_memory_index() {
        // let mut assembler = default_assembler();
        //
        // let token_iter = vec!["100"];
        // let tokens = TokenIter::new(token_iter.iter().copied());
        //
        // let source = SourceCode::new("100".to_string());
        // let mut tokens = source.tokens();
        // let expr = parse_expr(&mut tokens).expect("Expression should be valid");
        // let (index, relocation) = assembler
        //     .evaluate_memory_index(&expr, Assembler::NO_SECTION)
        //     .expect("Memory index should be valid");
        // assert_eq!(relocation, false);
        // assert_eq!(index.disp, 100);
        // assert!(index.base.is_invalid());
        // assert!(index.index.is_invalid());
        // assert!(index.scale == 1);
        // assert_eq!(index.is_label, false);
        //
        // let source = SourceCode::new("r0".to_string());
        // let mut tokens = source.tokens();
        // let expr = parse_expr(&mut tokens).expect("Expression should be valid");
        // let (index, relocation) = assembler
        //     .evaluate_memory_index(&expr, Assembler::NO_SECTION)
        //     .expect("Memory index should be valid");
        // assert_eq!(relocation, false);
        // assert_eq!(index.disp, 0);
        // assert!(index.base == Register::new_gp(0));
        // assert!(index.index.is_invalid());
        // assert!(index.scale == 1);
        // assert_eq!(index.is_label, false);
        //
        // let source = SourceCode::new("r2 + 100".to_string());
        // let mut tokens = source.tokens();
        // let expr = parse_expr(&mut tokens).expect("Expression should be valid");
        // let (index, relocation) = assembler
        //     .evaluate_memory_index(&expr, Assembler::NO_SECTION)
        //     .expect("Memory index should be valid");
        // assert_eq!(relocation, false);
        // assert_eq!(index.disp, 100);
        // assert!(index.base == Register::new_gp(2));
        // assert!(index.index.is_invalid());
        // assert!(index.scale == 1);
        // assert_eq!(index.is_label, false);
        //
        // let source = SourceCode::new("2 * r7 + r1 + 1029".to_string());
        // let mut tokens = source.tokens();
        // let expr = parse_expr(&mut tokens).expect("Expression should be valid");
        // let (index, relocation) = assembler
        //     .evaluate_memory_index(&expr, Assembler::NO_SECTION)
        //     .expect("Memory index should be valid");
        // assert_eq!(relocation, false);
        // assert_eq!(index.disp, 1029);
        // assert!(index.base == Register::new_gp(1));
        // assert!(index.index == Register::new_gp(7));
        // assert!(index.scale == 2);
        // assert_eq!(index.is_label, false);
    }
}
