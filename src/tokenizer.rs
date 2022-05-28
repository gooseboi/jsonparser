use core::iter::Peekable;
use core::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    OpenBracket,
    ClosedBracket,

    OpenSqBracket,
    ClosedSqBracket,

    Comma,
    Colon,

    Ident(String),
    Val(String),

    End,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Loc {
    pub(crate) col: usize,
    pub(crate) line: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) loc: Loc,
}

#[derive(Clone)]
pub struct Tokenizer<Iter: Iterator<Item = char>> {
    iter: Peekable<Iter>,
    col: usize,
    line: usize,
}

impl<'a> Tokenizer<Chars<'a>> {
    pub fn from_str(str: &'a str) -> Self {
        Self::from_iter(str.chars())
    }
}

impl<Iter: Iterator<Item = char>> Tokenizer<Iter> {
    pub fn from_iter(iter: Iter) -> Self {
        Self {
            iter: iter.peekable(),
            col: 0,
            line: 1,
        }
    }

    fn cur_loc(&self) -> Loc {
        Loc {
            col: self.col,
            line: self.line,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let mut text = String::new();

        match self.iter.next() {
            Some(c) => {
                self.col += 1;
                match c {
                    '{' => Token {
                        kind: TokenKind::OpenBracket,
                        loc: self.cur_loc(),
                    },
                    '}' => Token {
                        kind: TokenKind::ClosedBracket,
                        loc: self.cur_loc(),
                    },
                    '[' => Token {
                        kind: TokenKind::OpenSqBracket,
                        loc: self.cur_loc(),
                    },
                    ']' => Token {
                        kind: TokenKind::ClosedSqBracket,
                        loc: self.cur_loc(),
                    },
                    ':' => Token {
                        kind: TokenKind::Colon,
                        loc: self.cur_loc(),
                    },
                    ',' => Token {
                        kind: TokenKind::Comma,
                        loc: self.cur_loc(),
                    },
                    // Strings, can be Identifiers or Values
                    '"' => {
                        let loc = self.cur_loc();
                        while let Some(c) = self.iter.next_if(|c| *c != '"') {
                            self.col += 1;
                            text.push(c);
                        }
                        if self.iter.next_if(|c| *c == '"').is_some() {
                            self.col += 1;
                            self.skip_whitespace();
                            if let Some(':') = self.iter.peek() {
                                Token {
                                    kind: TokenKind::Ident(text),
                                    loc,
                                }
                            } else {
                                text.insert(0, '"');
                                text.push('"');
                                Token {
                                    kind: TokenKind::Val(text),
                                    loc,
                                }
                            }
                        } else {
                            panic!("Missing ending \"")
                        }
                    }
                    // Numbers
                    '0'..='9' => {
                        text.push(c);
                        let loc = self.cur_loc();
                        while let Some(c) = self.iter.next_if(|c| {
                            (!c.is_ascii_alphabetic() || c.to_ascii_lowercase() == 'e')
                                && (c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
                        }) {
                            self.col += 1;
                            text.push(c);
                        }
                        Token {
                            kind: TokenKind::Val(text),
                            loc,
                        }
                    }
                    // Cases like `null` or `true`
                    c => {
                        text.push(c);
                        if c.is_ascii() {
                            let loc = self.cur_loc();
                            while let Some(c) = self.iter.next_if(|c| *c != ',' && *c != '\n') {
                                self.col += 1;
                                text.push(c);
                            }
                            if let Some(_) = self.iter.next_if(|c| *c == ',' || *c == '\n') {
                                self.col += 1;
                                Token {
                                    kind: TokenKind::Val(text.clone()),
                                    loc,
                                }
                            } else {
                                panic!("Missing ending token={}", text)
                            }
                        } else {
                            panic!("Unsupported token, {}", c)
                        }
                    }
                }
            }
            None => Token {
                kind: TokenKind::End,
                loc: self.cur_loc(),
            },
        }
    }

    pub(crate) fn expect_token(&mut self, kind: TokenKind) -> Token {
        // Should return an error, but for now panics
        let token = self.next_token();
        if token.kind == kind {
            token
        } else {
            panic!(
                "Incorrect token, expected `{:?}`, got `{:?}`",
                token.kind, kind
            )
        }
    }

    fn skip_whitespace(&mut self) {
        while self
            .iter
            .next_if(|c| {
                if c.is_whitespace() {
                    if *c == '\n' {
                        self.col = 0;
                        self.line += 1;
                    } else {
                        self.col += 1;
                    }
                    true
                } else {
                    false
                }
            })
            .is_some()
        {}
    }
}

impl<Iter: Iterator<Item = char>> Iterator for Tokenizer<Iter> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::End {
            None
        } else {
            Some(token)
        }
    }
}
