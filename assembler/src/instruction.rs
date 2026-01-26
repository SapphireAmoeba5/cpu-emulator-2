use strum::{AsRefStr, EnumCount, FromRepr, IntoStaticStr};

use crate::{
    opcode::{self, InstEncoding, MAX_OPERANDS, OperandFlags},
    tokens::Register,
};

pub enum Operand {
    Register(Register),
    /// The kind of constant (displacement, immediate) depends on the instruction's types bit flag
    Constant(u64),
}

pub struct Instruction {
    encoding: InstEncoding,
    operands: [Operand; MAX_OPERANDS],
    /// The type of the i'th operand
    types: [OperandFlags; MAX_OPERANDS],
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntoStaticStr, AsRefStr, EnumCount, FromRepr)]
pub enum Mnemonic {
    Mov,
    Str,
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    And,
    Or,
    Xor,
    Jmp,
    Jnz,
    Jz,
    Int,
}
