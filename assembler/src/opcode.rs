use std::{collections::HashMap, sync::LazyLock};

use bitflags::bitflags;
use clap::builder::OsStringValueParser;
use strum::{AsRefStr, EnumCount, IntoStaticStr};

use crate::instruction::Mnemonic;

pub const MAX_OPERANDS: usize = 3;

/// Syntactic sugar for getting the nth bit
#[macro_export]
macro_rules! bit {
    ($n:expr) => {
        (1 << ($n))
    };
}

/// Easy way to combine multiple flags from the bitflags crate
///
/// Example:
/// ```rust
/// flags!(IMM8 | IMM32)
/// ```
/// Expands to:
/// ```rust
/// Self::IMM8.bits() | Self::IMM32.bits()
/// ```
macro_rules! flags {
    ($first:ident $(| $opt:ident)*) => {
        Self::$first.bits() $(| Self::$opt.bits())*
    };
}

/// Similar to the `flags!` macro except it's only for EncodingFlags and this macro can be used
/// anywhere to simplify or'ing together multiple EncodingFlags
#[macro_export]
macro_rules! encoding {
    ($first:ident $(| $opt:ident)*) => {
        EncodingFlags::$first $(| EncodingFlags::$opt)*
    };
}

/// Similar to the `flags!` macro except it's only for OperandFlags and this macro can be used
/// anywhere to simplify or'ing together multiple OperandFlags
#[macro_export]
macro_rules! operand {
    ($first:ident $(| $opt:ident)*) => {
        OperandFlags::$first $(| OperandFlags::$opt)*
    };
}

bitflags! {
    #[derive(Debug, Hash, Clone, Copy)]
    pub struct OperandFlags: u32 {
        const GP_REG = bit!(0);
        const REG = bit!(1);
        const IMM8 = bit!(2);
        const IMM16 = bit!(3);
        const IMM32 = bit!(4);
        const IMM64 = bit!(5);
        /// An immediate value
        const IMM = flags!(IMM8 | IMM16 | IMM32 | IMM64);

        /// A displacement
        const DISP32 = bit!(10);
        const DISP = flags!(DISP32);
        // const DISP = Self::DISP32.bits();
        
        const ADDR64 = bit!(20);
        const ADDR = flags!(ADDR64);

        const INDEX = bit!(21);
    }
}

