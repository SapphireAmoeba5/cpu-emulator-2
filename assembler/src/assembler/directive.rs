use std::iter::{self, Peekable};

use crate::{
    assembler::{AsmTokenIter, Assembler, ForwardReferenceEntry},
    expression::{Node, parse_expr},
    opcode::Relocation,
    tokens::{Directive, Token},
};
use anyhow::{Result, bail};
use strum::EnumDiscriminants;

/// This type doesn't implement `Iterator` because it needs to know the kind of token you expect to
/// be there which the normal Iterator trait doesn't allow you to do
struct _DirectiveOperandIter<'a, I: for<'b> AsmTokenIter<'b>> {
    tokens: &'a mut Peekable<I>,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(DirectiveArgumentKind))]
enum DirectiveArgument {
    Expr {
        value: u64,
        relocation: bool,
        expr: Box<Node>,
    },
    Identifier(String),
}

impl Assembler {
    /// Iterates over `kinds` and parses arguments from `tokens` based off each
    /// specified DirectionArgumentKind and returns the number of arguments
    /// parsed (which is just the length of `arguments`)
    ///
    /// Fully consumes `tokens` up-to and including the next `Token::Newline` or None
    ///
    /// This function will return Ok if the `kinds` iterator has been fully
    /// consumed or if `tokens` reaches either a Newline or None in which case this
    /// function will return a value lower than number of elements `kinds` would yield
    /// if it were fully consumed.
    ///
    /// # Errors
    /// This function will return Err if the syntax for the arguments is malformed, if `kinds`
    /// is fully consumed but there are still tokens left on the current line,
    /// or if any of the functions called internally fail
    fn parse_directive_arguments<'a>(
        &self,
        arguments: &mut Vec<DirectiveArgument>,
        kinds: impl IntoIterator<Item = DirectiveArgumentKind>,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<usize> {
        match tokens.peek().map(|a| &a.token) {
            None | Some(Token::Newline) => return Ok(0),
            _ => {}
        }

        for expected_kind in kinds {
            match expected_kind {
                DirectiveArgumentKind::Expr => {
                    let expr = parse_expr(tokens)?;
                    let (value, relocation) = self.evaluate_non_operand_expression(&expr)?;
                    arguments.push(DirectiveArgument::Expr {
                        value,
                        relocation,
                        expr,
                    });
                }
                DirectiveArgumentKind::Identifier => {
                    let Some(Token::Identifier(id)) = tokens.next().map(|a| a.token.clone()) else {
                        bail!("Expected identifier")
                    };

                    arguments.push(DirectiveArgument::Identifier(id));
                }
            }

            // Now we expect either a comma, newline, or None
            match tokens.peek().map(|a| &a.token) {
                None | Some(Token::Newline) => break,
                Some(Token::Comma) => _ = tokens.next(),
                _ => bail!("Expected a comma, newline, or EOF"),
            }
        }

        // A .section directive must take up an entire line,
        // and we may exit the loop because the `kinds` iterator was fully consumed
        // while the `token` iter still has tokens left on the line
        match tokens.peek().map(|a| &a.token) {
            None | Some(Token::Newline) => {}
            _ => bail!("Too many arguments for directive"),
        }

        Ok(arguments.len())
    }
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

            // Parsing the embed directives I.E .u8 {EXPR}, .u16 {EXPR}, etc
            Directive::U8 => {
                let value = self.parse_embed(Relocation::Abs8, tokens)?;
                let (_, section) = self.sections.get_section()?;
                section.write_u8(value as u8);
                Ok(())
            }
            Directive::U16 => {
                let value = self.parse_embed(Relocation::Abs16, tokens)?;
                let (_, section) = self.sections.get_section()?;
                section.write_u16(value as u16);
                Ok(())
            }
            Directive::U32 => {
                let value = self.parse_embed(Relocation::Abs32, tokens)?;
                let (_, section) = self.sections.get_section()?;
                section.write_u32(value as u32);
                Ok(())
            }
            Directive::U64 => {
                let value = self.parse_embed(Relocation::Abs64, tokens)?;
                let (_, section) = self.sections.get_section()?;
                section.write_u64(value);
                Ok(())
            }
        }
    }

    fn parse_embed<'a>(
        &mut self,
        relocation_kind: Relocation,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<u64> {
        let mut arguments = Vec::new();
        let _ = self.parse_directive_arguments(
            &mut arguments,
            iter::once(DirectiveArgumentKind::Expr),
            tokens,
        )?;

        let Some(DirectiveArgument::Expr {
            value,
            relocation,
            expr,
        }) = arguments.pop()
        else {
            bail!("Expected argument for .embed");
        };

        if relocation {
            let (section_id, section) = self.sections.get_section()?;
            let cursor = section.cursor();

            let entry = ForwardReferenceEntry::new(
                relocation_kind,
                section_id,
                cursor,
                expr,
                self.current_line,
            );
            self.forward_references.push(entry);
        }

        Ok(value)
    }

    fn parse_section_directive<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let mut arguments = Vec::new();
        let _ = self.parse_directive_arguments(
            &mut arguments,
            iter::once(DirectiveArgumentKind::Identifier),
            tokens,
        )?;

        let Some(DirectiveArgument::Identifier(id)) = arguments.pop() else {
            bail!("Expected section name")
        };

        self.sections.set_section(id.as_str());

        Ok(())
    }

    fn parse_section_align<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let mut arguments = Vec::new();
        let _ = self.parse_directive_arguments(
            &mut arguments,
            iter::once(DirectiveArgumentKind::Expr),
            tokens,
        )?;

        let Some(DirectiveArgument::Expr {
            value: align,
            relocation,
            ..
        }) = arguments.pop()
        else {
            bail!("Expected argument for .align");
        };

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

    fn parse_skip<'a>(&mut self, tokens: &mut Peekable<impl AsmTokenIter<'a>>) -> Result<()> {
        let mut arguments = Vec::new();
        let num = self.parse_directive_arguments(
            &mut arguments,
            iter::repeat_n(DirectiveArgumentKind::Expr, 2),
            tokens,
        )?;

        // Must have 1 or 2 arguments
        if num < 1 || num > 2 {
            bail!("Expected argument for .skip");
        }

        let DirectiveArgument::Expr {
            value: skip_count,
            relocation,
            ..
        } = arguments.swap_remove(0)
        else {
            panic!(
                "The enum variant stored in `arguments` should match the DirectiveOperandKind we asked for. This is a bug"
            )
        };

        if relocation {
            bail!("The skip count cannot be relocated");
        }

        let fill_value: u8 = if let Some(DirectiveArgument::Expr {
            value, relocation, ..
        }) = arguments.pop()
        {
            if relocation {
                bail!("The fill value cannot be relocated");
            }

            value as u8
        } else {
            0
        };

        let (_, section) = self.sections.get_section()?;

        for _ in 0..skip_count {
            section.write_u8(fill_value);
        }

        Ok(())
    }

    fn parse_global_directive<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let mut arguments = Vec::new();
        let _ = self.parse_directive_arguments(
            &mut arguments,
            iter::once(DirectiveArgumentKind::Identifier),
            tokens,
        )?;

        let Some(DirectiveArgument::Identifier(id)) = arguments.pop() else {
            bail!("Expected argument for .global");
        };

        self.global_symbols.push(id);

        Ok(())
    }
}
