use std::{collections::HashMap, sync::LazyLock};

use crate::tokens::Mnemonic;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum OperandType {
    // No operand
    None,

    // Different register sizes
    Reg8,
    Reg16,
    Reg32,
    Reg64,

    Constant,
}

/// Stores high level information about an instruction (Mnemonic, operand types) in order to use it
/// to access a hashmap that contains the details about how to encode that instruction
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct InstructionInfo {
    mnemonic: Mnemonic,
    operands: [OperandType; 2],
}

impl InstructionInfo {
    pub fn new(mnemonic: Mnemonic, operands: [OperandType; 2]) -> Self {
        Self { mnemonic, operands }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Encoding {
    /// 3 byte instruction
    MovlikeRegReg,
    /// 3 byte instruction
    MovlikeRegImm8,
    /// 4 byte instruction
    MovlikeRegImm16,
    /// 6 byte instruction
    MovlikeRegImm32,
    /// 10 byte instruction
    MovlikeRegImm64,

    // 2 byte instruction
    IntLike
}

#[derive(Debug, Copy, Clone)]
pub struct EncodingInfo {
    opcode: u8,
    encoding: Encoding,
    /// The size of the instruction in bytes
    size: usize,
    /// Not used yet, but specifies if the opcode extension byte should be outputted
    opcode_ext: bool,
}

impl EncodingInfo {
    pub fn new(opcode: u8, encoding: Encoding, size: usize, opcode_ext: bool) -> Self {
        Self {
            opcode,
            encoding,
            size,
            opcode_ext,
        }
    }
}

pub fn encoding(info: &InstructionInfo) -> Option<EncodingInfo> {
    OPCODES.get(info).map(|enc| *enc)
}

pub static OPCODES: LazyLock<HashMap<InstructionInfo, EncodingInfo>> = LazyLock::new(|| {
    let opcodes = HashMap::from([
        (
            InstructionInfo::new(Mnemonic::Int, [OperandType::Constant, OperandType::None]),
            EncodingInfo::new(0x01, Encoding::IntLike, 2, false),
        ),
        // Mov instructions
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x05, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x05, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x05, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x05, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x06, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x07, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x08, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mov, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x09, Encoding::MovlikeRegImm64, 10, false),
        ),
        // Add instructions
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x15, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x15, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x15, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x15, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x16, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x17, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x18, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Add, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x19, Encoding::MovlikeRegImm64, 10, false),
        ),
        // Sub instructions
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x25, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x25, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x25, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x25, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x26, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x27, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x28, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Sub, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x29, Encoding::MovlikeRegImm64, 10, false),
        ),
        // Mul instructions
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x35, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x35, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x35, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x35, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x36, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x37, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x38, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Mul, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x39, Encoding::MovlikeRegImm64, 10, false),
        ),
        // Div instructions
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x45, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x45, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x45, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x45, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x46, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x47, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x48, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Div, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x49, Encoding::MovlikeRegImm64, 10, false),
        ),
        // Idiv instructions
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x55, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x55, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x55, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x55, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x56, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x57, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x58, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Idiv, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x59, Encoding::MovlikeRegImm64, 10, false),
        ),
        // And instructions
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x65, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x65, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x65, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x65, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x66, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x67, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x68, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::And, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x69, Encoding::MovlikeRegImm64, 10, false),
        ),
        // Or instructions
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x75, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x75, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x75, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x75, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x76, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x77, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x78, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Or, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x79, Encoding::MovlikeRegImm64, 10, false),
        ),
        // Xor instructions
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg8, OperandType::Reg8]),
            EncodingInfo::new(0x85, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg16, OperandType::Reg16]),
            EncodingInfo::new(0x85, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg32, OperandType::Reg32]),
            EncodingInfo::new(0x85, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg64, OperandType::Reg64]),
            EncodingInfo::new(0x85, Encoding::MovlikeRegReg, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg8, OperandType::Constant]),
            EncodingInfo::new(0x86, Encoding::MovlikeRegImm8, 3, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg16, OperandType::Constant]),
            EncodingInfo::new(0x87, Encoding::MovlikeRegImm16, 4, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg32, OperandType::Constant]),
            EncodingInfo::new(0x88, Encoding::MovlikeRegImm32, 6, false),
        ),
        (
            InstructionInfo::new(Mnemonic::Xor, [OperandType::Reg64, OperandType::Constant]),
            EncodingInfo::new(0x89, Encoding::MovlikeRegImm64, 10, false),
        ),
    ]);

    opcodes
});
