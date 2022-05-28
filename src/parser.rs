use crate::tokenizer::*;
use std::collections::HashMap;

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
    Object(HashMap<String, JsonVal>),
}

fn parse_object(tokenizer: &mut Tokenizer<impl Iterator<Item = char>>) -> JsonVal {
    let mut map = HashMap::new();
    loop {
        let token = tokenizer.next_token();
        match token.kind {
            TokenKind::ClosedBracket => {
                break JsonVal::Object(map);
            }
            TokenKind::Ident(_) => {
                let (ident, val) = parse_ident(token, tokenizer);
                map.insert(ident, val);
            }
            TokenKind::Comma => {} // Ignore
            _ => {
                unreachable!("Incorrect parsing, {:?}", token)
            }
        }
    }
}

fn parse_array(tokenizer: &mut Tokenizer<impl Iterator<Item = char>>) -> JsonVal {
    let mut arr = Vec::new();
    loop {
        let token = tokenizer.next_token();
        match token.kind {
            TokenKind::ClosedSqBracket => {
                break JsonVal::Array(arr);
            }
            TokenKind::OpenBracket => arr.push(parse_object(tokenizer)),
            TokenKind::Val(_) => {
                arr.push(parse_val(token, tokenizer));
            }
            TokenKind::Comma => {} // Ignore
            _ => {
                unreachable!("Incorrect parsing, {:?}", token)
            }
        }
    }
}

fn parse_val(val: Token, tokenizer: &mut Tokenizer<impl Iterator<Item = char>>) -> JsonVal {
    match val.kind {
        TokenKind::Val(str) => {
            let chars: Vec<_> = str.chars().collect();

            if chars[0] == '"' && chars[chars.len() - 1] == '"' {
                // This is a string
                JsonVal::String(chars[1..chars.len() - 1].iter().collect::<String>())
            } else if chars.iter().all(|c| {
                (!c.is_ascii_alphabetic() || c.to_ascii_lowercase() == 'e')
                    && (c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
            }) {
                // This is a number
                if chars
                    .iter()
                    .any(|c| c.to_ascii_lowercase() == 'e' || *c == '.')
                {
                    // It is floating point
                    let s: String = chars.iter().collect();
                    let num: f64 = s.parse().expect("Invalid number");
                    JsonVal::Number(Number::Float(num))
                } else {
                    // It is an int
                    let s: String = chars.iter().collect();
                    if *chars.first().unwrap() == '-' {
                        let num: i64 = s.parse().expect("Invalid number");
                        JsonVal::Number(Number::SignedInt(num))
                    } else {
                        let num: u64 = s.parse().expect("Invalid number");
                        JsonVal::Number(Number::UnsignedInt(num))
                    }
                }
            } else if chars.len() == 4 && chars.iter().zip("true".chars()).all(|(&a, b)| a == b) {
                JsonVal::Boolean(true)
            } else if chars.len() == 5 && chars.iter().zip("false".chars()).all(|(&a, b)| a == b) {
                JsonVal::Boolean(false)
            } else if chars.iter().zip("null".chars()).all(|(&a, b)| a == b) {
                JsonVal::Null
            } else {
                unreachable!("Invalid token text")
            }
        }
        TokenKind::OpenSqBracket => parse_array(tokenizer),
        TokenKind::OpenBracket => parse_object(tokenizer),
        _ => unreachable!("Invalid token: {:?}", val),
    }
}

pub fn parse_ident(
    ident: Token,
    tokenizer: &mut Tokenizer<impl Iterator<Item = char>>,
) -> (String, JsonVal) {
    let _ = tokenizer.expect_token(TokenKind::Colon);
    let next_token = tokenizer.next_token();

    if let TokenKind::Ident(ident) = ident.kind {
        (ident, parse_val(next_token, tokenizer))
    } else {
        unreachable!()
    }
}

pub fn parse(mut tokenizer: Tokenizer<impl Iterator<Item = char>>) -> JsonVal {
    let mut vals = HashMap::new();

    while let Some(token) = tokenizer.next() {
        match token.kind {
            TokenKind::Ident(_) => {
                let (ident, val) = parse_ident(token, &mut tokenizer);
                vals.insert(ident, val);
            }
            TokenKind::OpenBracket | TokenKind::ClosedBracket | TokenKind::Comma => {}
            _ => {
                unreachable!("Invalid token: {:?}", token)
            }
        }
    }

    JsonVal::Object(vals)
}
