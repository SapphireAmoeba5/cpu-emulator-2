
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Size {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
}