bitflags! {
    #[derive(Debug, Hash, Clone, Copy)]
    pub struct EncodingFlags: u64 {
        /* DATA TRANSFER BITS */

        /// Top level flag that specifies that this instruction will involve some form of data
        /// transfer
        /// If an instruction has this flag set, then it will have at most one other flag set from
        /// the data transfer category including the flags that combine multiple other flags
        const DATA_TRANSFER = bit!(0);
        /// 64bit Register to register transfer
        const REG = bit!(1);

        /// A 64bit immediate value
        const IMM64 = bit!(3);
        /// A 32bit immediate value
        const IMM32 = bit!(4);
        /// A 8bit immediate value
        const IMM8 = bit!(5);
        /// All immediate value types
        const IMM = flags!(IMM64 | IMM32 | IMM8);

        /// one byte memory access
        const MEM8 = bit!(6);
        /// Two byte memory access
        const MEM16 = bit!(7);
        /// 4 byte memory access
        const MEM32 = bit!(8);
        /// 8 byte memory access
        const MEM64 = bit!(9);
        /// All memory accesses
        const MEM = flags!(MEM8 | MEM16 | MEM32 | MEM64);

        /*
        *   SYSTEM CONTROL BITS
        *   These are for instructions like int, or syscall etc
        * */
        const SYS_CONTROL = bit!(10);

        const JMP = bit!(11);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
pub enum Relocation {
    // No relocation
    None,
    // Absolute value relocation
    Abs8,
    Abs16,
    Abs32,
    Abs64,
    // Sign extended absolute value
    Abs8S,
    Abs16S,
    Abs32S,
    Abs64S,
    // PC relative relocation
    PC8,
    PC32,
    PC64,
    Addr64,
}

#[derive(Debug, Clone, Copy)]
pub struct InstEncoding {
    pub opcode: u8,
    pub extension: bool,
    /// Stores extra information about the encoding of this instruction
    pub options: EncodingFlags,
    /// Stores the possible operand types for the i'th operand
    pub operands: [OperandFlags; MAX_OPERANDS],
}

impl InstEncoding {
    pub fn new(
        opcode: u8,
        extension: bool,
        options: EncodingFlags,
        operands: [OperandFlags; MAX_OPERANDS],
    ) -> Self {
        Self {
            opcode,
            extension,
            options,
            operands,
        }
    }

    pub fn operand_count(&self) -> usize {
        let mut operand_count = 0;
        for i in self.operands {
            if i.is_empty() {
                break;
            } else {
                operand_count += 1;
            }
        }
        operand_count
    }
}
pub fn get_encodings(mnemonic: Mnemonic) -> &'static [InstEncoding] {
    ENCODING_TABLE.get(mnemonic as usize).unwrap().as_slice()
}

#[rustfmt::skip]
static ENCODING_TABLE: LazyLock<[Vec<InstEncoding>; Mnemonic::COUNT]> = LazyLock::new(|| {
    // We first make a hashmap because the ordering and values of each Mnemonic might change for
    // now. We can initialize this directly as an array at any point. It's very slow but this is
    // only run once so it's not a big deal
    let encodings = HashMap::from([
        // TODO: Make all the DATA_TRANSFER instructions nto need the | REG or | IMM, etc bitflag
        // and instead have the emitter use the second operand type to figure out what to do
        (Mnemonic::Mov, vec![
            InstEncoding::new(0x05, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x06, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),

            InstEncoding::new(0x07, false, encoding!(DATA_TRANSFER | MEM64), [OperandFlags::GP_REG, OperandFlags::ADDR64, OperandFlags::empty()]),
            InstEncoding::new(0x07, false, encoding!(DATA_TRANSFER | MEM64), [OperandFlags::GP_REG, OperandFlags::DISP32 | OperandFlags::INDEX, OperandFlags::empty()]),
        ]),

        (Mnemonic::Str, vec![
            InstEncoding::new(0x08, false, encoding!(DATA_TRANSFER | MEM64), [OperandFlags::GP_REG, OperandFlags::ADDR64, OperandFlags::empty()]),
            InstEncoding::new(0x08, false, encoding!(DATA_TRANSFER | MEM64), [OperandFlags::GP_REG, OperandFlags::DISP32 | OperandFlags::INDEX, OperandFlags::empty()]),
        ]),

        (Mnemonic::Add, vec![
            InstEncoding::new(0x015, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x016, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),

            InstEncoding::new(0x017, false, encoding!(DATA_TRANSFER | MEM), [OperandFlags::GP_REG, OperandFlags::ADDR64, OperandFlags::empty()]),
            InstEncoding::new(0x017, false, encoding!(DATA_TRANSFER | MEM), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::Sub, vec![
            InstEncoding::new(0x025, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x026, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),

            InstEncoding::new(0x027, false, encoding!(DATA_TRANSFER | MEM), [OperandFlags::GP_REG, OperandFlags::ADDR64, OperandFlags::empty()]),
            InstEncoding::new(0x027, false, encoding!(DATA_TRANSFER | MEM), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::Mul, vec![
            InstEncoding::new(0x35, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x36, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),
            InstEncoding::new(0x36, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::Div, vec![
            InstEncoding::new(0x45, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x46, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),
            InstEncoding::new(0x46, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::Idiv, vec![
            InstEncoding::new(0x55, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x56, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),
            InstEncoding::new(0x56, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::And, vec![
            InstEncoding::new(0x65, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x66, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),
            InstEncoding::new(0x66, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::Or, vec![
            InstEncoding::new(0x75, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x76, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),
            InstEncoding::new(0x76, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::Xor, vec![
            InstEncoding::new(0x85, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::GP_REG, OperandFlags::empty()]),

            InstEncoding::new(0x86, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::IMM64, OperandFlags::empty()]),
            InstEncoding::new(0x86, false, encoding!(DATA_TRANSFER), [OperandFlags::GP_REG, OperandFlags::DISP32, OperandFlags::empty()]),
        ]),

        (Mnemonic::Jmp, vec![
            InstEncoding::new(0x10, false, encoding!(JMP), [OperandFlags::DISP32, OperandFlags::empty(), OperandFlags::empty()]),
        ]),
        (Mnemonic::Jnz, vec![
            InstEncoding::new(0x11, false, encoding!(JMP), [OperandFlags::DISP32, OperandFlags::empty(), OperandFlags::empty()]),
        ]),

        (Mnemonic::Jz, vec![
            InstEncoding::new(0x20, false, encoding!(JMP), [OperandFlags::DISP32, OperandFlags::empty(), OperandFlags::empty()]),
        ]),

        (Mnemonic::Int, vec![
            InstEncoding::new(0x01, false, encoding!(SYS_CONTROL), [OperandFlags::IMM8, OperandFlags::empty(), OperandFlags::empty()]),
        ])
    ]);

    let mut encodings_array: [Vec<InstEncoding>; Mnemonic::COUNT] = core::array::from_fn(|_| Vec::new());

    let mut inserted = 0usize;
    for (key, value) in encodings {
        inserted += 1;
        encodings_array[key as usize] = value; 
    }

    assert_eq!(inserted, Mnemonic::COUNT, "Not all mnemonics have encodings defined");

    encodings_array
});

#[cfg(test)]
mod tests {
    use super::*;

    /// Return an iterator over [`ENCODING_TABLE`] mapped to a tuple 
    /// of ([`Mnemonic`], &[`InstEncoding`])
    fn iterate_over_encodings() -> impl Iterator<Item = (Mnemonic, &'static [InstEncoding])>{
        let encodings: &'static [Vec<InstEncoding>] = ENCODING_TABLE.as_slice();
        encodings.iter().enumerate().map(|(idx, encodings)| {
            (
                Mnemonic::from_repr(idx).expect(
                    "Index {idx} in the instruction encodings doesn't corrsospond Mnemonic variant",
                ),
                encodings.as_slice(),
            )
        })

    }

    // #[test]
    // fn all_instruction_encodings_have_valid_option_flags_set() {
    //     #[track_caller]
    //     fn invalid_bits_set(mnemonic: Mnemonic, i: usize) {
    //         panic!("{mnemonic:?} at {i}: Invalid bits set");
    //     }
    //
    //
    //     for (mnemonic, encodings) in iterate_over_encodings() {
    //         for (i, encoding) in encodings.iter().enumerate() {
    //             // This is mutable because as we test if the option flags upholds the invariants we
    //             // will unset each flag then at the end make sure that there are no bits set
    //             let mut options = encoding.options;
    //             /* 
    //             *   Encodings with the DATA_TRANSFER flag set must have only one other flag set in
    //             *   the DATA_TRANSFER category
    //             * */
    //             if options.intersects(EncodingFlags::DATA_TRANSFER) {
    //                 options &= !EncodingFlags::DATA_TRANSFER;
    //                 if options.intersects(EncodingFlags::REG) {
    //                     options &= !EncodingFlags::REG;
    //                 } else if options.intersects(EncodingFlags::IMM) {
    //                     options &= !EncodingFlags::IMM;
    //                 } else if options.intersects(EncodingFlags::MEM) {
    //                     options &= !EncodingFlags::MEM;
    //                 } else {
    //                     invalid_bits_set(mnemonic, i);
    //                 }
    //             } else if options.intersects(EncodingFlags::SYS_CONTROL) {
    //                 options &= !EncodingFlags::SYS_CONTROL;
    //                 if options.intersects(EncodingFlags::BYTE) {
    //                     options &= !EncodingFlags::BYTE;
    //                 } else {
    //                     invalid_bits_set(mnemonic, i);
    //                 }
    //             } 
    //             else {
    //                 panic!("{mnemonic:?} at {i}: Invalid encoding flag set ({:?})", encoding.options)
    //             }
    //
    //             if options.bits().count_ones() > 0 {
    //                 invalid_bits_set(mnemonic, i);
    //             }
    //         } 
    //     }
    // }
}
