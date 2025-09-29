use crate::{
    assembler::{Assembler},
    tokens::{Register, Token, TokenIter},
};
use anyhow::{anyhow, bail, Context, Result};
use spdlog::prelude::*;

