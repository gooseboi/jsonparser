use crate::tokenizer::*;
use core::fmt::Display;
use indexmap::IndexMap;

pub type MapType<K, V> = IndexMap<K, V>;

#[derive(Debug, PartialEq)]
pub enum Number {
    UnsignedInt(u64),
    SignedInt(i64),
    Float(f64),
}

impl Eq for Number {}

#[derive(Debug, PartialEq, Eq)]
pub enum JsonVal {
    Null,
    Number(Number),
    String(String),
    Boolean(bool),
    Array(Vec<JsonVal>),
    Object(MapType<String, JsonVal>),
}

impl JsonVal {
    fn print_indent(&self, f: &mut std::fmt::Formatter<'_>, depth: u8) -> std::fmt::Result {
        for _ in 0..depth {
            write!(f, "    ")?;
        }
        Ok(())
    }
    fn fmt_impl(&self, f: &mut std::fmt::Formatter<'_>, depth: u8) -> std::fmt::Result {
        match self {
            JsonVal::Array(arr) => {
                write!(f, "[")?;
                if !arr.is_empty() {
                    write!(f, "\n")?;
                    for val in arr {
                        self.print_indent(f, depth + 1)?;
                        val.fmt_impl(f, depth + 1)?;
                        write!(f, ",\n")?;
                    }
                    self.print_indent(f, depth)?;
                }
                write!(f, "]")?;
            }
            JsonVal::Object(obj) => {
                write!(f, "{{")?;
                if !obj.is_empty() {
                    write!(f, "\n")?;
                    for (i, (ident, val)) in obj.iter().enumerate() {
                        self.print_indent(f, depth + 1)?;
                        write!(f, "\"{}\": ", ident)?;
                        val.fmt_impl(f, depth + 1)?;
                        if i != obj.len() - 1 {
                            write!(f, ",")?;
                        }
                        write!(f, "\n")?;
                    }
                }
                self.print_indent(f, depth)?;
                write!(f, "}}")?;
            }
            JsonVal::String(s) => {
                write!(f, "\"{}\"", s)?;
            }
            JsonVal::Boolean(b) => {
                write!(f, "{}", b)?;
            }
            JsonVal::Null => {
                write!(f, "null")?;
            }
            JsonVal::Number(num) => match num {
                Number::Float(n) => {
                    write!(f, "{}", n)?;
                }
                Number::UnsignedInt(n) => {
                    write!(f, "{}", n)?;
                }
                Number::SignedInt(n) => {
                    write!(f, "{}", n)?;
                }
            },
        }
        Ok(())
    }
}

impl Display for JsonVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_impl(f, 0)
    }
}

fn parse_object(
    tokenizer: &mut Tokenizer<impl Iterator<Item = char>>,
) -> Result<JsonVal, ParsingError> {
    let mut map = IndexMap::new();
    loop {
        let token = tokenizer.next_token()?;
        match token.kind {
            TokenKind::ClosedBracket => {
                break Ok(JsonVal::Object(map));
            }
            TokenKind::Ident(_) => {
                let (ident, val) = parse_ident(token, tokenizer)?;
                map.insert(ident, val);
            }
            TokenKind::Comma => {} // Ignore
            _ => {
                break Err(ParsingError {
                    kind: ParsingErrorKind::UnexpectedToken,
                    loc: token.loc,
                });
            }
        }
    }
}

fn parse_array(
    tokenizer: &mut Tokenizer<impl Iterator<Item = char>>,
) -> Result<JsonVal, ParsingError> {
    let mut arr = Vec::new();
    loop {
        let token = tokenizer.next_token()?;
        match token.kind {
            TokenKind::ClosedSqBracket => {
                break Ok(JsonVal::Array(arr));
            }

            TokenKind::OpenBracket => arr.push(parse_object(tokenizer)?),
            TokenKind::OpenSqBracket => arr.push(parse_array(tokenizer)?),
            TokenKind::Val(_) => {
                arr.push(parse_val(token, tokenizer)?);
            }

            TokenKind::Ident(_) => {
                break Err(ParsingError {
                    kind: ParsingErrorKind::InvalidIdentInArray,
                    loc: token.loc,
                });
            }
            TokenKind::Comma => {} // Ignore
            _ => {
                break Err(ParsingError {
                    kind: ParsingErrorKind::UnexpectedToken,
                    loc: token.loc,
                });
            }
        }
    }
}

