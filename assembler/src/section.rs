use std::{
    collections::{HashMap, hash_map::Entry},
    io::{Cursor, Write},
    num::NonZero,
    ops,
    rc::Rc,
};

use anyhow::{Context, Result, anyhow, bail};

use crate::{
    instruction::{Instruction, Operand},
    section,
};

#[derive(Debug)]
pub struct Section {
    /// The name of the section
    pub name: Rc<str>,
    /// The alignment this section requires
    pub alignment: u64,
    pub data: Cursor<Vec<u8>>,
    // pub section_data: Vec<SectionEntry>,
}

impl Section {
    fn new(name: Rc<str>) -> Self {
        Self {
            name,
            alignment: 1,
            data: Cursor::new(Vec::new()),
        }
    }

    pub fn replace_bytes(&mut self, offset: usize, bytes: &[u8]) {
        let old_position = self.data.position();
        self.data
            .set_position(offset.try_into().expect("Offset is too large"));
        // Garunteed to return Ok
        _ = self.data.write(bytes);
        self.data.set_position(old_position);
    }

    pub fn align(&mut self, align: u64) {
        if align > self.alignment {
            self.alignment = align;
        }

        let cursor: u64 = self.cursor().try_into().expect("Value too large");

        let n = (align - (cursor % align)) % align;
        self.data.set_position(cursor + n);
    }

    pub fn write_u8(&mut self, byte: u8) {
        _ = self.data.write(&byte.to_le_bytes());
    }

    pub fn write_u16(&mut self, byte: u16) {
        _ = self.data.write(&byte.to_le_bytes());
    }

    pub fn write_u32(&mut self, byte: u32) {
        _ = self.data.write(&byte.to_le_bytes());
    }

    pub fn write_u64(&mut self, byte: u64) {
        _ = self.data.write(&byte.to_le_bytes());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        _ = self.data.write(bytes);
    }

    /// Get's the current size of the section in bytes
    pub fn size(&self) -> usize {
        self.data.get_ref().len()
    }

    /// Gets the current byte position of the
    pub fn cursor(&self) -> usize {
        self.data.position().try_into().expect("Value too large")
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
    const ERROR_MESSAGE: &str =
        "Section to place data not defined. Try doing .section {section_name} before your code";
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
    pub fn get_section_mut(&mut self) -> Result<(usize, &mut Section)> {
        match self.current_section {
            Some(current) => Ok((current, &mut self.sections[current])),
            None => bail!("{}", Self::ERROR_MESSAGE),
        }
    }

    /// Returns a tuple containing the section id and an immutable reference to the section last set with `set_section`
    ///
    /// # Errors
    /// Returns Err if there is no current section (I.E `set_section` wasn't called yet)
    pub fn get_section(&self) -> Result<(usize, &Section)> {
        match self.current_section {
            Some(current) => Ok((current, &self.sections[current])),
            None => bail!("{}", Self::ERROR_MESSAGE),
        }
    }

    pub fn get(&self, section: impl AsRef<str>) -> Option<(usize, &Section)> {
        let section_id = *self.section_map.get(section.as_ref())?;
        let section = self.sections.get(section_id)?;
        Some((section_id, section))
    }

    /// Returns a tuple containing the ID of the current section, and the current cursor position
    /// in that section
    ///
    /// # Errors
    /// Returns Err if there was no section defined using `set_section`
    pub fn cursor(&self) -> Result<(usize, usize)> {
        match self.current_section {
            Some(current) => Ok((current, self.sections[current].cursor())),
            None => bail!("{}", Self::ERROR_MESSAGE),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        let mut section = Section::new(Rc::from("Test section"));

        section.write_u8(1);
        section.align(8);

        section.write_u64(0xabababababababab);

        assert_eq!(section.data.get_ref(), &[1, 0, 0, 0, 0, 0, 0, 0, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab]);

        let mut section = Section::new(Rc::from("Test section"));

        section.write_u8(1);
        section.align(16);

        section.write_u64(0xabababababababab);

        assert_eq!(section.data.get_ref(), &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab]);
    }
}
