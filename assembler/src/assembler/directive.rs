use crate::{
    assembler::{Assembler, ExprValue, symbol_table::Symbol}, expression::parse_expr, section::Section, tokens::{Directive, Token, TokenIter}
};
use anyhow::{Context, Result, anyhow};
use spdlog::debug;

impl Assembler {
    pub(super) fn parse_directive(
        &mut self,
        directive: Directive,
        tokens: &mut TokenIter,
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

    pub(super) fn parse_section_directive(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let section_name = tokens
            .next()?
            .context("Expected section name but found EOF")?;

        match section_name {
            Token::Identifier(identifier) => {
                let section = match self.section_map.get(&identifier) {
                    Some(section) => *section,
                    None => {
                        let index = self.sections.len();
                        let section = Section::new(identifier.clone());
                        self.sections.push(section);
                        self.section_map.insert(identifier, index);
                        index
                    }
                };

                self.current_section = Some(section);

                Ok(())
            }
            other => Err(anyhow!(
                "Expected identifier after .section but got {}",
                other.to_string()
            )),
        }
    }

    pub(super) fn parse_section_align(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let value = self.evalute_expression(&expr, Self::NO_SECTION)?;

        let align = if let ExprValue::Constant(align) = value.value {
            align
        } else {
            return Err(anyhow!("Invalid expression"));
        };

        let section = self.get_section_mut()?;
        

        if align > section.alignment {
            section.alignment = align;
        }

        let align: usize = align.try_into().unwrap();
        let n: usize = (align - (section.size() % align)) % align;
        
        section.data.resize(section.data.len() + n, 0);

        Ok(())
    }

    pub(super) fn parse_skip(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let skip_count = if let ExprValue::Constant(skip_count) = self.evalute_expression(&expr, Self::NO_SECTION)?.value {
            skip_count
        } else {
            return Err(anyhow!("Invalid expression"));
        };

        let fill_value: u8 = if let Some(Token::Comma) = tokens.peek()? {
            let _ = tokens.next();
            let fill_value_expr = parse_expr(tokens)?;
            if let ExprValue::Constant(fill_value) = self.evalute_expression(&fill_value_expr, Self::NO_SECTION)?.value {
                fill_value as u8
            } else {
                return Err(anyhow!("Invalid expression"));
            }
        } else {
            0
        };

        let section = self.get_section_mut()?;

        for _ in 0..skip_count {
            section.write_u8(fill_value);
        }

        Ok(())
    }

    pub(super) fn parse_global_directive(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let symbol = tokens.next()?.context("Expected symbol but found EOF")?;

        match symbol {
            Token::Identifier(identifier) => self.global_symbols.push(identifier),
            _ => {
                return Err(anyhow!(
                    "Espected identifier after .global but got {}",
                    symbol.to_string()
                ));
            }
        }

        Ok(())
    }

    pub(super) fn parse_embed_u8(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let value = self.evalute_expression(&expr, Self::NO_SECTION)?;

        if let ExprValue::Constant(value) = value.value {
            self.get_section_mut()?.write_u8(value.try_into().context("Constant is too large")?);
        } else {
            return Err(anyhow!("Invalid expression for a u64 embed"));
        }


        Ok(())
    }

    pub(super) fn parse_embed_u16(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let value = self.evalute_expression(&expr, Self::NO_SECTION)?;

        if let ExprValue::Constant(value) = value.value {
            self.get_section_mut()?.write_u16(value.try_into().context("Constant is too large")?);
        } else {
            return Err(anyhow!("Invalid expression for a u64 embed"));
        }


        Ok(())
    }

    pub(super) fn parse_embed_u32(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let value = self.evalute_expression(&expr, Self::NO_SECTION)?;

        if let ExprValue::Constant(value) = value.value {
            self.get_section_mut()?.write_u32(value.try_into().context("Constant is too large")?);
        } else {
            return Err(anyhow!("Invalid expression for a u64 embed"));
        }


        Ok(())
    }

    pub(super) fn parse_embed_u64(&mut self, tokens: &mut TokenIter) -> Result<()> {
        let expr = parse_expr(tokens)?;
        let value = self.evalute_expression(&expr, Self::NO_SECTION)?;

        if let ExprValue::Constant(value) = value.value {
            self.get_section_mut()?.write_u64(value);
        } else {
            return Err(anyhow!("Invalid expression for a u64 embed"));
        }


        Ok(())
    }
}
