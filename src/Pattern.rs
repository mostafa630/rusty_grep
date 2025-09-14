use std::{
    iter::Skip,
    str::{Chars, FromStr},
};

use crate::pattern_matcher;

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(char),
    CharClass(CharClass),
    GroupClass(GroupClass),
}

#[derive(Debug, PartialEq)]
pub enum CharClass {
    Digit,      // \d
    Identifier, // \w
}

#[derive(Debug, PartialEq)]
pub enum GroupClass {
    MatchOne(Vec<Token>),
    MatchNone(Vec<Token>),
}

impl Token {
    // take string match  try to match the token from the start of the str
    // if ok retuen the rest of the str
    pub fn _match<'a>(&self, str: &'a str) -> Option<&'a str> {
        match self {
            Self::Literal(c) if str.chars().next()? == *c => Some(skip(str, 1)),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Unclosed(String),      // e.g. missing ]
    InvalidEscape(String), // e.g. \q
    UnexpectedEof(String), // e.g. lone \
}

#[derive(Debug, PartialEq)]
pub struct Pattern {
    pub tokens: Vec<Token>,
}

impl FromStr for Pattern {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = vec![];
        let mut chars = s.chars();

        while let Some(token) = get_tokens(&mut chars)? {
            tokens.push(token);
        }
        Ok(Self { tokens })
    }
}

impl Pattern {
    pub fn match_input(&self, input_line: &str) -> bool {
        let mut rest = input_line;
        for token in &self.tokens {
            if let Some(remaining) = token._match(rest) {
                rest = remaining
            } else {
                return false;
            }
        }
        true
    }
}

fn get_tokens(chars: &mut Chars) -> Result<Option<Token>, ParseError> {
    match chars.next() {
        Some('\\') => match chars
            .next()
            .ok_or(ParseError::UnexpectedEof(format!("Expcted char after \\")))?
        {
            'd' => Ok(Some(Token::CharClass(CharClass::Digit))),
            'w' => Ok(Some(Token::CharClass(CharClass::Identifier))),
            '\\' => Ok(Some(Token::Literal('\\'))),
            c => {
                return Err(ParseError::InvalidEscape(format!(
                    "\\ doesn't allow {} after it",
                    c
                )))
            }
        },
        Some('[') => get_mathc_one_tokens(chars),
        Some(c) => Ok(Some(Token::Literal(c))),
        None => Ok(None),
    }
}

fn get_mathc_one_tokens(chars: &mut Chars) -> Result<Option<Token>, ParseError> {
    let mut tokens = vec![];
    let is_inverted = match get_next_char(chars) {
        Some('^') => {
            chars.next(); // consume the '^' character
            true
        }
        _ => false,
    };
    loop {
        match chars.next() {
            Some(']') => {
                if is_inverted {
                    break Ok(Some(Token::GroupClass(GroupClass::MatchNone(tokens))));
                } else {
                    break Ok(Some(Token::GroupClass(GroupClass::MatchOne(tokens))));
                }
            }
            Some(c) => tokens.push(Token::Literal((c))),
            None => return Err(ParseError::Unclosed(format!("Missing ]"))),
        }
    }
}
// get_next char and keeping the parser iterator as it was before calling this function
fn get_next_char(chars: &mut Chars) -> Option<char> {
    let mut clone = chars.clone();
    clone.next()
}
fn skip(s: &str, chars: usize) -> &str {
    let mut iter = s.chars();
    for _ in 0..chars {
        iter.next();
    }
    iter.as_str()
}
#[test]
fn test_parsing_digit_class() {
    let s = r"\d"; // raw string so the backslash is preserved
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::CharClass(CharClass::Digit)],
    };

    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_identifier_class() {
    let s = r"\w"; // raw string so the backslash is preserved
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::CharClass(CharClass::Identifier)],
    };
    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_literals() {
    let s = "abc";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![
            Token::Literal('a'),
            Token::Literal('b'),
            Token::Literal('c'),
        ],
    };

    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_match_one_class() {
    let s = "[abc]";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::GroupClass(GroupClass::MatchOne(vec![
            Token::Literal('a'),
            Token::Literal('b'),
            Token::Literal('c'),
        ]))],
    };
    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_combination() {
    let s = r"a\d[bc]\w";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![
            Token::Literal('a'),
            Token::CharClass(CharClass::Digit),
            Token::GroupClass(GroupClass::MatchOne(vec![
                Token::Literal('b'),
                Token::Literal('c'),
            ])),
            Token::CharClass(CharClass::Identifier),
        ],
    };

    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_match_none_class() {
    let s = "[^abc]";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::GroupClass(GroupClass::MatchNone(vec![
            Token::Literal('a'),
            Token::Literal('b'),
            Token::Literal('c'),
        ]))],
    };
    assert_eq!(parsed, expected);
}
