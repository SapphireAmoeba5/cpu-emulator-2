use anyhow::{Context, Error, Result, anyhow};
use spdlog::debug;
use std::collections::{HashMap, hash_map::Entry};

use crate::{
    assembler::{
        Assembler, calculate_disp32_offset,
        symbol_table::{self, Symbol, SymbolTable, Type},
    },
    module::{self, Module},
    opcode::Relocation,
    section::{self, Section},
};

pub enum Instr {
    // A specific section
    Section(String),
    // All sections not yet placed
    GlobSection,
}

fn linker_error(failed: &mut bool, filename: &str, section: &str, offset: usize, message: String) {
    *failed = true;
    println!("{filename} {section}:+{offset:#x}:\n\t{message}");
}

pub fn replace_bytes(dest: &mut Vec<u8>, offset: usize, bytes: &[u8]) {
    let count = bytes.len();
    let copy = &mut dest[offset..offset + count];
    copy.copy_from_slice(bytes);
}

pub struct Global {
    /// The module this symbol belongs to
    module: usize,
    /// The actual global symbol
    symbol: Symbol,
}

pub struct Program {
    /// The modules that make up this program
    modules: Vec<Module>,
    /// The global symbols inside the program
    globals: HashMap<String, Global>,
    /// The final, linked program
    pub linked: Vec<u8>,
    /// `section_offset[i][y]` is the offset of the y'th section in the list of sections of the i'th
    /// module in the `modules` array relative to the final linked program
    pub section_offset: Vec<Vec<usize>>,
    /// `section_included[i][y]` is a flag for if the y'th section in the list of sections of the
    /// i'th module in the `modules` array has been included in the final program
    pub section_included: Vec<Vec<bool>>,
}

