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
        // This should have errored out earlier in the code
        Node::Register(_) => unreachable!(),

        Node::Constant(value) => (String::new(), *value),
        Node::Identifier(symbol) => match assembler.symbols.get_symbol(&symbol) {
            Some(value) => {
                // The symbol is label, otherwise it's a constant value
                if let Some(section_idx) = value.section_index {
                    if section_idx == section {
                        // The label's section and the expression's section are the same so we can
                        // use the label's offset.
                        // Offsets within the same section and the same Assembler unit remain fixed
                        // after linking.
                        (String::new(), value.value)
                    } else {
                        // We can't use the label's offset since it is relative to a different
                        // section
                        (symbol.clone(), 0)
                    }
                } else {
                    // The symbol is a constant and is valid in any context
                    (String::new(), value.value)
                }
            }
            None => (symbol.clone(), 0),
        },
        Node::BinaryOp { op, left, right } => {
            let (left_symbol, left_addend) = evaluate_expression(assembler, section, left)?;
            let (right_symbol, right_addend) = evaluate_expression(assembler, section, right)?;

            let symbol = if left_symbol.is_empty() && right_symbol.is_empty() {
                String::new()
            } else if !left_symbol.is_empty() && right_symbol.is_empty() {
                left_symbol
            } else if left_symbol.is_empty() && !right_symbol.is_empty() {
                right_symbol
            } else {
                return Err(anyhow!(
                    "Cannot perform an operation on two undefined symbols"
                ));
            };

            let new_addend = op.calculate(left_addend, right_addend);
            if symbol.is_empty() {
                (String::new(), new_addend)
            } else if valid_operation_on_undefined_symbol(*op) {
                (symbol, new_addend)
            } else {
                return Err(anyhow!("Invalid operation on an undefined symbol"));
            }
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
                return Err(anyhow!("in {}:\n\tGlobal symbol {} has no definition", value.filename, symbol))
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
