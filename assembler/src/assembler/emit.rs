use core::hash;

use crate::{
    assembler::{Assembler, Instruction, Operand},
    encoding,
    opcode::{EncodingFlags, OperandFlags, Relocation},
    operand,
    section::Section,
    tokens::Register,
};
use anyhow::{Context, Result, anyhow};
use spdlog::debug;

const EXTENSION_BYTE: u8 = 0x0f;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum Size {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
}

fn reg_transfer_byte(dest: Register, src: Register) -> u8 {
    /*
     *  reg/reg transfer byte encoding
     *     bit:   7 6 5 4   3 2 1 0
     * purpose:   dst reg | src reg
     */
    dest.index() << 4 | src.index()
}

// The imm/mem transfer byte encoding layout
/*
 *
 *      bit:  7 6 5 4       3 2       1 0
 * purpose:   dst reg |  addr mode | size
 *
 * Values for `addr mode`
 *   0b00 | PC rel disp32
 *   0b01 | SP + Index * scale
 *   0b10 | Base + Index * scale
 *   0b11 | Immediate value (Doesn't reference memory)
 */

fn imm_transfer_byte(dest: Register, size: Size) -> u8 {
    /*
     *                 Byte layout
     *     bit:   7 6 5 4    3 2       1 0
     * purpose:    dest  | addr mode | size
     */
    dest.index() << 4 | 0b11 << 2 | (size as u8)
}

fn disp_transfer_byte(dest: Register, size: Size) -> u8 {
    dest.index() << 4 | 0b00 << 2 | (size as u8)
}

fn sp_rel_transfer_byte(dest: Register, size: Size) -> u8 {
    dest.index() << 4 | 0b01 | (size as u8)
}

fn base_index_transfer_byte(dest: Register, size: Size) -> u8 {
    dest.index() << 4 | 0b10 | (size as u8)
}

fn immediate_fits(src: u64, options: OperandFlags) -> bool {
    if options.intersects(OperandFlags::IMM64) {
        // Useless but included just to have it
        src <= u64::MAX
    } else if options.intersects(OperandFlags::IMM32 | OperandFlags::DISP32) {
        src <= u32::MAX.into()
    } else if options.intersects(OperandFlags::IMM8) {
        src <= u8::MAX.into()
    } else {
        unreachable!()
    }
}

/// Get's the size of the `value`
/// # Return
/// Returns the smallest unsigned integer type (u8, u16, u32, u64) the value can fit in
/// respectively
fn get_size(value: u64) -> Size {
    if value <= u8::MAX.into() {
        Size::U8
    } else if value <= u16::MAX.into() {
        Size::U16
    } else if value <= u32::MAX.into() {
        Size::U32
    } else if value <= u64::MAX {
        Size::U64
    } else {
        unreachable!()
    }
}

fn get_size_from_relocation(reloc: Relocation) -> Size {
    match reloc {
        Relocation::Abs64 | Relocation::PC64 => Size::U64,
        Relocation::Abs32 | Relocation::PC32 => Size::U32,
        Relocation::Abs16 => Size::U16,
        Relocation::Abs8 | Relocation::PC8 => Size::U8,
        Relocation::None => unreachable!(),
    }
}

fn get_memory_access_size(flags: EncodingFlags) -> Size {
    if flags.intersects(EncodingFlags::MEM64) {
        Size::U64
    } else if flags.intersects(EncodingFlags::MEM32) {
        Size::U32
    } else if flags.intersects(EncodingFlags::MEM16) {
        Size::U16
    } else if flags.intersects(EncodingFlags::MEM8) {
        Size::U8
    } else {
        unreachable!()
    }
}

/// # Error
/// Returns Err if the displacement doesn't fit in an i32
///
/// # Arguments
/// * `pc` - The source address
/// * `addr` - The destination address
///
/// # Return
/// The offset of `addr` relative to `pc`
pub fn calculate_disp32_offset(pc: u64, addr: u64) -> Result<i32> {
    (addr.wrapping_sub(pc) as i64)
        .try_into()
        .context("Displacement is too large to fit in 4 bytes")
}

impl Assembler {
    fn get_section(&mut self) -> &mut Section {
        self.get_section_mut().unwrap()
    }

