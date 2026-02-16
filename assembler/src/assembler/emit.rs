use crate::{
    assembler::{Assembler, Instruction, Operand},
    bit, encoding,
    opcode::{EncodingFlags, OperandFlags, Relocation},
    operand,
    section::Section,
    tokens::Register,
};
use anyhow::{Context, Result, anyhow};
use spdlog::debug;
use std::{
    mem::{self, size_of},
    ops::Index,
};

struct GPRegister(u8);

impl GPRegister {
    pub fn get_gp(&self) -> u8 {
        // This value is garunteed to be a valid index for any GP register
        self.0
    }
}

impl TryFrom<Register> for GPRegister {
    type Error = anyhow::Error;

    fn try_from(value: Register) -> std::result::Result<Self, Self::Error> {
        if let Some(index) = value.get_gp() {
            Ok(Self(index))
        } else {
            let as_str: &str = value.into();
            Err(anyhow!(
                "{as_str} is not a general purpose register (r0 -> r15)"
            ))
        }
    }
}

pub const EXTENSION_BYTE: u8 = 0x0f;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum Size {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
}

impl From<Relocation> for Size {
    fn from(value: Relocation) -> Self {
        match value {
            Relocation::Abs8 | Relocation::Abs8S | Relocation::PC8 => Size::U8,
            Relocation::Abs16 | Relocation::Abs16S => Size::U16,
            Relocation::Abs32 | Relocation::Abs32S | Relocation::PC32 => Size::U32,
            Relocation::Abs64 | Relocation::Abs64S | Relocation::PC64 => Size::U64,
            Relocation::None => unreachable!("This shouldn't be reached"),
        }
    }
}

fn reg_transfer_byte(dest: GPRegister, src: GPRegister) -> u8 {
    /*
     *  reg/reg transfer byte encoding
     *     bit:   7 6 5 4   3 2 1 0
     * purpose:   dst reg | src reg
     */
    dest.get_gp() << 4 | src.get_gp()
}

fn imm_transfer_byte(dest: GPRegister, size: Size) -> u8 {
    /*
     *                 Byte layout
     *     bit:   7 6 5 4    3 2    1  0
     * purpose:    dest  | size | reserved (always 0)
     *
     * the `size` field tells the CPU how bytes to read for the immediate
     */

    dest.get_gp() << 4 | (size as u8) << 2
}

// The mem transfer byte encoding layout
/*
 *
 *      bit:  7 6 5 4       3 2       1 0
 * purpose:   dst reg |  addr mode | size
 *
 * Values for `addr mode`
 *   0b00 | PC rel disp32
 *   0b01 | SP + Index * scale
 *   0b10 | Base + Index * scale (BIS)
 *   0b11 | Immediate address
 */

fn disp_transfer_byte(dest: GPRegister, size: Size) -> u8 {
    dest.get_gp() << 4 | 0b00 << 2 | (size as u8)
}

fn sp_rel_transfer_byte(dest: GPRegister, size: Size) -> u8 {
    dest.get_gp() << 4 | 0b01 << 2 | (size as u8)
}

fn base_index_transfer_byte(dest: GPRegister, size: Size) -> u8 {
    dest.get_gp() << 4 | 0b10 << 2 | (size as u8)
}

fn const_addr_transfer_byte(dest: GPRegister, size: Size) -> u8 {
    dest.get_gp() << 4 | 0b11 << 2 | (size as u8)
}

