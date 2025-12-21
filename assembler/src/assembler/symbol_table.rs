use core::fmt;
use std::{collections::{HashMap, hash_map::Entry}, ffi::os_str::Display};

use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type {
    Label,
    Constant,
}

impl Type {
    fn as_str(&self) -> &'static str {
        match self {
            Type::Label => "Label",
            Type::Constant => "Constant"
        } 
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str()) 
     } 
}

#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    pub section_index: Option<usize>,
    pub type_: Type,
    pub value: u64,
}

#[derive(Debug)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
    pub fn insert_symbol(&mut self, id: String, value: u64, type_: Type, section: Option<usize>) -> Result<()> {
        match self.symbols.entry(id) {
            Entry::Vacant(vacant) => {
                vacant.insert(Symbol {
                    section_index: section,
                    type_,
                    value,
                });
                Ok(())
            }
            Entry::Occupied(_) => Err(anyhow!("Symbol already defined")),
        }
    }

    pub fn get_symbol(&self, id: &str) -> Option<Symbol> {
        match self.symbols.get(id) {
            Some(value) => Some(*value),
            None => None,
        }
    }
}
