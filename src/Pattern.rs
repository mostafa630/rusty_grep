use std::{
    str::{Chars, FromStr},
    vec,
};

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(char),
    CharClass(CharClass),
    GroupClass(GroupClass),
    SOL(Vec<Token>),   // Start Of Line
    EOL(Vec<Token>),   // End Of Line
    Exact(Vec<Token>), // ^....$
    OneORMore(Box<Token>),
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
#[derive(Debug, PartialEq)]
pub enum Anchor {
    Start, // ^abc
    End,   // abc$
    Both,  // ^abc$
}

#[derive(Debug)]
pub enum Remaining<'a> {
    Single(Option<&'a str>),
    Multiple(Vec<Option<&'a str>>),
}

impl Token {
    

    // ------------------------------------------------------------------------------//
    //                                 Token Matcher                                 //
    // ------------------------------------------------------------------------------//

    pub fn _match<'a>(&self, str: &'a str) -> Option<Remaining<'a>> {
        match self {
            Self::Literal(c) if str.chars().next()? == *c => {
                Some(Remaining::Single(Some(skip(str, 1))))
            }

            Self::CharClass(char_class)
                if Self::match_char_class(char_class, str.chars().next()?) =>
            {
                Some(Remaining::Single(Some(skip(str, 1))))
            }
            Self::SOL(sub_tokens) => Some(Remaining::Single(Self::match_group(
                sub_tokens.as_slice(),
                str,
            ))),
            Self::EOL(sub_tokens) => Some(Remaining::Single(Self::match_group(
                sub_tokens.as_slice(),
                str,
            ))),
            Self::Exact(sub_tokens) => Some(Remaining::Single(Self::match_group(
                sub_tokens.as_slice(),
                str,
            ))),
            Self::OneORMore(token) => Self::match_one_or_more(token, str),
            _ => None,
        }
    }

    fn match_char_class(class: &CharClass, c: char) -> bool {
        match class {
            CharClass::Digit => c.is_ascii_digit(),
            CharClass::Identifier => c.is_ascii_alphabetic() || c == '_',
        }
    }

    fn match_one_or_more<'a>(literal_token: &Box<Token>, s: &'a str) -> Option<Remaining<'a>> {
        let mut remainings: Vec<Option<&'a str>> = Vec::new();

        // First, try matching the token at least once
        if let Some(rem) = literal_token._match(s) {
            match rem {
                Remaining::Single(remaining_str) => remainings.push(remaining_str),
                _ => return None,
            }
        } else {
            return None; // doesn't match even one
        }

        while let Some(last) = remainings.last().and_then(|opt| *opt) {
            if let Some(rem) = literal_token._match(last) {
                match rem {
                    Remaining::Single(remaining_str) => remainings.push(remaining_str),
                    _ => break,
                }
            } else {
                break;
            }
        }

        Some(Remaining::Multiple(remainings))
    }

    fn match_group<'a>(tokens: &[Token], str: &'a str) -> Option<&'a str> {
        if tokens.is_empty() {
            return Some(str);
        }

        let (current_token, remaning_tokens) = tokens.split_first().unwrap();
        if let Some(Remaining::Single(remaining_str)) = current_token._match(str) {
            if let Some(remaining_str) = remaining_str {
                Self::match_group(remaning_tokens, remaining_str)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Unclosed(String),      // e.g. missing ]
    InvalidEscape(String), // e.g. \q
    UnexpectedEof(String), // e.g. alone \
    InvalidPattern(String),
    InvalidAnchorType,
}

#[derive(Debug, PartialEq)]
pub struct Pattern {
    pub tokens: Vec<Token>,
}


// ----------------------- Start point of Parsing ---------------------------------- //     
impl FromStr for Pattern {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = vec![];
        let mut chars = s.chars();
        //start Parsing
        while let Some(token) = Self::get_tokens(&mut chars)? {
            tokens.push(token);
        }
        Ok(Self { tokens })
    }
}

impl Pattern {


    // ------------------------------------------------------------------------------//
    //                                 Parsing Logic                                 //
    // ------------------------------------------------------------------------------//

