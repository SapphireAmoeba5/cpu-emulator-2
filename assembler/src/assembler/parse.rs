use crate::{
    assembler::Assembler,
    tokens::{Register, Token, TokenIter},
};
use anyhow::{Context, Result, anyhow, bail};
use spdlog::prelude::*;
