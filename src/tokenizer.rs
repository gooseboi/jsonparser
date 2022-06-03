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

#[derive(Debug)]
pub(crate) enum ParsingErrorKind {
    InvalidTrailingComma,
    MissingEndingComma,
    UnsupportedToken,
    UnexpectedToken,
    InvalidStartingToken,
    InvalidIdentInArray,
    InvalidToken,
}

#[derive(Debug)]
pub struct ParsingError {
    pub(crate) kind: ParsingErrorKind,
    pub(crate) loc: Loc,
}

type Result = std::result::Result<Token, ParsingError>;

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

    fn tokenize_val(&mut self, text: String, loc: Loc) -> Result {
        use ParsingErrorKind::*;
        if let Some(',' | '}' | ']') = self.peek() {
            Ok(Token {
                kind: TokenKind::Val(text),
                loc,
            })
        } else {
            Err(ParsingError {
                kind: MissingEndingComma,
                loc: self.cur_loc(),
            })
        }
    }

    pub fn next_token(&mut self) -> Result {
        self.skip_whitespace();
        let mut text = String::new();

        use ParsingErrorKind::*;
        match self.iter.next() {
            Some(c) => {
                self.col += 1;
                match c {
                    '{' => Ok(Token {
                        kind: TokenKind::OpenBracket,
                        loc: self.cur_loc(),
                    }),
                    '}' => Ok(Token {
                        kind: TokenKind::ClosedBracket,
                        loc: self.cur_loc(),
                    }),
                    '[' => Ok(Token {
                        kind: TokenKind::OpenSqBracket,
                        loc: self.cur_loc(),
                    }),
                    ']' => Ok(Token {
                        kind: TokenKind::ClosedSqBracket,
                        loc: self.cur_loc(),
                    }),
                    ':' => Ok(Token {
                        kind: TokenKind::Colon,
                        loc: self.cur_loc(),
                    }),
                    ',' => {
                        let loc = self.cur_loc();
                        if let Some('}' | ']') = self.peek() {
                            Err(ParsingError {
                                kind: InvalidTrailingComma,
                                loc,
                            })
                        } else {
                            Ok(Token {
                                kind: TokenKind::Comma,
                                loc,
                            })
                        }
                    }
                    // Strings, can be Identifiers or Values
                    '"' => {
                        let loc = self.cur_loc();
                        let mut was_escape = false;
                        while let Some(c) = self.iter.next_if(|c| match *c {
                            '"' => was_escape,
                            _ => true,
                        }) {
                            self.col += 1;
                            was_escape = c == '\\';
                            text.push(c);
                        }
                        if self.iter.next_if(|c| *c == '"').is_some() {
                            self.col += 1;
                            if let Some(':') = self.peek() {
                                Ok(Token {
                                    kind: TokenKind::Ident(text),
                                    loc,
                                })
                            } else {
                                text.insert(0, '"');
                                text.push('"');
                                Ok(Token {
                                    kind: TokenKind::Val(text),
                                    loc,
                                })
                            }
                        } else {
                            unreachable!("Text: {text}")
                        }
                    }
                    // Numbers
                    '0'..='9' => {
                        text.push(c);
                        let loc = self.cur_loc();
                        while let Some(c) = self.iter.next_if(is_num_char) {
                            self.col += 1;
                            text.push(c);
                        }
                        self.tokenize_val(text, loc)
                    }
                    // Cases like `null` or `true`
                    c => {
                        text.push(c);
                        if c.is_ascii() {
                            let loc = self.cur_loc();
                            while let Some(c) =
                                self.next_if(|c| *c != ',' && *c != '}' && *c != ']')
                            {
                                self.col += 1;
                                text.push(c);
                            }
                            self.tokenize_val(text, loc)
                        } else {
                            Err(ParsingError {
                                kind: UnsupportedToken,
                                loc: self.cur_loc(),
                            })
                        }
                    }
                }
            }
            None => Ok(Token {
                kind: TokenKind::End,
                loc: self.cur_loc(),
            }),
        }
    }

    pub(crate) fn expect_token(&mut self, kind: TokenKind) -> Result {
        match self.next_token() {
            Ok(token) => {
                if token.kind == kind {
                    Ok(token)
                } else {
                    Err(ParsingError {
                        kind: ParsingErrorKind::UnexpectedToken,
                        loc: token.loc,
                    })
                }
            }
            err @ Err(_) => err,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.skip_whitespace();
        self.iter.peek()
    }

    fn next_if(&mut self, pred: impl FnOnce(&char) -> bool) -> Option<char> {
        self.skip_whitespace();
        self.iter.next_if(pred)
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

pub(crate) fn is_num_char(c: &char) -> bool {
    (!c.is_alphabetic() || c.to_lowercase().next().unwrap() == 'e')
        && (c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '+')
}
