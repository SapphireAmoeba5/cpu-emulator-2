mod directive;
mod emit;
mod parse;
pub mod symbol_table;
use bitflags::Flag;
use itertools::izip;

use std::collections::HashMap;
use std::usize;

use anyhow::anyhow;
use spdlog::debug;

use crate::assembler::symbol_table::SymbolTable;
use crate::expression::{Mode, Node, parse_expr};
use crate::instruction::Mnemonic;
use crate::opcode::{
    EncodingFlags, InstEncoding, MAX_OPERANDS, OperandFlags, Relocation, get_encodings,
};
use crate::operand;
use crate::section::{self, Section};
pub use emit::calculate_disp32_offset;

use super::assembler_source::*;
use super::tokens::*;

use anyhow::{Context, Result};

fn get_operand_from_expr_result(result: ExprResult) -> (Operand, OperandFlags) {
    match result.type_ {
        ExprType::Register => {
            let mut operand = operand!(REG);
            if result.register.is_gp() {
                operand |= operand!(GP_REG);
            }
            (Operand::Register(result.register), operand)
        }
        ExprType::Constant => {
            let operand;

            if result.is_label {
                // TODO: If we ever implement non position independent machine code then we should
                // check for it here
                operand = operand!(DISP);
            } else {
                operand = operand!(IMM | ADDR64 | DISP)
            }
            (Operand::Constant(result.immediate), operand)
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Operand {
    None,
    Constant(u64),
    Register(Register),
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
    /// Reloaction per operand
    reloc: [Relocation; MAX_OPERANDS],
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExprType {
    Constant,
    Register,
}

#[derive(Debug, Clone, Copy)]
pub struct ExprResult {
    type_: ExprType,
    /// Is valid if `type_` is set to ExprType::Immediate
    immediate: u64,
    /// Is valid if `type_` is set to ExprType::Register
    register: Register,
    /// If the result of this expression is a label or memory offset
    is_label: bool,
    /// If the result would require a relocation
    relocation: bool,
}

impl ExprResult {
    pub fn new_imm(immediate: u64) -> Self {
        Self {
            type_: ExprType::Constant,
            immediate,
            register: Register::none(),
            is_label: false,
            relocation: false,
        }
    }

    pub fn new_reloc() -> Self {
        Self {
            type_: ExprType::Constant,
            immediate: 0,
            register: Register::none(),
            is_label: false,
            relocation: true,
        }
    }
}

pub struct Assembler {
    pub filename: String,
    pub symbols: SymbolTable,
    pub global_symbols: Vec<String>,

    pub forward_references: Vec<ForwardReferenceEntry>,
    /// Index of the current section being written to
    current_section: Option<usize>,
    pub sections: Vec<Section>,
    /// Section name -> index into sections vec
    pub section_map: HashMap<String, usize>,

    /// The current line number being parsed
    current_line: usize,
}

impl Assembler {
    const NO_SECTION: usize = usize::MAX;
}

impl Assembler {
    /// Does the fixup
    fn do_fixup(&mut self, relocation: &ForwardReferenceEntry) -> Result<bool> {
        let result = self.evalute_expression(&relocation.expr, relocation.section)?;

        assert!(matches!(result.type_, ExprType::Constant));

        if result.relocation {
            return Ok(false);
        }
        let constant = result.immediate;

        match relocation.relocation {
            Relocation::Abs64 => {
                if result.is_label {
                    // TODO: Add error messages
                }
                let offset = relocation.offset;

                self.sections[relocation.section].replace_bytes(offset, &constant.to_le_bytes());
            }
            Relocation::Abs8 => {
                let constant: u8 = constant
                    .try_into()
                    .context("Symbol's value is too large for a ABS8 relocation")?;
                let offset = relocation.offset;
                self.sections[relocation.section].replace_bytes(offset, &constant.to_le_bytes());
            }
            Relocation::PC32 => {
                // Where the program counter will be when the instruction is executed
                let pc: u64 = (relocation.offset + 4).try_into().unwrap();
                let offset = emit::calculate_disp32_offset(pc, constant)?;

                debug!(
                    "Fixing PC32 relocation at {:#x} to {offset:#x} {}+{:#x}",
                    relocation.offset,
                    self.sections[relocation.section].name,
                    pc as i64 + offset as i64
                );

                self.sections[relocation.section]
                    .replace_bytes(relocation.offset, &offset.to_le_bytes());
            }
            // TODO: Implement the other relocation types
            _ => todo!("Relocation {:?}", relocation.relocation),
        }

        Ok(true)
    }

    /// Iterates over all the relocations and attempts fixups on them if possible
    fn fix_forward_references(&mut self) -> Result<()> {
        let relocation_count = self.forward_references.len();
        let mut resolved = vec![false; relocation_count];

        // std::mem::take to appease the borrow checker
        let mut relocations = std::mem::take(&mut self.forward_references);

        let mut failed = false;
        for i in 0..relocation_count {
            let relocation = &relocations[i];
            let line_number = relocation.line_number;
            let was_resolved = match self.do_fixup(relocation) {
                Ok(resolved) => resolved,
                Err(e) => {
                    failed = true;
                    println!("Error {}:{}: {e}", self.filename, line_number);
                    false
                }
            };

            resolved[i] = was_resolved;
        }

        let mut resolved = resolved.iter();
        relocations.retain(|_| !*resolved.next().unwrap());

        self.forward_references = relocations;

        if failed {
            Err(anyhow!("Failed to fix forward references"))
        } else {
            Ok(())
        }
    }

    fn evalute_expression(&self, expr: &Box<Node>, current_section: usize) -> Result<ExprResult> {
        match &**expr {
            Node::Constant(value) => Ok(ExprResult::new_imm(*value)),
            Node::Register(register) => Ok(ExprResult {
                type_: ExprType::Register,
                immediate: 0,
                register: *register,
                is_label: false,
                relocation: false,
            }),
            Node::Identifier(id) => {
                // If the symbol isn't defined
                let Some(symbol) = self.symbols.get_symbol(id) else {
                    return Ok(ExprResult::new_reloc());
                };

                // The symbol is a label
                if let Some(section) = symbol.section_index {
                    if section == current_section {
                        Ok(ExprResult {
                            type_: ExprType::Constant,
                            immediate: symbol.value,
                            register: Register::none(),
                            is_label: true,
                            relocation: false,
                        })
                    } else {
                        // The label exists in a different section
                        Ok(ExprResult {
                            type_: ExprType::Constant,
                            immediate: 0,
                            register: Register::none(),
                            is_label: true,
                            relocation: true,
                        })
                    }
                } else {
                    // The symbol is a constant
                    Ok(ExprResult {
                        type_: ExprType::Constant,
                        immediate: symbol.value,
                        register: Register::none(),
                        is_label: false,
                        relocation: false,
                    })
                }
            }
            Node::Expression(expr) => self.evalute_expression(expr, current_section),
            Node::BinaryOp { op, left, right } => {
                let left = self.evalute_expression(left, current_section)?;
                let right = self.evalute_expression(right, current_section)?;

                if left.type_ == ExprType::Register || right.type_ == ExprType::Register {
                    Err(anyhow!("Invalid operation on register"))
                } else if left.relocation || right.relocation {
                    Ok(ExprResult {
                        type_: ExprType::Constant,
                        immediate: 0,
                        register: Register::none(),
                        is_label: left.is_label | right.is_label,
                        relocation: true,
                    })
                } else {
                    let immediate = op.calculate(left.immediate, right.immediate);
                    Ok(ExprResult {
                        type_: ExprType::Constant,
                        immediate,
                        register: Register::none(),
                        is_label: left.is_label | right.is_label,
                        // Neither the left or right hand side are relocations so hardcode this to
                        // false
                        relocation: false,
                    })
                }
            }

            Node::UnaryOp { op, expr } => {
                let operand = self.evalute_expression(expr, current_section)?;

                if operand.type_ == ExprType::Register {
                    Err(anyhow!("Invalid operation on register"))
                } else {
                    let immediate = op.calculate(operand.immediate);
                    Ok(ExprResult {
                        type_: operand.type_,
                        immediate,
                        register: Register::none(),
                        is_label: operand.is_label,
                        relocation: operand.relocation,
                    })
                }
            }
        }
    }

    fn evaluate_non_operand_expression(&self, expr: &Box<Node>) -> Result<u64> {
        let result = self.evalute_expression(expr, Self::NO_SECTION)?;

        if result.is_label {
            return Err(anyhow!("Cannot use labels here"));
        } else if result.relocation {
            return Err(anyhow!("Cannot use undefined symbols here"));
        }

        match result.type_ {
            ExprType::Register => Err(anyhow!("Cannot use registers here")),
            ExprType::Constant => Ok(result.immediate),
        }
    }

    pub fn emit_relocation(&mut self, relocation: Relocation, offset: usize, expr: Box<Node>) {
        let section = self.current_section.unwrap();
        let name: &str = relocation.into();
        debug!(
            "Emitting a {} relocation at {}+{offset:#x}",
            name, self.sections[section].name
        );
        let entry = ForwardReferenceEntry {
            relocation,
            section,
            offset,
            expr,
            line_number: self.current_line,
        };

        self.forward_references.push(entry);
    }

    fn get_section_mut(&mut self) -> Result<&mut Section> {
        let section = self.get_section_index()?;
        let section = self.sections.get_mut(section).unwrap();
        Ok(section)
    }

    fn get_section_index(&self) -> Result<usize> {
        if let Some(section) = self.current_section {
            Ok(section)
        } else {
            Err(anyhow!(
                "Section to place data not defined. Try doing .section {{section_name}} before your code"
            ))
        }
    }
}

impl Assembler {
    pub fn assemble(filename: String, source: String) -> Result<Self> {
        debug!("Assembling file {filename}");
        // let sections = Vec::new();
        let sections = Vec::new();
        let section_map = HashMap::new();

        let mut assembler = Assembler {
            filename,
            symbols: SymbolTable::new(),
            global_symbols: Vec::new(),
            forward_references: Vec::new(),
            current_section: None,
            sections,
            section_map,
            current_line: 0,
        };

        let result = assembler.parse_source(source);

        if result {
            assembler.fix_forward_references()?;
        }

        result
            .then(|| assembler)
            .with_context(|| "Failed to assemble source")
    }

    fn parse_source(&mut self, source: String) -> bool {
        let source_code = SourceCode::new(source);

        let mut token_iter = source_code.tokens();
        let mut success = true;

        loop {
            let current_line = token_iter.line();
            self.current_line = current_line;
            let token = token_iter.next();
            if let Ok(token) = token {
                match token {
                    Some(token) => {
                        if let Err(e) = self.parse_token(token, &mut token_iter) {
                            println!("Error {}:{current_line}:\n\t{e}", self.filename);
                            success = false;
                            token_iter.skip_line();
                        }
                    }
                    // Stop iterating when iterator reaches None
                    None => break,
                }
            } else if let Err(e) = &token {
                println!("Error {}:{current_line}:\n\t{e}", self.filename);
                success = false;
                token_iter.skip_line();
            }
        }

        success
    }

    fn parse_token(&mut self, token: Token, tokens: &mut TokenIter) -> Result<()> {
        match token {
            Token::Mnemonic(instruction) => self.parse_instruction(&instruction, tokens),
            Token::Keyword(keyword) => self.parse_keyword(&keyword, tokens),
            Token::Directive(directive) => self.parse_directive(directive, tokens),
            Token::Identifier(id) => self.parse_label(id, tokens),
            Token::Newline => Ok(()),
            other => Err(anyhow!("Unknown token {other:?}")),
        }
    }

    fn parse_instruction(&mut self, instruction: &Mnemonic, tokens: &mut TokenIter) -> Result<()> {
        // All possible instruction encodings of the current mnemonic
        let encodings = get_encodings(*instruction);

        let mut operands = [Operand::None; MAX_OPERANDS];
        let mut reloc_needed = [false; MAX_OPERANDS];
        let mut types = [OperandFlags::empty(); MAX_OPERANDS];

        // This is the expression for each operand
        let mut operand_exprs = std::array::from_fn(|_| None);

        let operand_count = self.parse_operands(
            tokens,
            &mut operands,
            &mut reloc_needed,
            &mut types,
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
            for (encoding_type, operand_type) in
                izip!(&encoding.operands, &types).take(operand_count)
            {
                // Set matches to false an break from the loop if the two instruction types don't
                // match
                if !encoding_type.intersects(*operand_type) {
                    matches = false;
                    break;
                }
            }

            // We found the right instruction encoding
            if matches {
                for (type_, encoding_operands) in
                    izip!(&mut types, &encoding.operands).take(operand_count)
                {
                    *type_ &= *encoding_operands;
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

        let mut instr_relocs = [Relocation::None; MAX_OPERANDS];

        for (reloc_needed, type_, reloc) in
            izip!(&reloc_needed, &types, &mut instr_relocs).take(operand_count)
        {
            if *reloc_needed {
                // Figure out which relocation type we need
                if type_.intersects(operand!(IMM)) {
                    if type_.intersects(operand!(IMM8)) {
                        *reloc = Relocation::Abs8;
                    } else if type_.intersects(operand!(IMM32)) {
                        *reloc = Relocation::Abs32;
                    } else if type_.intersects(operand!(IMM64)) {
                        *reloc = Relocation::Abs64;
                    } else {
                        unreachable!();
                    }
                } else if type_.intersects(operand!(DISP)) {
                    if type_.intersects(operand!(DISP32)) {
                        *reloc = Relocation::PC32;
                    } else {
                        unreachable!();
                    }
                } else if type_.intersects(operand!(ADDR)) {
                    if type_.intersects(operand!(ADDR64)) {
                        *reloc = Relocation::Addr64;
                    } else {
                        unreachable!();
                    }
                } 
                else {
                    unreachable!("No other operand flag should need a relocation");
                }
            }
        }

        let instruction = Instruction {
            encoding,
            operand_count,
            types,
            operands,
            exprs: operand_exprs,
            reloc: instr_relocs,
        };

        let _ = self.emit_instruction(instruction)?;
        Ok(())
    }

    /// TODO: Documentation
    fn parse_operands(
        &mut self,
        tokens: &mut TokenIter,
        operands: &mut [Operand; MAX_OPERANDS],
        reloc_needed: &mut [bool; MAX_OPERANDS],
        types: &mut [OperandFlags; MAX_OPERANDS],
        operand_exprs: &mut [Option<Box<Node>>; MAX_OPERANDS],
    ) -> Result<usize> {
        assert!(
            operands.len() == types.len() && operand_exprs.len() == types.len(),
            "Arrays not the same size"
        );

        let mut num_operands = 0;

        let mut expecting_comma = false;

        while let Some(token) = tokens.peek()? {
            if expecting_comma {
                if matches!(token, Token::Comma) {
                    let _ = tokens.next();
                    expecting_comma = false;
                } else if matches!(token, Token::Newline) {
                    // Don't consume the newline
                    expecting_comma = false;
                    break;
                } else {
                    return Err(anyhow!("Expected comma"));
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

                expecting_comma = true;
                let current_section = self.get_section_index()?;

                let flag_override = match tokens.peek()? {
                    Some(Token::Dollar) => {
                        let _ = tokens.next();
                        FlagOverride::Constant
                    }
                    Some(Token::Mul) => {
                        let _ = tokens.next();
                        FlagOverride::Memory
                    }
                    Some(Token::AtSign) => {
                        let _ = tokens.next();
                        FlagOverride::Addr
                    }
                    Some(Token::Ampersand) => {
                        let _ = tokens.next();
                        FlagOverride::Offset
                    }
                    _ => FlagOverride::None,
                };

                let expr = parse_expr(tokens)?;
                let result = self.evalute_expression(&expr, current_section)?;

                if let Some(operand) = operands.get_mut(num_operands)
                    && let Some(reloc_needed) = reloc_needed.get_mut(num_operands)
                    && let Some(op_type) = types.get_mut(num_operands)
                    && let Some(operand_expr) = operand_exprs.get_mut(num_operands)
                {
                    *operand_expr = Some(expr);
                    num_operands += 1;
                    *reloc_needed = result.relocation;

                    if !matches!(flag_override, FlagOverride::None)
                        && matches!(result.type_, ExprType::Register)
                    {
                        return Err(anyhow!(
                            "Cannont use operand type specifiers with registers"
                        ));
                    }

                    (*operand, *op_type) = match flag_override {
                        FlagOverride::None => get_operand_from_expr_result(result),
                        FlagOverride::Constant => (Operand::Constant(result.immediate), operand!(IMM)),
                        FlagOverride::Memory => (Operand::Constant(result.immediate), operand!(ADDR | DISP)),
                        FlagOverride::Addr => (Operand::Constant(result.immediate), operand!(ADDR)),
                        FlagOverride::Offset => (Operand::Constant(result.immediate), operand!(DISP)),
                        
                    };
                } else {
                    return Err(anyhow!("Too many operands. Max is {MAX_OPERANDS}"));
                }
            }
        }

        if !expecting_comma {
            Ok(num_operands)
        } else {
            Err(anyhow!("Expected comma"))
        }
    }

    fn parse_keyword(&mut self, keyword: &Keyword, tokens: &mut TokenIter) -> Result<()> {
        match keyword {
            Keyword::Const => self.parse_const(tokens),
        }
    }

    fn parse_label(&mut self, name: String, tokens: &mut TokenIter) -> Result<()> {
        let token = tokens.next()?.context("Expected token but got EOF")?;

        if !matches!(token, Token::Colon) {
            return Err(anyhow!(
                "Expected colon after identifier but got {}",
                token.to_string()
            ));
        }

        let current_section = self.get_section_index()?;
        let position = self.sections[current_section].cursor();

        debug!(
            "Label at {}+{position:#x}",
            self.sections[current_section].name
        );
        self.symbols
            .insert_symbol(name, position as u64, Some(current_section))?;

        Ok(())
    }

    fn parse_const(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let name = tokens
            .next()?
            .with_context(|| "Expected variable name")?
            .to_identifier()
            .with_context(|| format!("Expected an identifier"))?;
        tokens.is_equal_sign()?;

        let expr = parse_expr(tokens)?;
        let value = self.evalute_expression(&expr, Self::NO_SECTION)?;
        if value.relocation || value.is_label || value.type_ != ExprType::Constant {
            return Err(anyhow!("Invalid expression for constant"));
        }

        tokens.newline_or_eof()?;
        self.symbols.insert_symbol(name, value.immediate, None)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use super::*;

    fn default_assembler() -> Assembler {
        Assembler {
            section_map: HashMap::new(),
            filename: "test.asm".to_string(),
            symbols: SymbolTable::new(),
            global_symbols: Vec::new(),
            forward_references: Vec::new(),
            current_section: None,
            sections: Vec::new(),
            current_line: 0,
        }
    }

    #[test]
    fn test_something() {}
}
