use std::iter::Peekable;

use crate::{
    assembler::{AsmTokenIter, Assembler},
    expression::parse_expr,
    section::Section,
    tokens::{Directive, Token, TokenIter},
};
use anyhow::{Context, Result, anyhow, bail};
use spdlog::debug;

impl Assembler {
    pub(super) fn parse_directive<'a>(
        &mut self,
        directive: Directive,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        match directive {
            Directive::Section => self.parse_section_directive(tokens),
            Directive::Align => self.parse_section_align(tokens),
            Directive::Skip => self.parse_skip(tokens),
            Directive::Global => self.parse_global_directive(tokens),
            Directive::U8 => self.parse_embed_u8(tokens),
            Directive::U16 => self.parse_embed_u16(tokens),
            Directive::U32 => self.parse_embed_u32(tokens),
            Directive::U64 => self.parse_embed_u64(tokens),
        }
    }

    pub(super) fn parse_section_directive<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let section_name = &tokens
            .next()
            .context("Expected section name but found EOF")?
            .token;

        match section_name {
            Token::Identifier(identifier) => {
                self.sections.set_section(identifier.as_str());
                Ok(())
            }
            other => bail!("Expected identifier after .section but got {}", other),
        }
    }

    pub(super) fn parse_section_align<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let (align, relocation) = self.evaluate_non_operand_expression(&expr)?;

        if relocation {
            bail!("Cannot align using a relocatable symbol");
        }

        let (_, section) = self.sections.get_section()?;

        if align > section.alignment {
            section.alignment = align;
        }

        let align: usize = align.try_into().unwrap();
        let n: usize = (align - (section.size() % align)) % align;

        section.data.resize(section.data.len() + n, 0);

        Ok(())
    }

    pub(super) fn parse_skip<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let (skip_count, relocation) = self.evaluate_non_operand_expression(&expr)?;

        if relocation {
            bail!("The amount to fill cannot reference a relocatable symbol");
        }

        let fill_value: u8 = if let Some(Token::Comma) = tokens.peek().map(|a| &a.token) {
            let _ = tokens.next();
            let fill_value_expr = parse_expr(tokens)?;
            let (fill_value, relocation) =
                self.evaluate_non_operand_expression(&fill_value_expr)?;
            if relocation {
                bail!("The fill value cannot reference a relocatable symbol");
            }
            fill_value as u8
        } else {
            0
        };

        let (_, section) = self.sections.get_section()?;

        for _ in 0..skip_count {
            section.write_u8(fill_value);
        }

        Ok(())
    }

    pub(super) fn parse_global_directive<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let symbol = &tokens
            .next()
            .context("Expected symbol but found EOF")?
            .token;

        match symbol {
            Token::Identifier(identifier) => self.global_symbols.push(identifier.clone()),
            _ => {
                return Err(anyhow!(
                    "Espected identifier after .global but got {}",
                    symbol.to_string()
                ));
            }
        }

        Ok(())
    }

    pub(super) fn parse_embed_u8<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let (value, relocation) = self.evaluate_non_operand_expression(&expr)?;

        // TODO: Relocation

        let(_, section) = self.sections.get_section()?;
        section.write_u8(value as u8);

        Ok(())
    }

    pub(super) fn parse_embed_u16<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let (value, relocation) = self.evaluate_non_operand_expression(&expr)?;

        // TODO: Relocation

        let(_, section) = self.sections.get_section()?;
        section.write_u16(value as u16);

        Ok(())
    }

    pub(super) fn parse_embed_u32<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let (value, relocation) = self.evaluate_non_operand_expression(&expr)?;

        // TODO: Relocation

        let(_, section) = self.sections.get_section()?;
        section.write_u32(value as u32);

        Ok(())
    }

    pub(super) fn parse_embed_u64<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let (value, relocation) = self.evaluate_non_operand_expression(&expr)?;

        // TODO: Relocation

        let(_, section) = self.sections.get_section()?;
        section.write_u64(value);

        Ok(())
    }
}
