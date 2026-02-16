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
    Halt,
    Mov,
    MovU8,
    MovU16,
    MovU32,
    MovU64,

    Str,
    StrU8,
    StrU16,
    StrU32,

    Lea,

    Add,
    AddU8,
    AddU16,
    AddU32,
    AddU64,

    Sub,
    SubU8,
    SubU16,
    SubU32,
    SubU64,

    Mul,
    MulU8,
    MulU16,
    MulU32,
    MulU64,

    Div,
    DivU8,
    DivU16,
    DivU32,
    DivU64,

    Idiv,
    IdivU8,
    IdivU16,
    IdivU32,
    IdivU64,

    And,
    AndU8,
    AndU16,
    AndU32,
    AndU64,

    Or,
    OrU8,
    OrU16,
    OrU32,
    OrU64,

    Xor,
    XorU8,
    XorU16,
    XorU32,
    XorU64,

    Cmp,
    CmpU8,
    CmpU16,
    CmpU32,
    CmpU64,

    Test,
    TestU8,
    TestU16,
    TestU32,
    TestU64,

    Jmp,

    Jnz,
    Jz,

    Jc,
    Jnc,

    Jo,
    Jno,

    Js,
    Jns,

    Ja,
    Jbe,

    Jg,
    Jle,

    Jge,
    Jl,

    Call,
    Ret,

    Rdt,
    Rdtf,

    Rdsp,
    Stsp,

    Int,
}
