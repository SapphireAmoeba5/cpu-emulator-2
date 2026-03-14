use crate::tokens::TokenIter;

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    current: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source,
            current: 0,
        }
    }

    fn is_seperator_char(ch: char) -> bool {
        matches!(
            ch,
            ',' | '['
                | ']'
                | '('
                | ')'
                | '\n'
                | '='
                | '+'
                | '-'
                | '*'
                | '/'
                | '^'
                | '&'
                | '@'
                | ':'
                | '$'
        )
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a str;
    /// Parsing logic that uses the output of `next` must take into consideration that `next` may
    /// return None before a final newline if the source code being lexed does not end in a
    /// newline.
    fn next(&mut self) -> Option<Self::Item> {
        let start_index = self.current;

        let mut final_index = self.current;

        let mut iter = self.source[start_index..]
            .chars()
            .enumerate()
            .map(|(i, ch)| (start_index + i, ch))
            .peekable();

        while let Some((i, ch)) = iter.next() {
            if Self::is_seperator_char(ch) {
                // The current token is everything before the seperator char
                if self.current != i {
                    final_index = i;
                    break;
                }
                // The current token is the seperator char
                else {
                    final_index = i + 1;
                    break;
                }
            } else if ch.is_whitespace() {
                if self.current != i {
                    final_index = i;
                    break;
                }

                self.current = i + 1;
            } else if ch == '"' || ch == '\'' {
                if self.current != i {
                    final_index = i;
                    break;
                }

                let opening_quote_type = ch;

                while let Some((i, ch)) = iter.next() {
                    final_index = i;
                    if ch == opening_quote_type {
                        break;
                    }
                }

                final_index += 1;
                break;
            } else if ch == ';' {
                // Return the token before the comment
                if self.current != i {
                    final_index = i;
                    break;
                }
                // The current token is the ';' character, we need to skip everything up to the
                // next newline or EOF
                while let Some((_, ch)) = iter.peek()
                    && *ch != '\n'
                {
                    _ = iter.next();
                }

                if let Some((newline_index, ch)) = iter.next()
                    && ch == '\n'
                {
                    self.current = newline_index;
                    final_index = newline_index + 1;
                    break;
                } else {
                    // The comment extended to EOF, so we set `self.current`
                    // to `self.source.len()`
                    self.current = self.source.len();
                    break;
                }
            }
        }

        if self.current < self.source.len() {
            // If this is true then we have reached the end of the source code so we return
            // everything from self.current to source.len()
            if self.current >= final_index {
                let token = &self.source[self.current..];
                self.current = self.source.len();
                Some(token)
            } else {
                let token = &self.source[self.current..final_index];
                self.current = final_index;
                Some(token)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<&str> {
        Lexer::new(source).collect()
    }

    #[test]
    fn test_lexer() {
        let lexed = lex(".section .entry");
        assert_eq!(lexed, &[".section", ".entry"]);

        let lexed = lex("test\ntest");
        assert_eq!(lexed, &["test", "\n", "test"]);

        let lexed = lex("1 + 2+      3\n");
        assert_eq!(lexed, &["1", "+", "2", "+", "3", "\n"]);

        let lexed = lex("1+2+3\n");
        assert_eq!(lexed, &["1", "+", "2", "+", "3", "\n"]);

        let lexed = lex("      test\n        ");
        assert_eq!(lexed, &["test", "\n"]);
    }

    #[test]
    fn test_comments() {
        let lexed = lex("Test ; This is a comment\n");
        assert_eq!(lexed, &["Test", "\n"]);

        let lexed = lex("Test ; This is a comment");
        assert_eq!(lexed, &["Test"]);

        let lexed = lex("Test ; This is a comment\nTest2");
        assert_eq!(lexed, &["Test", "\n", "Test2"]);

        let lexed = lex("Test;This is a comment\nTest2");
        assert_eq!(lexed, &["Test", "\n", "Test2"]);
    }

    #[test]
    fn test_string() {
        let lexed = lex("test \"This is a string\"");
        assert_eq!(lexed, &["test", "\"This is a string\""]);

        let lexed = lex("test \"This is a string\"\n");
        assert_eq!(lexed, &["test", "\"This is a string\"", "\n"]);

        let lexed = lex("test \"This is a string\"\nafter the string");
        assert_eq!(
            lexed,
            &[
                "test",
                "\"This is a string\"",
                "\n",
                "after",
                "the",
                "string"
            ]
        );

        let lexed = lex("Testing single quote 'string'");
        assert_eq!(lexed, &["Testing", "single", "quote", "'string'"]);

        let lexed = lex("Testing single quote 'string'\n");
        assert_eq!(lexed, &["Testing", "single", "quote", "'string'", "\n"]);

        let lexed = lex("Testing 'single' quote 's'\n");
        assert_eq!(lexed, &["Testing", "'single'", "quote", "'s'", "\n"]);
    }
}
