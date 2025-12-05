mod directive;
mod emit;
mod parse;
pub mod symbol_table;
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

/// The value an expression contains
#[derive(Debug, Clone, Copy)]
enum ExprValue {
    None,
    Constant(u64),
    Register(Register),
    /// A relocation entry needs to be created
    Relocation,
}

impl ExprValue {
    pub fn is_constant(&self) -> bool {
        matches!(self, Self::Constant(_))
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Self::Register(_))
    }

    pub fn is_relocation(&self) -> bool {
        matches!(self, Self::Relocation)
    }

    /// Unwraps the constant, Panics if the value isn't a constant
    pub fn constant(&self) -> u64 {
        match self {
            ExprValue::Constant(constant) => *constant,
            _ => panic!("Value isn't a constant"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ExprResult {
    value: ExprValue,
    /// The possible types the expression can be
    pub flags: OperandFlags,
}

impl ExprResult {
    /// Shortcut for making a ExprResult that is a reference to an undefined symbol
    pub fn relocation(mode: Mode) -> Self {
        let flags = match mode {
            Mode::None => OperandFlags::IMM | OperandFlags::DISP,
            Mode::Immediate => OperandFlags::IMM,
            Mode::Addr => OperandFlags::DISP,
        };

        Self {
            value: ExprValue::Relocation,
            flags,
        }
    }

    pub fn disp_relocation() -> Self {
        Self {
            value: ExprValue::Relocation,
            flags: OperandFlags::DISP,
        }
    }

    /// Shortcut for making an ExprResult that is an immediate value, or displacement
    pub fn immediate(value: u64, mode: Mode) -> Self {
        let flags = match mode {
            Mode::None => OperandFlags::IMM | OperandFlags::DISP | OperandFlags::ADDR,
            Mode::Immediate => OperandFlags::IMM,
            Mode::Addr => OperandFlags::DISP | OperandFlags::ADDR,
        };

        Self {
            value: ExprValue::Constant(value),
            flags,
        }
    }

    /// Shortcut for making an ExprResult that is a displacement
    pub fn disp(value: u64) -> Self {
        Self {
            value: ExprValue::Constant(value),
            flags: OperandFlags::DISP,
        }
    }

    pub fn register(reg: Register) -> Self {
        Self {
            value: ExprValue::Register(reg),
            flags: reg.get_operand_flag(),
        }
    }
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

impl  Assembler {
    const NO_SECTION: usize = usize::MAX;
}

impl Assembler {
    /// Does the fixup
    fn do_fixup(&mut self, relocation: &ForwardReferenceEntry) -> Result<bool> {
        let result = self.evalute_expression(&relocation.expr, relocation.section)?;

        let flags = result.flags;
        let constant = match result.value {
            ExprValue::Constant(constant) => constant,
            ExprValue::Relocation => return Ok(false),
            ExprValue::Register(_) | ExprValue::None => unreachable!(),
        };

        match relocation.relocation {
            Relocation::Abs64 => {
                if !flags.intersects(OperandFlags::IMM) {
                    return Err(anyhow!(
                        "Cannot relocate a non-immediate value as an Abs64 relocation"
                    ));
                }
                let offset = relocation.offset;

                self.sections[relocation.section].replace_bytes(offset, &constant.to_le_bytes());
            }
            Relocation::Abs8 => {
                if !flags.intersects(OperandFlags::IMM) {
                    return Err(anyhow!(
                        "Cannot relocate a non-immediate value as an Abs64 relocation"
                    ));
                }
                
                let constant: u8 = constant.try_into().context("Symbol's value is too large for a ABS8 relocation")?;
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
            Node::Constant(value, mode) => Ok(ExprResult::immediate(*value, *mode)),
            Node::Register(reg) => Ok(ExprResult::register(*reg)),
            Node::Identifier(id, mode) => {
                // If the symbol isn't defined then return immediately
                let Some(symbol) = self.symbols.get_symbol(&id) else {
                    return Ok(ExprResult::relocation(*mode));
                };

                // This symbol is a label
                if let Some(section) = symbol.section_index {
                    // Labels can only be used as an address, nothing else
                    if *mode != Mode::None && *mode != Mode::Addr {
                        return Err(anyhow!("A label cannot be used as an immediate"));
                    }
                    // The label is in the current section (able to calculate the displacement
                    // right away)
                    if section == current_section {
                        Ok(ExprResult::disp(symbol.value))
                    } else {
                        Ok(ExprResult::disp_relocation())
                    }
                } else {
                    // The symbol is a constant value
                    Ok(ExprResult::immediate(symbol.value, *mode))
                }
            }
            Node::Expression(expr) => self.evalute_expression(expr, current_section),
            Node::BinaryOp { op, left, right } => {
                let left = self.evalute_expression(left, current_section)?;
                let right = self.evalute_expression(right, current_section)?;

                if left.value.is_register() || right.value.is_register() {
                    Err(anyhow!("Invalid operation on register"))
                } else if left.value.is_relocation() || right.value.is_relocation() {
                    // Get the intsersecting flags
                    let flags = left.flags & right.flags;
                    let value = ExprValue::Relocation;

                    Ok(ExprResult { value, flags })
                } else {
                    let mut flags = left.flags & right.flags;

                    if flags.is_empty() {
                        if (left.flags | right.flags).intersects(OperandFlags::DISP) {
                            flags |= OperandFlags::DISP;
                        }
                        if (left.flags | right.flags).intersects(OperandFlags::ADDR) {
                            flags |= OperandFlags::ADDR;
                        }

                        if flags.is_empty() {
                            unreachable!("This should be unreachable");
                        }
                    }

                    // Values are garunteed to be constants at this point
                    let left = left.value.constant();
                    let right = right.value.constant();

                    let value = ExprValue::Constant(op.calculate(left, right));

                    Ok(ExprResult { value, flags })
                }
            }
            Node::UnaryOp { op, expr } => {
                let operand = self.evalute_expression(expr, current_section)?;

                if operand.value.is_register() {
                    Err(anyhow!("Invalid operation on register"))
                } else if operand.value.is_relocation() {
                    Ok(operand)
                } else {
                    let value = ExprValue::Constant(op.calculate(operand.value.constant()));

                    Ok(ExprResult {
                        value,
                        flags: operand.flags,
                    })
                }
            }
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

        let mut expr_values = [ExprValue::None; MAX_OPERANDS];
        let mut types = [OperandFlags::empty(); MAX_OPERANDS];

        // This is the expression for each operand
        let mut operand_exprs = std::array::from_fn(|_| None);

        let operand_count =
            self.parse_operands(tokens, &mut expr_values, &mut types, &mut operand_exprs)?;

        let mut chosen_encoding: Option<InstEncoding> = None;

        for (index, encoding) in encodings.iter().enumerate() {
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
                    assert_eq!(type_.bits().count_ones(), 1);
                }

                chosen_encoding = Some(*encoding);
                break;
            }
        }

        let Some(encoding) = chosen_encoding else {
            return Err(anyhow!("Invalid instruction"));
        };

        let mut instr_operands = [Operand::None; MAX_OPERANDS];
        let mut instr_relocs = [Relocation::None; MAX_OPERANDS];

        for (operand, reloc, expr_value, expr_type) in
            izip!(&mut instr_operands, &mut instr_relocs, &expr_values, &types).take(operand_count)
        {
            match expr_value {
                ExprValue::Register(reg) => *operand = Operand::Register(*reg),
                ExprValue::Constant(num) => *operand = Operand::Constant(*num),
                ExprValue::Relocation => {
                    *operand = Operand::Constant(0);

                    // Figure out which relocation type we need
                    if expr_type.intersects(OperandFlags::IMM) {
                        if expr_type.intersects(OperandFlags::IMM8) {
                            *reloc = Relocation::Abs8;
                        } else if expr_type.intersects(OperandFlags::IMM32) {
                            *reloc = Relocation::Abs32;
                        } else if expr_type.intersects(OperandFlags::IMM64) {
                            *reloc = Relocation::Abs64;
                        }
                    } else if expr_type.intersects(OperandFlags::DISP) {
                        if expr_type.intersects(OperandFlags::DISP32) {
                            *reloc = Relocation::PC32;
                        }
                    } else {
                        unreachable!("No other operand flag should need a relocation");
                    }
                }
                ExprValue::None => unreachable!(
                    "Cannot be none since every value iterated over should be a valid operand"
                ),
            }
        }

        debug!("Chosen encoding: {chosen_encoding:?} {}:{}", self.filename, self.current_line);

        let instruction = Instruction {
            encoding,
            operand_count,
            types,
            operands: instr_operands,
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
        operands: &mut [ExprValue; MAX_OPERANDS],
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
                expecting_comma = true;
                let current_section = self.get_section_index()?;

                let expr = parse_expr(tokens)?;
                let result = self.evalute_expression(&expr, current_section)?;

                if let Some(operand) = operands.get_mut(num_operands)
                    && let Some(op_type) = types.get_mut(num_operands)
                    && let Some(operand_expr) = operand_exprs.get_mut(num_operands)
                {
                    // Convert result.value to an operand
                    *operand = result.value;
                    *op_type = result.flags;
                    *operand_expr = Some(expr);
                    num_operands += 1;
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
        if let ExprValue::Constant(value) = value.value {
            tokens.newline_or_eof()?;
            self.symbols.insert_symbol(name, value, None)
        } else {
            Err(anyhow!("Invalid expression for constant"))
        }
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
    fn test_something() {
        
    }
}