    // main Parsing Function
    fn get_tokens(chars: &mut Chars) -> Result<Option<Token>, ParseError> {
        // check if SOL
        let last_char = chars.as_str().chars().last();
        let first_char = chars.as_str().chars().nth(0);
        if let Some('$') = last_char {
            if let Some('^') = first_char {
                // Exact
                let s1 = remove_char_at(0, chars);
                let s2 = remove_char_at(s1.len() - 1, &mut s1.chars());
                return Self::get_anchor_tokens(&mut s2.chars(), Anchor::Both);
            }
            //EOL
            return Self::get_anchor_tokens(
                &mut remove_char_at(chars.as_str().len() - 1, chars).chars(),
                Anchor::End,
            );
        }
        //if not SOL do normal Parsing
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
            Some('^') => Self::get_anchor_tokens(chars, Anchor::Start),
            Some('[') => Self::get_group_tokens(chars),
            Some(c) => {
                let next_char = get_next_char(chars);
                match next_char {
                    Some('+') => {
                        chars.next(); // consume +
                        Ok(Some(Token::OneORMore(Box::new(Token::Literal(c)))))
                    }
                    _ => Ok(Some(Token::Literal(c))),
                }
            }
            None => Ok(None),
        }
    }

    fn get_group_tokens(chars: &mut Chars) -> Result<Option<Token>, ParseError> {
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
                Some(c) => tokens.push(Token::Literal(c)),
                None => return Err(ParseError::Unclosed(format!("Missing ]"))),
            }
        }
    }

    fn get_anchor_tokens(
        chars: &mut Chars,
        AnchorType: Anchor,
    ) -> Result<Option<Token>, ParseError> {
        let mut tokens = vec![];
        while let Some(token) = Self::get_tokens(chars)? {
            tokens.push(token);
        }
        if tokens.len() == 0 {
            return Err(ParseError::InvalidPattern(
                "No thing after ^ or nothing before$".to_string(),
            ));
        } else {
            match AnchorType {
                Anchor::Start => Ok(Some(Token::SOL(tokens))),
                Anchor::End => {
                    tokens.reverse();
                    Ok(Some(Token::EOL(tokens)))
                }
                Anchor::Both => Ok(Some(Token::Exact(tokens))),
                _ => Err(ParseError::InvalidAnchorType),
            }
        }
    }

    // ------------------------------------------------------------------------------//
    //                                 Matching Logic                                //
    // ------------------------------------------------------------------------------//

    // start point of matching
    pub fn matches(&self, input: &str) -> bool {
        // just a Closure to call it  in the next match
        let exhaustive_mathc = |input: &str| -> bool {
            (0..input.len())
                .map(|offset| &input[offset..])
                .any(|input| self.match_str(input))
        };

        match &self.tokens[0] {
            Token::SOL(_) => self.match_str(input),
            Token::EOL(_) => {
                let reversed_str: String = input.chars().rev().collect();
                self.match_str(reversed_str.as_str())
            }
            Token::Exact(exact_tokens) => {
                println!("{:?}", &self.tokens);
                if exact_tokens.len() != input.len() {
                    return false;
                } else {
                    self.match_str(input)
                }
            }
            _ => exhaustive_mathc(input),
        }
    }

    fn match_str(&self, input_line: &str) -> bool {
        fn match_tokens(tokens: &[Token], input: &str) -> bool {
            if tokens.is_empty() {
                return true;
            }

            let (first, rest_tokens) = tokens.split_first().unwrap();
            println!("First token = {:?}", first);
            println!("input = {}", input);
            if let Some(remaining) = first._match(input) {
                match remaining {
                    Remaining::Single(Some(remaining_str)) => {
                        return match_tokens(rest_tokens, remaining_str)
                    }
                    Remaining::Multiple(remaining_strs) => {
                        for remaining in remaining_strs {
                            if let Some(remaining_str) = remaining {
                                if match_tokens(rest_tokens, remaining_str) {
                                    return true; // found a successful match
                                }
                            }
                        }
                        return false;
                    }

                    _ => return false,
                }
                // Try all remaining options
            }
            false // no match found
        }

        match_tokens(&self.tokens, input_line)
    }
}
// Parser Start Point

fn remove_char_at<'a>(idx: usize, chars: &'a mut Chars) -> String {
    let mut str: String = chars.collect();
    str.remove(idx);
    str
}
// get_next char and keeping the parser iterator as it was before calling this function
fn get_next_char(chars: &mut Chars) -> Option<char> {
    let mut clone = chars.clone();
    clone.next()
}
fn skip<'a>(s: &'a str, chars: usize) -> &'a str {
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
fn test_parsing_one_class() {
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
fn test_parsing_sol() {
    let s = "^abc\\d\\w";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![Token::SOL(vec![
            Token::Literal('a'),
            Token::Literal('b'),
            Token::Literal('c'),
            Token::CharClass(CharClass::Digit),
            Token::CharClass(CharClass::Identifier),
        ])],
    };
    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_eol() {
    let s = "abc\\d\\w$";
    let parsed: Pattern = s.parse().unwrap();
    let expected = Pattern {
        tokens: vec![Token::EOL(vec![
            Token::CharClass(CharClass::Identifier),
            Token::CharClass(CharClass::Digit),
            Token::Literal('c'),
            Token::Literal('b'),
            Token::Literal('a'),
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
fn test_parsing_none_class() {
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
#[test]
fn test_parsing_one_or_more() {
    let s = "abc+\\w\\d";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        tokens: vec![
            Token::Literal('a'),
            Token::Literal('b'),
            Token::OneORMore(Box::new(Token::Literal('c'))),
            Token::CharClass(CharClass::Identifier),
            Token::CharClass(CharClass::Digit),
        ],
    };

    assert_eq!(parsed, expected);
}