pub fn link(modules: Vec<Module>, script: Vec<Instr>) -> Result<Program, ()> {
    let mut failed = false;

    let mut linked: Vec<u8> = Vec::new();
    let mut globals: HashMap<String, Global> = HashMap::new();
    let mut section_offset: Vec<Vec<usize>> = vec![Vec::new(); modules.len()];
    let mut section_included: Vec<Vec<bool>> = vec![Vec::new(); modules.len()];

    // Fill everything with default value so we don't need to check if indices exist later
    for (module_idx, module) in modules.iter().enumerate() {
        section_offset[module_idx].resize(module.sections.len(), 0);
        section_included[module_idx].resize(module.sections.len(), false);

        for global in module.global_symbols.iter() {
            // If the symbol is registered as global then the symbol should exist in the symbol
            // table
            let symbol = module.symbols.get_symbol(global).unwrap();
            let symbol = Global {
                module: module_idx,
                symbol,
            };
            globals.insert(global.clone(), symbol);
        }
    }

    // This stores a list of all module and section indexes for each section
    let mut section_map: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for (module_idx, module) in modules.iter().enumerate() {
        for (section_idx, section) in module.sections.iter().enumerate() {
            match section_map.entry(section.name.clone()) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push((module_idx, section_idx));
                }
                Entry::Vacant(entry) => {
                    let vec = vec![(module_idx, section_idx)];
                    entry.insert(vec);
                }
            }
        }
    }

    // TODO: Make the linker_error function more ergonomic to use
    for instr in &script {
        match instr {
            Instr::Section(section) => {
                // Glob all remaining sections
                if section == "*" {
                    for (_, value) in section_map.iter() {
                        for (module_idx, section_idx) in value.iter() {
                            let alignment = modules[*module_idx].sections[*section_idx].alignment;
                            add_section(
                                &mut linked,
                                &mut section_offset.as_mut_slice(),
                                section_included.as_mut_slice(),
                                &modules,
                                *module_idx,
                                *section_idx,
                                alignment,
                            );
                        }
                    }
                } else {
                    if let Some(sections) = section_map.get(section) {
                        for (module_idx, section_idx) in sections.iter() {
                            let alignment = modules[*module_idx].sections[*section_idx].alignment;
                            add_section(
                                &mut linked,
                                section_offset.as_mut_slice(),
                                section_included.as_mut_slice(),
                                &modules,
                                *module_idx,
                                *section_idx,
                                alignment,
                            );
                        }
                    }
                }
            }

            _ => todo!(),
        }
    }

    for (module_idx, module) in modules.iter().enumerate() {
        for relocation in module.relocations.iter() {
            let section_name = &module.sections[relocation.section].name;
            let relocation_offset =
                section_offset[module_idx][relocation.section] + relocation.offset;

            let (value, type_) = if let Some(symbol) = module.symbols.get_symbol(&relocation.symbol)
            {
                let value = if let Some(section) = symbol.section_index {
                    // TODO: Handle the case where the section won't be included in the final
                    // program
                    let offset: u64 = section_offset[module_idx][section].try_into().unwrap();
                    symbol.value + offset
                } else {
                    symbol.value
                };

                (value, symbol.type_)
            } else if let Some(global) = globals.get(&relocation.symbol) {
                let value = if let Some(section) = global.symbol.section_index {
                    // TODO: Handle the case where the section won't be included in the final
                    // program
                    let offset: u64 = section_offset[global.module][section].try_into().unwrap();
                    global.symbol.value + offset
                } else {
                    global.symbol.value
                };
                (value, global.symbol.type_)
            } else {
                linker_error(
                    &mut failed,
                    &module.filename,
                    section_name,
                    relocation_offset,
                    format!("Undefined symbol {}", relocation.symbol),
                );
                continue;
            };

            match relocation.relocation {
                Relocation::PC32 => {
                    debug!(
                        "PC32 relocation at {} {section_name}:+{:#x}",
                        module.filename, relocation_offset
                    );
                    if type_ != Type::Label {
                        linker_error(
                            &mut failed,
                            &module.filename,
                            section_name,
                            relocation_offset,
                            format!("Attempting to perform a PC32 relocation on a {type_}"),
                        );
                        continue;
                    }
                    let pc = (relocation_offset + 4).try_into().unwrap();

                    let offset = match calculate_disp32_offset(pc, value) {
                        Ok(offset) => offset,
                        Err(e) => {
                            linker_error(
                                &mut failed,
                                &module.filename,
                                section_name,
                                relocation_offset,
                                format!("{e}"),
                            );
                            continue;
                        }
                    };

                    debug!(
                        "Fixup at {} {section_name}:{relocation_offset} to {offset}",
                        module.filename
                    );
                    replace_bytes(&mut linked, relocation_offset, &offset.to_le_bytes());
                }
                Relocation::Abs64 => {
                    if type_ != Type::Constant {
                        linker_error(
                            &mut failed,
                            &module.filename,
                            section_name,
                            relocation_offset,
                            format!("ABS64 relocation on a {type_}"),
                        );
                        continue;
                    }

                    debug!(
                        "Fixup at {} {section_name}:{relocation_offset} to {value}",
                        module.filename
                    );
                    replace_bytes(&mut linked, relocation_offset, &value.to_le_bytes());
                }
                Relocation::Abs32 => {
                    if type_ != Type::Constant {
                        linker_error(
                            &mut failed,
                            &module.filename,
                            section_name,
                            relocation_offset,
                            format!("ABS64 relocation on a {type_}"),
                        );
                        continue;
                    }

                    if let Ok(value) = u32::try_from(value) {
                        debug!(
                            "Fixup at {} {section_name}:{relocation_offset} to {value}",
                            module.filename
                        );
                        replace_bytes(&mut linked, relocation_offset, &value.to_le_bytes());
                    } else {
                        linker_error(
                            &mut failed,
                            &module.filename,
                            section_name,
                            relocation_offset,
                            format!("Relocated value ({}) out of bounds for ABS32", value),
                        );
                        continue;
                    }
                }
                Relocation::Abs32S => {
                    if type_ != Type::Constant {
                        linker_error(
                            &mut failed,
                            &module.filename,
                            section_name,
                            relocation_offset,
                            format!("ABS64 relocation on a {type_}"),
                        );
                        continue;
                    }

                    if let Ok(value) = i32::try_from(value as i64) {
                        debug!(
                            "ABS32S Fixup at {} {section_name}:{relocation_offset} to {value}",
                            module.filename
                        );
                        replace_bytes(&mut linked, relocation_offset, &value.to_le_bytes());
                    } else {
                        linker_error(
                            &mut failed,
                            &module.filename,
                            section_name,
                            relocation_offset,
                            format!("Relocated value ({}) out of bounds for ABS32", value),
                        );
                        continue;
                    }
                }
                Relocation::Addr64 => {
                    if type_ != Type::Constant {
                        linker_error(
                            &mut failed,
                            &module.filename,
                            section_name,
                            relocation_offset,
                            "ADDR64 relocation on a constant".to_string(),
                        );
                        continue;
                    }
                }

                // TODO: Implement the other fixup types
                _ => todo!(),
            }
        }
    }

    if !failed {
        Ok(Program {
            // Initialize `modules` with a filler for now to prevent issues with the borrow checker
            modules,
            globals,
            linked,
            section_offset,
            section_included,
        })
    } else {
        Err(())
    }
}

fn add_section(
    linked: &mut Vec<u8>,
    section_offset: &mut [Vec<usize>],
    section_included: &mut [Vec<bool>],
    modules: &[Module],
    module: usize,
    section: usize,
    alignment: u64,
) {
    // Skip already included section
    if section_included[module][section] {
        debug!(
            "Section {} in {} was already added",
            modules[module].sections[section].name, modules[module].filename
        );
        return;
    }

    debug!(
        "Adding {} in {} to the final program",
        modules[module].sections[section].name, modules[module].filename
    );

    let alignment: usize = alignment.try_into().unwrap();
    let padding = (alignment - (linked.len() % alignment)) % alignment;

    linked.resize(linked.len() + padding, 0);

    let offset = linked.len();

    section_included[module][section] = true;
    section_offset[module][section] = offset;

    let section = modules[module].sections[section].data.as_slice();
    linked.extend_from_slice(section);
}
