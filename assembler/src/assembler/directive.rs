use std::{
    iter::{self, Peekable},
    rc::Rc,
};

use crate::{
    assembler::{
        AsmTokenIter, Assembler, AssemblerToken, ForwardReferenceEntry, symbol_table::Type,
    },
    expression::{Node, parse_expr},
    opcode::Relocation,
    section,
    size::Size,
    tokens::{self, Directive, Token},
};
use anyhow::{Context, Result, anyhow, bail};
use strum::EnumDiscriminants;

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

fn should_return_none<'a>(tokens: &mut Peekable<impl AsmTokenIter<'a>>) -> bool {
    match tokens.peek() {
        None
        | Some(AssemblerToken {
            token: Token::Newline,
            ..
        }) => true,
        _ => false,
    }
}

fn valid_comma<'a>(tokens: &mut Peekable<impl AsmTokenIter<'a>>) -> bool {
    match tokens.peek() {
        None
        | Some(AssemblerToken {
            token: Token::Newline,
            ..
        }) => true,
        Some(AssemblerToken {
            token: Token::Comma,
            ..
        }) => {
            _ = tokens.next();
            true
        }
        _ => false,
    }
}

impl Assembler {
    fn parse_expr_argument<'a>(
        &self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<Option<(u64, bool, Box<Node>)>> {
        if should_return_none(tokens) {
            return Ok(None);
        }

        let expr = parse_expr(tokens)?;
        let (value, relocation) = self.evaluate_non_operand_expression(&expr)?;

        if !valid_comma(tokens) {
            bail!("Expected comma");
        }

        Ok(Some((value, relocation, expr)))
    }

    fn parse_identifier_argument<'a>(
        &self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<Option<String>> {
        if should_return_none(tokens) {
            return Ok(None);
        }

        let Some(AssemblerToken {
            token: Token::Identifier(id),
            ..
        }) = tokens.next()
        else {
            bail!("Expected identifier");
        };

        if !valid_comma(tokens) {
            bail!("Expected comma");
        }

        Ok(Some(id.clone()))
    }

    fn parse_string_argument<'a>(
        &self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<Option<Rc<str>>> {
        if should_return_none(tokens) {
            return Ok(None);
        }

        let Some(AssemblerToken {
            token: Token::Ascii(string),
            ..
        }) = tokens.next()
        else {
            bail!("Expected string");
        };

        if !valid_comma(tokens) {
            bail!("Expected comma");
        }

        Ok(Some(string.clone()))
    }

    pub(super) fn parse_directive<'a>(
        &mut self,
        directive: Directive,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        match directive {
            Directive::Section => self.parse_section_directive(tokens),
            Directive::Equ => self.parse_equ(tokens),
            Directive::Align => self.parse_section_align(tokens),
            Directive::Skip => self.parse_skip(tokens),
            Directive::Global => self.parse_global_directive(tokens),
            Directive::U8 => self.parse_embed(Size::U8, tokens),
            Directive::U16 => self.parse_embed(Size::U16, tokens),
            Directive::U32 => self.parse_embed(Size::U32, tokens),
            Directive::U64 => self.parse_embed(Size::U64, tokens),
            Directive::Ascii => self.parse_ascii(tokens),
        }?;

        // A directive must consist of the entire line, if not then it is an error
        match tokens.next() {
            None
            | Some(AssemblerToken {
                token: Token::Newline,
                ..
            }) => {}
            _ => bail!("Unexpected token"),
        }

