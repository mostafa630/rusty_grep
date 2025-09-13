use std::str::{Chars, FromStr};

use anyhow::Error;

#[derive(Debug, PartialEq)]
pub enum Token {
    // Define the fields for PatternComponent here
    Literal(char), //any hardcoded character
    Class(Class),
    MatchOneClass(Vec<Self>),
}
#[derive(Debug, PartialEq)]
pub enum Class {
    Digit,      // \d
    Identifier, // \w
}

#[derive(Debug, PartialEq)]
pub struct Pattern {
    pub tokens: Vec<Token>,
}

impl FromStr for Pattern {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = vec![];
        let mut chars = s.chars();

        while let Some(token) = get_tokens(&mut chars)? {
            tokens.push(token)
        }
        Ok(Self { tokens })
    }
}
fn get_tokens(chars: &mut Chars) -> Result<Option<Token>, String> {
    match chars.next() {
        Some('\\') => match chars.next().ok_or("Expected another character after \\")? {
            'd' => Ok(Some(Token::Class(Class::Digit))),
            'w' => Ok(Some(Token::Class(Class::Identifier))),
            '\\' => Ok(Some(Token::Literal('\\'))),
            c => return Err(format!("{} is not Accepted After \\", c)),
        },
        Some('[') => get_mathc_one_class_tokens(chars),
        Some(c) => Ok(Some(Token::Literal(c))),
        None => Ok(None),
    }
}

fn get_mathc_one_class_tokens(chars: &mut Chars) -> Result<Option<Token>, String> {
    let mut tokens = vec![];

    loop {
        match chars.next() {
            Some(']') => break Ok(Some(Token::MatchOneClass(tokens))),
            Some(c) => tokens.push(Token::Literal((c))),
            None => return Err(format!("Can't Find a Closing  ']'")),
        }
    }
}

#[test]
fn test_parsing_digit_class() {
    let s = r"\d"; // raw string so the backslash is preserved
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::Class(Class::Digit)],
    };

    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_identifier_class() {
    let s = r"\w"; // raw string so the backslash is preserved
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::Class(Class::Identifier)],
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
    let s = "[def]";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::MatchOneClass(vec![
            Token::Literal('d'),
            Token::Literal('e'),
            Token::Literal('f'),
        ])],
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
            Token::Class(Class::Digit),
            Token::MatchOneClass(vec![Token::Literal('b'), Token::Literal('c')]),
            Token::Class(Class::Identifier),
        ],
    };

    assert_eq!(parsed, expected);
}
