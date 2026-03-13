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
        for (i, ch) in self.source[start_index..]
            .chars()
            .enumerate()
            .map(|(i, ch)| (start_index + i, ch))
        {
            if Self::is_seperator_char(ch) {
                // The current token is the seperator char so we
                // do different logic
                if self.current == i {
                    final_index = i + 1;
                    break;
                }
                // The current token is everything before the seperator char
                else {
                    final_index = i;
                    break;
                }
            } else if ch.is_whitespace() {
                if self.current != i {
                    final_index = i;
                    break;
                } else {
                    self.current = i + 1;
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
                // let token = &self.source[self.current..final_index];
                let token = &self.source[self.current..final_index];
                self.current = final_index;
                Some(token)
            }
        } else {
            None
        }
    }
}
