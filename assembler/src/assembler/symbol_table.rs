use std::{collections::HashMap, rc::Rc};

#[must_use]
pub enum Error {
    CircularDefinition,
    AlreadyDefined,
}

enum Symbol {
    Constant(u64),
    Symbol(Rc<str>),
}
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn insert_id(&mut self, id: String, symbol: String) -> Result<(), Error> {



        Err(Error::CircularDefinition)
    } 
}