        Ok(())
    }

    fn parse_ascii<'a>(&mut self, tokens: &mut Peekable<impl AsmTokenIter<'a>>) -> Result<()> {
        fn escape_char(chars: &mut impl Iterator<Item = char>) -> Result<u8> {
            match chars.next() {
                None => bail!("'\\' must be followed by an escape character"),
                Some(c) => match c {
                    'n' => Ok(b'\n'),
                    '\'' => Ok(b'\''),
                    '\"' => Ok(b'\"'),
                    'a' => Ok(0x07),
                    'b' => Ok(0x08),
                    'f' => Ok(0x0c),
                    '\r' => Ok(b'\r'),
                    '\t' => Ok(b'\t'),
                    'v' => Ok(0x0b),
                    '\\' => Ok(b'\\'),
                    '0' => Ok(0),
                    'x' => {
                        let mut total = 0;
                        for i in (0..2).rev() {
                            if let Some(c) = chars.next()
                                && let Some(digit) = c.to_digit(16)
                            {
                                total += (digit as u8) << (4 * i);
                            } else {
                                break;
                            }
                        }

                        Ok(total)
                    }
                    _ => bail!("Invalid escape character '{c}'"),
                },
            }
        }
        let mut count = 0usize;

        while let Some(string) = self.parse_string_argument(tokens)? {
            count += 1;

            let (_, section) = self.sections.get_section_mut()?;
            let mut chars = string.chars();
            while let Some(c) = chars.next() {
                let mut buf = [0; 4];
                if c == '\\' {
                    let escaped = escape_char(&mut chars)?;
                    section.write_u8(escaped);
                } else {
                    let slice = c.encode_utf8(&mut buf);
                    section.write_bytes(slice.as_bytes());
                }
            }
        }

        if count > 0 {
            Ok(())
        } else {
            bail!("Expected one or more arguments");
        }
    }

    fn parse_embed<'a>(
        &mut self,
        size: Size,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let relocation_kind = match size {
            Size::U8 => Relocation::Abs8,
            Size::U16 => Relocation::Abs16,
            Size::U32 => Relocation::Abs32,
            Size::U64 => Relocation::Abs64,
        };

        let mut count = 0usize;
        while let Some((value, relocation, expr)) = self.parse_expr_argument(tokens)? {
            count += 1;
            let (section_id, section) = self.sections.get_section_mut()?;
            if relocation {
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

            match size {
                Size::U8 => section.write_u8(value as u8),
                Size::U16 => section.write_u16(value as u16),
                Size::U32 => section.write_u32(value as u32),
                Size::U64 => section.write_u64(value as u64),
            }
        }
        if count > 0 {
            Ok(())
        } else {
            bail!("Expected one or more arguments")
        }
    }

    fn parse_equ<'a>(&mut self, tokens: &mut Peekable<impl AsmTokenIter<'a>>) -> Result<()> {
        let name = self
            .parse_identifier_argument(tokens)?
            .context("Expected identifier")?;

        let (value, relocation, _) = self
            .parse_expr_argument(tokens)?
            .context("Expected expression")?;

        if relocation {
            bail!("Constant cannot have a relocatable value");
        }

        self.symbols
            .insert_symbol(name, value, Type::Constant, None)?;

        Ok(())
    }

    fn parse_section_directive<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let section_name = self
            .parse_identifier_argument(tokens)?
            .with_context(|| "Expected identifier")?;

        self.sections.set_section(section_name.as_str());

        Ok(())
    }

    fn parse_section_align<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let (align, relocation, _) = self
            .parse_expr_argument(tokens)?
            .context("Expected expression")?;

        if relocation {
            bail!("Cannot align using a relocatable symbol");
        }

        let (_, section) = self.sections.get_section_mut()?;

        section.align(align);

        Ok(())
    }

    fn parse_skip<'a>(&mut self, tokens: &mut Peekable<impl AsmTokenIter<'a>>) -> Result<()> {
        let (skip_count, relocation, _) = self
            .parse_expr_argument(tokens)?
            .context("Expected expression")?;

        if relocation {
            bail!("The skip count cannot be relocated");
        }

        let fill_value =
            if let Some((skip_count, relocation, _)) = self.parse_expr_argument(tokens)? {
                if relocation {
                    bail!("Cannot relocate the fill value");
                }
                skip_count as u8
            } else {
                0
            };

        let (_, section) = self.sections.get_section_mut()?;

        for _ in 0..skip_count {
            section.write_u8(fill_value);
        }

        Ok(())
    }

    fn parse_global_directive<'a>(
        &mut self,
        tokens: &mut Peekable<impl AsmTokenIter<'a>>,
    ) -> Result<()> {
        let id = self
            .parse_identifier_argument(tokens)?
            .context("Expected identifier")?;

        self.global_symbols.push(id);

        Ok(())
    }
}