/// Creates the memory index byte used in normal Base + Index * Scale + Displacement,
/// and SP + Index * Scale + Displacement addressing.
///
/// `base_or_index` have different meaning based on if this byte is
/// using the BIS addressing mode, or SP rel addressing mode respectively.
///
/// `disp_width` determines if the displacement is 4bytes or 2bytes. false if four bytes, true if
/// 2bytes
///
/// if `ignore` is set to true then that means to ignore the register encoded within this byte.
/// For BIS addressing it means there is another byte that will encode both the base and index
/// register, while for SP rel addressing it means there is no index register
///
/// if `ignore` is true then the value of `base_or_index` doesn't matter
///
/// This function will panic if any of the parameters are invalid
///
/// `base_or_index` must be a general purpose register, or if `ignore` is true then
/// it can be invalid
///
/// `scale` must be less than or equal to 3
fn memory_index_byte(base_or_index: Register, scale: u8, disp_width: bool, ignore: bool) -> u8 {
    // Scale must be between these two values
    debug_assert!(scale <= 3);

    // TODO: Update documentation for this function

    let reg_byte = if !ignore {
        base_or_index
            .get_gp()
            .expect("Register must be a general purpose register")
    } else {
        0
    };

    reg_byte << 4 | scale << 2 | (disp_width as u8) << 1 | (ignore as u8)
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
    Size::from(reloc)
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
        unreachable!("Unknown memory access size")
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

            if instruction.types[1].intersects(OperandFlags::GP_REG) {
                let dest = instruction.operands[0].register();
                // let dest = GPRegister::try_from(dest)?;

                let src = instruction.operands[1].register();

                let transfer_byte = reg_transfer_byte(dest.try_into()?, src.try_into()?);
                self.get_section().write_u8(transfer_byte);
            } else if instruction.types[1].intersects(OperandFlags::IMM) {
                // Two operands that are a register, and an immediate are garunteed
                let dest = instruction.operands[0].register();
                let src = instruction.operands[1].constant();

                let constant_size = if instruction.reloc[1] {
                    // Add plus one to the offset to account for the transfer byte we haven't
                    // written yet
                    let offset = self.get_section().cursor() + 1;
                    let expr = std::mem::replace(&mut instruction.exprs[1], None);
                    // Emit the relocation
                    self.emit_relocation(Relocation::Abs64, offset, expr.unwrap());
                    Size::U64
                } else {
                    if src <= u8::MAX.into() {
                        Size::U8
                    } else if src <= u16::MAX.into() {
                        Size::U16
                    } else if src <= u32::MAX.into() {
                        Size::U32
                    } else {
                        Size::U64
                    }
                };

                let transfer_byte = imm_transfer_byte(dest.try_into()?, constant_size);
                self.get_section().write_u8(transfer_byte);

                match constant_size {
                    Size::U8 => self.get_section().write_u8(src as u8),
                    Size::U16 => self.get_section().write_u16(src as u16),
                    Size::U32 => self.get_section().write_u32(src as u32),
                    Size::U64 => self.get_section().write_u64(src),
                }
            } else if instruction.types[1].intersects(OperandFlags::ADDR) {
                let dest = instruction.operands[0].register();
                let src = instruction.operands[1].constant();

                if instruction.reloc[1] {
                    // Add one to account for the transfer byte
                    let offset = self.get_section().cursor() + 1;
                    let expr = std::mem::replace(&mut instruction.exprs[1], None);
                    self.emit_relocation(Relocation::Abs64, offset, expr.unwrap());
                }

                let size = get_memory_access_size(options);

                let transfer_byte = const_addr_transfer_byte(dest.try_into()?, size);

                self.get_section().write_u8(transfer_byte);
                self.get_section().write_u64(src);
            } else if instruction.types[1].intersects(OperandFlags::DISP32) {
                let dest = instruction.operands[0].register();
                let disp = instruction.operands[1].constant();

                let memory_access_size = get_memory_access_size(options);
                let transfer_byte = disp_transfer_byte(dest.try_into()?, memory_access_size);

                self.get_section().write_u8(transfer_byte);

                let offset = if !instruction.reloc[1] {
                    // Where the program counter will be when this instruction is executed
                    // We add the size of an i32 because the displacement is encoded as a 4 byte
                    // i32 integer
                    let pc: u64 = (self.get_section().cursor() + size_of::<i32>())
                        .try_into()
                        .unwrap();

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
                    self.emit_relocation(Relocation::PC32, cursor, expr);
                    0
                };

                self.get_section().write_u32(offset as u32);
            } else if instruction.types[1].intersects(OperandFlags::INDEX) {
                let dest = instruction.operands[0].register();
                let mut memory_index = instruction.indexes[1];
                let size = get_memory_access_size(options);

                // Normalize the memory index because there are multiple representations of
                // equivlant operations which is difficult to deal with later on
                if memory_index.index.is_valid() && memory_index.base.is_invalid() {
                    // For this we don't check if the scale is only one because if there is only an
                    // index register with a scale then that is always representable the index
                    // register is the instruction pointer
                    memory_index.base = memory_index.index;
                    memory_index.index = Register::none();
                } else if memory_index.index.is_sp()
                    && memory_index.base.is_gp()
                    && memory_index.scale == 1
                {
                    // The index register cannot be the stack pointer but if the stack pointer is
                    // the index and the scale is one then that's still valid
                    let tmp = memory_index.base;
                    memory_index.base = memory_index.index;
                    memory_index.index = tmp;
                }

                let scale = match memory_index.scale {
                    1 => 0,
                    2 => 1,
                    4 => 2,
                    8 => 3,
                    _ if memory_index.index.is_valid() => return Err(anyhow!("Invalid scale")),
                    _ => 0,
                };

                // TODO: Make new relocation type for sign extended values

                // Stack pointer based addressing.
                if memory_index.base.is_sp() {
                    let trsnfr = sp_rel_transfer_byte(dest.try_into()?, size);
                    self.get_section().write_u8(trsnfr);

                    // We don't write this byte right after this statement because the 4byte/2byte
                    // flag hasn't been set to the correct value until we figure out the minimum size of
                    // the displacement
                    let mut sp_byte: u8 = if memory_index.index.is_gp() {
                        let byte = memory_index_byte(memory_index.index, scale, false, false);

                        byte
                    } else {
                        if memory_index.index.is_valid() && !memory_index.index.is_gp() {
                            return Err(anyhow!(
                                "Index register must be a general purpose register"
                            ));
                        }
                        let byte = memory_index_byte(Register::none(), scale, false, true);

                        byte
                    };

                    if !instruction.reloc[1]
                        && let Ok(disp) = i16::try_from(memory_index.disp as i64)
                    {
                        // We set this bit to one to signal to the CPU that this instruction has a
                        // two byte displacement
                        sp_byte |= bit!(1);
                        self.get_section().write_u8(sp_byte);

                        self.get_section().write_u16(disp as u16);
                    } else if let Ok(disp) = i32::try_from(memory_index.disp as i64) {
                        self.get_section().write_u8(sp_byte);
                        if instruction.reloc[1] {
                            let offset = self.get_section().cursor();
                            let expr = mem::replace(&mut instruction.exprs[1], None);
                            self.emit_relocation(
                                Relocation::Abs32S,
                                offset,
                                expr.expect("Expression should be some"),
                            );
                        }
                        self.get_section().write_u32(disp as u32);
                    } else {
                        return Err(anyhow!("Displacement out of range"));
                    }
                } else if memory_index.base.is_valid() {
                    let byte = base_index_transfer_byte(dest.try_into()?, size);
                    self.get_section().write_u8(byte);

                    let mut bis_byte = if memory_index.index.is_valid() {
                        if memory_index.index.is_gp() {
                            let byte = memory_index_byte(Register::none(), scale, false, true);
                            byte
                        } else {
                            return Err(anyhow!(
                                "Index register must be a general purpose register"
                            ));
                        }
                    } else {
                        if !memory_index.base.is_gp() {
                            return Err(anyhow!("Invalid base register"));
                        }
                        let byte = memory_index_byte(memory_index.base, scale, false, false);
                        byte
                    };

                    // This byte is only emitted if there is an index register
                    let base_index_byte = memory_index.base.get_gp().unwrap_or(0) << 4
                        | memory_index.index.get_gp().unwrap_or(0);

                    if !instruction.reloc[1] && let Ok(disp) = i16::try_from(memory_index.disp as i64) {
                        // We set this bit to one to signal to the CPU that this instruction has a
                        // two byte displacement
                        bis_byte |= bit!(1);
                        self.get_section().write_u8(bis_byte);

                        // We don't need to check if the register is a GP register because that
                        // would have already been done later
                        if memory_index.index.is_valid() {
                            self.get_section().write_u8(base_index_byte);
                        }

                        self.get_section().write_u16(disp as u16);
                    } else if let Ok(disp) = i32::try_from(memory_index.disp as i64) {
                        self.get_section().write_u8(bis_byte);

                        if memory_index.index.is_valid() {
                            self.get_section().write_u8(base_index_byte);
                        }

                        if instruction.reloc[1] {
                            let offset = self.get_section().cursor();
                            let expr = mem::replace(&mut instruction.exprs[1], None);
                            self.emit_relocation(
                                Relocation::Abs32S,
                                offset,
                                expr.expect("Expression should be some"),
                            );
                        }

                        self.get_section().write_u32(disp as u32);
                    } else {
                        return Err(anyhow!("Displacement out of range"));
                    }
                } else if memory_index.base.is_invalid() && memory_index.index.is_invalid() {
                    todo!("Constant addressing")
                }
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

                    if instruction.reloc[0] {
                        let offset = self.get_section().cursor();
                        let expr = std::mem::replace(&mut instruction.exprs[0], None);
                        self.emit_relocation(Relocation::Abs8, offset, expr.unwrap());
                    }

                    self.get_section().write_u8(byte);
                } else {
                    unreachable!("Invalid instruction template")
                }
            }
        } else if options.intersects(encoding!(JMP)) {
            let disp = instruction.operands[0].constant();
            let offset = if !instruction.reloc[0] {
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
                let expr = std::mem::replace(&mut instruction.exprs[0], None).unwrap();
                let cursor = self.get_section().cursor();
                self.emit_relocation(Relocation::PC32, cursor, expr);
                0
            };

            self.get_section().write_u32(offset as u32);
        } else if options.intersects(encoding!(OPCODE_REG)) {
            let reg = instruction.operands[0].register();

            // Instructions with the OPCODE_REG option has its register encoded as the last 4 bits
            *self.get_section().data.last_mut().unwrap() |= reg.get_gp().unwrap();
        } else if !options.is_empty() && instruction.operand_count == 0 {
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
