use std::str::{Chars, FromStr};

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(char), 
    Class(Class),
    MatchOne(Vec<Self>),
    MatchNone(Vec<Self>),
}
#[derive(Debug, PartialEq)]
pub enum Class {
    Digit,      // \d
    Identifier, // \w
}

#[derive(Debug)]
pub enum ParseError {
    Unclosed(String), // e.g. missing ]
    InvalidEscape(String), // e.g. \q
    UnexpectedEof(String),       // e.g. lone \
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

        while let Some(token) = get_tokens(&mut chars)?{
            tokens.push(token);
        }
        Ok(Self { tokens })
    }
}
fn get_tokens(chars: &mut Chars) -> Result<Option<Token>, ParseError> {
    match chars.next() {
        Some('\\') => match chars.next().ok_or(ParseError::UnexpectedEof(format!("Expcted char after \\")))? {
            'd' => Ok(Some(Token::Class(Class::Digit))),
            'w' => Ok(Some(Token::Class(Class::Identifier))),
            '\\' => Ok(Some(Token::Literal('\\'))),
            c => return Err(ParseError::InvalidEscape(format!("\\ doesn't allow {} after it" , c))),
        },
        Some('[') => get_mathc_one_class_tokens(chars),
        Some(c) => Ok(Some(Token::Literal(c))),
        None => Ok(None),
    }
}

fn get_mathc_one_class_tokens(chars: &mut Chars) -> Result<Option<Token>, ParseError> {
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
            Some(']') =>{ 
                if is_inverted {
                    break Ok(Some(Token::MatchNone(tokens)))
                }
                 else{
                    break Ok(Some(Token::MatchOne(tokens)))
                 }
            },
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
    let s = "[abc]";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::MatchOne(vec![
            Token::Literal('a'),
            Token::Literal('b'),
            Token::Literal('c'),
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
            Token::MatchOne(vec![Token::Literal('b'), Token::Literal('c')]),
            Token::Class(Class::Identifier),
        ],
    };

    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_match_none_class() {
    let s = "[^xyz]";
    let parsed : Pattern = s.parse().unwrap();
    
    let expected = Pattern { 
        tokens : vec![
            Token::MatchNone(vec![
                Token::Literal('x'),
                Token::Literal('y'),
                Token::Literal('z'),
            ])
        ]
    };
    assert_eq!(parsed, expected);
}
