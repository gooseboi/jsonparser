use core::iter::Peekable;
use core::str::Chars;

#[derive(Debug, PartialEq, Eq)]
enum TokenKind {
    OpenBracket,
    ClosedBracket,

    OpenSqBracket,
    ClosedSqBracket,

    Comma,
    Colon,

    Ident,
    Val,

    End,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    text: String,
}

pub struct Tokenizer<Iter: Iterator<Item = char>> {
    iter: Peekable<Iter>,
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
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.trim_whitespace();
        let mut text = String::new();

        match self.iter.next() {
            Some(c) => {
                text.push(c);
                match c {
                    '{' => Token {
                        kind: TokenKind::OpenBracket,
                        text,
                    },
                    '}' => Token {
                        kind: TokenKind::ClosedBracket,
                        text,
                    },
                    '[' => Token {
                        kind: TokenKind::OpenSqBracket,
                        text,
                    },
                    ']' => Token {
                        kind: TokenKind::ClosedSqBracket,
                        text,
                    },
                    ':' => Token {
                        kind: TokenKind::Colon,
                        text,
                    },
                    ',' => Token {
                        kind: TokenKind::Comma,
                        text,
                    },
                    '"' => {
                        text.clear();
                        while let Some(c) = self.iter.next_if(|c| *c != '"') {
                            text.push(c);
                        }
                        if let Some(_) = self.iter.next_if(|c| *c == '"') {
                            if let Some(':') = self.iter.peek() {
                                Token {
                                    kind: TokenKind::Ident,
                                    text,
                                }
                            } else {
                                text.insert(0, '"');
                                text.push('"');
                                Token {
                                    kind: TokenKind::Val,
                                    text,
                                }
                            }
                        } else {
                            panic!("Missing ending \"")
                        }
                    }
                    '0'..='9' => {
                        while let Some(c) = self.iter.next_if(|c| c.is_ascii_alphanumeric()) {
                            text.push(c);
                        }
                        Token {
                            kind: TokenKind::Val,
                            text,
                        }
                    }
                    c => {
                        if c.is_ascii() {
                            while let Some(c) = self.iter.next_if(|c| *c != ',') {
                                text.push(c);
                            }
                            if let None | Some(',' | '}' | ']') = self.iter.next_if(|c| *c == ',') {
                                Token {
                                    kind: TokenKind::Val,
                                    text,
                                }
                            } else {
                                panic!("Missing ending ',' text={}", text)
                            }
                        } else {
                            panic!("Unsupported token, {}", c)
                        }
                    }
                }
            }
            None => Token {
                kind: TokenKind::End,
                text: String::new(),
            },
        }
    }

    fn trim_whitespace(&mut self) {
        while self.iter.next_if(|c| c.is_whitespace()).is_some() {}
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
