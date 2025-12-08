use crate::tokens::TokenIter;

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    current: usize,
    line: usize,
    peeked: Option<Option<&'a str>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current: 1,
            line: 0,
            peeked: None,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn next(&mut self) -> Option<&'a str> {
        if let Some(peeked) = self.peeked {
            self.peeked = None;
            return peeked;
        }

        let start_index = self.current;

        let mut final_index = self.current;
        for (i, ch) in self.source[start_index..].chars().enumerate() {
            // Convert to the actual index
            let i = start_index + i;
            if Self::is_seperator_char(ch) {
                // The current token is the seperator char so we
                // do different logic
                if self.current == i {
                    // Count the number of lines
                    if ch == '\n' {
                        self.line += 1;
                    }
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
        if self.current < self.source.len() && final_index != self.current {
            // let token = &self.source[self.current..final_index];
            let token = &self.source[self.current..final_index];
            self.current = final_index;
            Some(token)
        } else {
            None
        }
    }

    pub fn peek(&mut self) -> Option<&str> {
        let token = if let Some(peeked) = self.peeked {
            peeked
        } else {
            let next_token = self.next();
            self.peeked = Some(next_token);
            next_token
        };

        token
    }

    fn is_seperator_char(ch: char) -> bool {
        matches!(
            ch,
            ',' | '[' | ']' | '(' | ')' | '\n' | '=' | '+' | '-' | '*' | '/' | '^' | '&' | '@' | ':' | '$'
        )
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[derive(Debug)]
pub struct SourceCode {
    source: String,
}

impl SourceCode {
    pub fn new(mut source: String) -> Self {
        // Normalize string with a newline at the end
        if !source.ends_with('\n') {
            source.push('\n');
        }
        Self { source }
    }

    pub fn iter<'a>(&'a self) -> Lexer<'a> {
        Lexer {
            source: &self.source,
            current: 0,
            line: 1,
            peeked: None,
        }
    }

    pub fn tokens<'a>(&self) -> TokenIter {
        let lexer = self.iter();
        TokenIter::new(lexer)
    }
}