fn parse_string(str: String) -> Result<JsonVal, ParsingError> {
    Ok(JsonVal::String(str))
}

fn parse_val(
    val: Token,
    tokenizer: &mut Tokenizer<impl Iterator<Item = char>>,
) -> Result<JsonVal, ParsingError> {
    match val.kind {
        TokenKind::Val(str) => {
            let chars: Vec<_> = str.chars().collect();

            if chars[0] == '"' && chars[chars.len() - 1] == '"' {
                // This is a string
                parse_string(chars[1..chars.len() - 1].iter().collect())
            } else if chars.iter().all(is_num_char) {
                // This is a number
                if chars
                    .iter()
                    .any(|c| c.to_ascii_lowercase() == 'e' || *c == '.')
                {
                    // It is floating point
                    let s: String = chars.iter().collect();
                    let num: Result<f64, _> = s.parse();
                    if let Ok(num) = num {
                        Ok(JsonVal::Number(Number::Float(num)))
                    } else {
                        Err(ParsingError {
                            kind: ParsingErrorKind::InvalidToken,
                            loc: val.loc,
                        })
                    }
                } else {
                    // It is an int
                    let s: String = chars.iter().collect();
                    if *chars.first().unwrap() == '-' {
                        let num: Result<i64, _> = s.parse();
                        if let Ok(num) = num {
                            Ok(JsonVal::Number(Number::SignedInt(num)))
                        } else {
                            Err(ParsingError {
                                kind: ParsingErrorKind::InvalidToken,
                                loc: val.loc,
                            })
                        }
                    } else {
                        let num: Result<u64, _> = s.parse();
                        if let Ok(num) = num {
                            Ok(JsonVal::Number(Number::UnsignedInt(num)))
                        } else {
                            Err(ParsingError {
                                kind: ParsingErrorKind::InvalidToken,
                                loc: val.loc,
                            })
                        }
                    }
                }
            } else if chars.len() == 4 && chars.iter().zip("true".chars()).all(|(&a, b)| a == b) {
                Ok(JsonVal::Boolean(true))
            } else if chars.len() == 5 && chars.iter().zip("false".chars()).all(|(&a, b)| a == b) {
                Ok(JsonVal::Boolean(false))
            } else if chars.iter().zip("null".chars()).all(|(&a, b)| a == b) {
                Ok(JsonVal::Null)
            } else {
                Err(ParsingError {
                    kind: ParsingErrorKind::InvalidToken,
                    loc: val.loc,
                })
            }
        }
        TokenKind::OpenSqBracket => parse_array(tokenizer),
        TokenKind::OpenBracket => parse_object(tokenizer),
        _ => Err(ParsingError {
            kind: ParsingErrorKind::InvalidToken,
            loc: val.loc,
        }),
    }
}

pub fn parse_ident(
    ident: Token,
    tokenizer: &mut Tokenizer<impl Iterator<Item = char>>,
) -> Result<(String, JsonVal), ParsingError> {
    tokenizer.expect_token(TokenKind::Colon)?;
    let next_token = tokenizer.next_token()?;

    if let TokenKind::Ident(ident) = ident.kind {
        Ok((ident, parse_val(next_token, tokenizer)?))
    } else {
        unreachable!()
    }
}

pub fn parse(
    mut tokenizer: Tokenizer<impl Iterator<Item = char>>,
) -> Result<JsonVal, ParsingError> {
    let token = tokenizer.next_token()?;
    match token.kind {
        TokenKind::OpenBracket => parse_object(&mut tokenizer),
        TokenKind::OpenSqBracket => parse_array(&mut tokenizer),
        _ => Err(ParsingError {
            kind: ParsingErrorKind::InvalidStartingToken,
            loc: token.loc,
        }),
    }
}
