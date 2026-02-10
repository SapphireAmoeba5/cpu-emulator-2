use std::collections::HashMap;

use crate::assembler::symbol_table::{self, SymbolTable};
use crate::assembler::{self, Assembler};
use crate::expression::{BinaryOp, Node};
use crate::opcode::Relocation;
use crate::section::Section;

use anyhow::{Error, Result, anyhow};

#[derive(Debug, Clone)]
pub struct RelocationEntry {
    /// Type of relocation
    pub relocation: Relocation,
    /// The extern symbol
    pub symbol: String,
    /// The amount to add to the value of the symbol
    pub addend: u64,
    /// The section to fixup
    pub section: usize,
    /// Where in that section to do the fixup
    pub offset: usize,
}

fn valid_operation_on_undefined_symbol(op: BinaryOp) -> bool {
    matches!(op, BinaryOp::Add | BinaryOp::Sub)
}

///
/// Parses `expr` and returns a tuple in the format of (symbol, addend) where symbol is an
/// either an undefined symbol, or a label in a section seperate from the one the expression comes from,
/// and addend is a constant value to be added to the symbol once the value is
/// resolved
///
///
/// # Arguments
/// * `assembler` - The assembler the expression comes from
/// * `section` - The section index the expression comes from
/// * `expr` - The expression to evalute
///
///
/// # Errors
/// Returns an error if the expression does an operation between two undefined symbols, or two labels in two
/// different sections
///
fn evaluate_expression(
    assembler: &Assembler,
    section: usize,
    expr: &Box<Node>,
) -> Result<(String, u64)> {
    let result = match &**expr {
        // // This should have errored out earlier in the code
        // Node::Register(_) => unreachable!(),

        // The values of registers are irrelevant, we only care about constant displacements
        Node::Register(_) => (String::new(), 0),

        Node::Constant(value) => (String::new(), *value),
        Node::Identifier(symbol) => match assembler.symbols.get_symbol(&symbol) {
            Some(value) => {
                // The symbol is label, otherwise it's a constant value
                if let Some(_) = value.section_index {
                    // The label's address is unknown
                    (symbol.clone(), 0)
                } else {
                    // The symbol is a constant and is valid in any context
                    (String::new(), value.value)
                }
            }
            None => (symbol.clone(), 0),
        },
        Node::BinaryOp { op, left, right } => {
            let (left_symbol, mut left_addend) = evaluate_expression(assembler, section, left)?;
            let (right_symbol, right_addend) = evaluate_expression(assembler, section, right)?;

            let symbol = if left_symbol.is_empty() && right_symbol.is_empty() {
                String::new()
            } else if !left_symbol.is_empty() && right_symbol.is_empty() {
                left_symbol
            } else if left_symbol.is_empty() && !right_symbol.is_empty() {
                if *op == BinaryOp::Sub {
                    return Err(anyhow!("Cannot subtract a relocatable symbol"))
                }
                right_symbol
            }
            /* If both symbols are offsets into the same section that we can calculate their difference */
            else if *op == BinaryOp::Sub
                && !left_symbol.is_empty()
                && !right_symbol.is_empty()
                && let Some(left) = assembler.symbols.get_symbol(&left_symbol)
                && let Some(right) = assembler.symbols.get_symbol(&right_symbol)
                && let Some(left_section) = left.section_index
                && let Some(right_section) = right.section_index
                && left_section == right_section
                && left_section == section
            {
                left_addend = left_addend.wrapping_add(left.value.wrapping_sub(right.value));
                String::new()
            } else {
                return Err(anyhow!("Failed to create relocation"));
            };

            if !symbol.is_empty() && *op != BinaryOp::Add {
                return Err(anyhow!("Invalid operation on relocatable symbol"));
            }

            let new_addend = op.calculate(left_addend, right_addend);
            (symbol, new_addend)
        }
        Node::UnaryOp { op, expr } => {
            let (symbol, addend) = evaluate_expression(assembler, section, expr)?;

            if !symbol.is_empty() {
                return Err(anyhow!(
                    "Cannot perform a unary operation on an undefined symbol"
                ));
            }

            let new_addend = op.calculate(addend);

            (String::new(), new_addend)
        }
        Node::Expression(expr) => evaluate_expression(assembler, section, expr)?,
    };

    Ok(result)
}

pub struct Module {
    pub filename: String,
    pub symbols: SymbolTable,
    pub global_symbols: Vec<String>,

    pub relocations: Vec<RelocationEntry>,
    pub sections: Vec<Section>,
    pub section_map: HashMap<String, usize>,
}

impl TryFrom<Assembler> for Module {
    type Error = anyhow::Error;
    fn try_from(value: Assembler) -> Result<Self, Error> {
        let mut relocations = Vec::new();

        // All global symbols must be actual symbols within the module
        for symbol in value.global_symbols.iter() {
            if value.symbols.get_symbol(symbol).is_none() {
                return Err(anyhow!(
                    "in {}:\n\tGlobal symbol {} has no definition",
                    value.filename,
                    symbol
                ));
            }
        }

        for forward_reference in value.forward_references.iter() {
            let (symbol, addend) = match evaluate_expression(
                &value,
                forward_reference.section,
                &forward_reference.expr,
            ) {
                Ok(result) => result,
                Err(e) => {
                    return Err(anyhow!(
                        "{}:{} {e}",
                        value.filename,
                        forward_reference.line_number
                    ));
                }
            };

            let relocation = RelocationEntry {
                relocation: forward_reference.relocation,
                symbol,
                addend,
                section: forward_reference.section,
                offset: forward_reference.offset,
            };

            relocations.push(relocation);
        }

        Ok(Self {
            filename: value.filename,
            symbols: value.symbols,
            global_symbols: value.global_symbols,

            relocations,
            sections: value.sections,
            section_map: value.section_map,
        })
    }
}
