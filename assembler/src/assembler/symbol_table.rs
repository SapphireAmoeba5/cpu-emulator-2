use std::collections::{hash_map::Entry, HashMap};

use anyhow::{anyhow, Result};


#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    pub section_index: Option<usize>,
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
    pub fn insert_symbol(&mut self, id: String, value: u64, section: Option<usize>) -> Result<()> {
        match self.symbols.entry(id) {
            Entry::Vacant(vacant) => {
                vacant.insert(Symbol {
                    section_index: section,
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