    /// Emits `instruction` into the current section's buffer
    ///
    /// `instruction.types[i]` should have only one bit set for each `i` up to
    /// `instruction.operand_count`
    pub(super) fn emit_instruction(&mut self, mut instruction: Instruction) -> Result<usize> {
        let options = instruction.encoding.options;

        // Make sure we are inside a section before continuing
        _ = self.get_section_mut()?;

        // Used for getting the current size of the instruction
        let start = self.get_section().cursor();

        if instruction.encoding.extension {
            self.get_section().write_u8(EXTENSION_BYTE);
        }

        self.get_section().write_u8(instruction.encoding.opcode);

        if options.intersects(EncodingFlags::DATA_TRANSFER) {
            // Maximum of two operands for any of these instructions
            debug_assert_eq!(instruction.operand_count, 2);

            if instruction.types[1].intersects(OperandFlags::REG) {
                let dest = instruction.operands[0].register();
                let src = instruction.operands[1].register();

                let transfer_byte = reg_transfer_byte(dest, src);
                self.get_section().write_u8(transfer_byte);
            } else if instruction.types[1].intersects(OperandFlags::IMM) {
                // Two operands that are a register, and an immediate are garunteed
                let dest = instruction.operands[0].register();
                let src = instruction.operands[1].constant();

                if instruction.reloc[1] != Relocation::None {
                    // Add plus one to the offset to account for the transfer byte we haven't
                    // written yet
                    let offset = self.get_section().cursor() + 1;
                    let expr = std::mem::replace(&mut instruction.exprs[1], None);
                    // Emit the relocation
                    self.emit_relocation(instruction.reloc[1], offset, expr.unwrap());
                }

                if options.intersects(encoding!(IMM_AS_ADDR)) {
                    let transfer_byte = imm_transfer_byte(dest, Size::U64);
                    self.get_section().write_u8(transfer_byte);

                    self.get_section().write_u64(src);
                } else if instruction.types[1].intersects(OperandFlags::IMM64) {

                    let transfer_byte = imm_transfer_byte(dest, Size::U64);
                    self.get_section().write_u8(transfer_byte);

                    self.get_section().write_u64(src);
                }
            } else if instruction.types[1].intersects(OperandFlags::DISP32) {
                let dest = instruction.operands[0].register();
                let disp = instruction.operands[1].constant();

                let memory_access_size = get_memory_access_size(options);
                let transfer_byte = disp_transfer_byte(dest, memory_access_size);

                self.get_section().write_u8(transfer_byte);

                let offset = if instruction.reloc[1] == Relocation::None {
                    // Where the program counter will be when this instruction is executed
                    let pc: u64 = (self.get_section().cursor() + 4).try_into().unwrap();

                    let offset = calculate_disp32_offset(pc, disp)?;

                    debug!(
                        "Calculated offset {:#x} to {}+{:#x}",
                        offset,
                        self.get_section().name,
                        pc as i64 + offset as i64
                    );

                    offset
                } else {
                    // `expr` should always be Some
                    let expr = std::mem::replace(&mut instruction.exprs[1], None).unwrap();
                    let cursor = self.get_section().cursor();
                    self.emit_relocation(instruction.reloc[1], cursor, expr);
                    0
                };

                self.get_section().write_u32(offset as u32);
            } else {
                unreachable!()
            }
        } else if options.intersects(encoding!(SYS_CONTROL)) {
            if instruction.operand_count == 1 {
                if instruction.types[0].intersects(operand!(IMM8)) {
                    let byte: u8 = instruction.operands[0]
                        .constant()
                        .try_into()
                        .context("Constant too large to fit in one byte")?;

                    if instruction.reloc[0] != Relocation::None {
                        let offset = self.get_section().cursor();
                        let expr = std::mem::replace(&mut instruction.exprs[0], None);
                        self.emit_relocation(instruction.reloc[0], offset, expr.unwrap());
                    }

                    self.get_section().write_u8(byte);
                } else {
                    unreachable!("Invalid instruction template")
                }
            }
        } else {
            panic!("Invalid instruction")
        }

        let position = self.get_section().cursor();

        let size = self.get_section().cursor() - start;
        debug!("Instruction size: {size}");

        // Instructions can't be bigger than 16 bytes
        debug_assert!(size <= 16);

        Ok(size)
    }
}
