use std::num::NonZero;

use crate::instruction::{Instruction, Operand};


#[derive(Debug)]
pub struct Section {
    /// The name of the section
    pub name: String,
    /// The alignment this section requires
    pub alignment: u64,
    pub data: Vec<u8>,
    // pub section_data: Vec<SectionEntry>,
}

impl Section {
    pub fn new(name: String) -> Self {
        Self {
            name,
            alignment: 1,
            data: Vec::new(),
        }
    }

    pub fn replace_bytes(&mut self, offset: usize, bytes: &[u8]) {
        let count = bytes.len();
        let copy = &mut self.data[offset..offset + count];
        copy.copy_from_slice(bytes);
    }

    pub fn write_u8(&mut self, byte: u8) {
        self.data.push(byte);
    }

    pub fn write_u16(&mut self, byte: u16) {
        self.write_bytes(&byte.to_le_bytes());
    }

    pub fn write_u32(&mut self, byte: u32) {
        self.write_bytes(&byte.to_le_bytes());
    }

    pub fn write_u64(&mut self, byte: u64) {
        self.write_bytes(&byte.to_le_bytes());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Get's the current size of the section in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Gets the current byte position of the
    pub fn cursor(&self) -> usize {
        self.size()
    }
}
