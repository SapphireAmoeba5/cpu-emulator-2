use std::{
    collections::{HashMap, hash_map::Entry},
    num::NonZero,
    ops,
    rc::Rc,
};

use anyhow::{Result, anyhow, bail};

use crate::instruction::{Instruction, Operand};

#[derive(Debug)]
pub struct Section {
    /// The name of the section
    pub name: Rc<str>,
    /// The alignment this section requires
    pub alignment: u64,
    pub data: Vec<u8>,
    // pub section_data: Vec<SectionEntry>,
}

impl Section {
    fn new(name: Rc<str>) -> Self {
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

#[derive(Debug)]
pub struct SectionMap {
    sections: Vec<Section>,
    section_map: HashMap<Rc<str>, usize>,
    current_section: Option<usize>,
}

impl std::ops::Index<usize> for SectionMap {
    type Output = Section;
    fn index(&self, index: usize) -> &Self::Output {
        &self.sections[index]
    }
}

impl std::ops::Index<&str> for SectionMap {
    type Output = Section;
    #[track_caller]
    fn index(&self, index: &str) -> &Self::Output {
        let (_, section) = self.get(index).expect("No such section");
        section
    }
}

impl SectionMap {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            section_map: HashMap::new(),
            current_section: None,
        }
    }

    pub fn set_section(&mut self, name: impl Into<Rc<str>>) {
        let name: Rc<str> = name.into();
        match self.section_map.entry(name.clone()) {
            Entry::Occupied(entry) => self.current_section = Some(*entry.get()),
            Entry::Vacant(entry) => {
                let section = Section::new(name);
                let index = self.sections.len();
                self.current_section = Some(index);
                self.sections.push(section);
                entry.insert(index);
            }
        }
    }

    /// Returns a tuple containing the section id and a mutable reference to the section last set with `set_section`
    ///
    /// # Errors
    /// Returns Err if there is no current section (I.E `set_section` wasn't called yet)
    pub fn get_section(&mut self) -> Result<(usize, &mut Section)> {
        match self.current_section {
            Some(current) => Ok((current, &mut self.sections[current])),
            None => bail!(
                "Section to place data not defined. Try doing .section {{section_name}} before your code"
            ),
        }
    }

    pub fn get(&self, section: impl AsRef<str>) -> Option<(usize, &Section)> {
        let section_id = *self.section_map.get(section.as_ref())?;
        let section = self.sections.get(section_id)?;
        Some((section_id, section))
    }

    /// Gets the number of sections
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    pub fn iter(&self) -> SectionIter<'_> {
        SectionIter::new(&self.sections)
    }
}

pub struct SectionIter<'a> {
    sections: &'a [Section],
    cur: usize,
}

impl<'a> SectionIter<'a> {
    pub fn new(sections: &'a impl AsRef<[Section]>) -> Self {
        Self {
            sections: sections.as_ref(),
            cur: 0,
        }
    }
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = &'a Section;

    /// TODO: Should this iterator return a tuple of (usize, &Section) where usize is the section
    /// id, or is it better to make `enumerate` the way to get the same tuple
    fn next(&mut self) -> Option<Self::Item> {
        match self.sections.get(self.cur) {
            Some(section) => {
                self.cur += 1;
                Some(section)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.sections.len() - self.cur;
        (remaining, Some(remaining))
    }
}
