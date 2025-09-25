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
    OneOrNone(Box<Token>),
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
            Self::Literal(c) if str.chars().next()? == *c || *c == '.' => {
                // (*c == '.') we add tatht to support wildcard
                Some(Remaining::Single(Some(skip(str, 1))))
            }
            //Self::WildCard => Some(Remaining::Single(Some(skip(str, 1)))),
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
            Self::OneOrNone(token) => Self::match_one_or_none(token, str),

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

    fn match_one_or_none<'a>(literal_token: &Box<Token>, s: &'a str) -> Option<Remaining<'a>> {
        let mut remainings: Vec<Option<&'a str>> = Vec::new();

        match literal_token._match(s) {
            // if the char matc if we consume directly that can lead to problem
            // if that is the case :   pattern = ca?at   and input = cat
            // so we will return  vector or remaning strs  and in our case  it will contain 2 str
            // one if we will consume and one if we won't consue
            Some(remaning) => {
                remainings.push(Some(s));
                if let Remaining::Single(remaining_str) = remaning {
                    remainings.push(remaining_str);
                };
                Some(Remaining::Multiple(remainings))
            }
            // if the char doesn't match that is fine and return the input with no consuing for any char
            None => Some(Remaining::Single(Some(s))),
        }
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
    pub sub_patterns: Vec<SubPattern>,
}

#[derive(Debug, PartialEq)]
pub struct SubPattern {
    pub tokens: Vec<Token>,
}

// ----------------------- Start point of Parsing ---------------------------------- //
impl FromStr for Pattern {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sub_patterns_strs: Vec<String> = Self::expand_pattern(s);

        let mut sub_patterns = vec![];
        println!("subpatterns = {:?}", sub_patterns_strs);

        for pattern in sub_patterns_strs {
            let mut sub_pattern_tokens = vec![];
            let mut chars = pattern.chars();
            //start Parsing
            while let Some(token) = Self::get_tokens(&mut chars)? {
                sub_pattern_tokens.push(token);
            }
            sub_patterns.push(SubPattern {
                tokens: sub_pattern_tokens,
            });
        }

        Ok(Self { sub_patterns })
    }
}

impl Pattern {
    fn expand_pattern(pattern: &str) -> Vec<String> {
        fn helper(s: &str) -> Vec<String> {
            if let Some(open) = s.find('(') {
                let close = s[open..]
                    .find(')')
                    .map(|i| open + i)
                    .expect("Unmatched '('");

                let before = &s[..open];
                let inside = &s[open + 1..close];
                let after = &s[close + 1..];

                let choices: Vec<&str> = if inside.contains('|') {
                    inside.split('|').collect()
                } else {
                    vec![inside]
                };

                let mut results = vec![];
                for choice in choices {
                    for rest in helper(after) {
                        results.push(format!("{}{}{}", before, choice, rest));
                    }
                }
                results
            } else {
                vec![s.to_string()]
            }
        }

        helper(pattern)
    }

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
            Some('\\') => {
                let mut token;
                match chars
                    .next()
                    .ok_or(ParseError::UnexpectedEof(format!("Expcted char after \\")))?
                {
                    'd' => token = Token::CharClass(CharClass::Digit),
                    'w' => token = Token::CharClass(CharClass::Identifier),
                    '\\' => token = Token::Literal('\\'),
                    c => {
                        return Err(ParseError::InvalidEscape(format!(
                            "\\ doesn't allow {} after it",
                            c
                        )))
                    }
                }
                let next_char = get_next_char(chars);
                if let Some(char) = next_char {
                    match char {
                        '+' => {
                            chars.next();
                            Ok(Some(Token::OneORMore(Box::new(token))))
                        },
                        '?' => {
                            chars.next();
                            Ok(Some(Token::OneOrNone(Box::new(token))))
                        },
                        _ => Ok(Some(token)),
                    }
                } else {
                    Ok(Some(token))
                }
            }
            Some('^') => Self::get_anchor_tokens(chars, Anchor::Start),
            Some('[') => Self::get_group_tokens(chars),
            Some(c) => {
                let next_char = get_next_char(chars);
                match next_char {
                    Some('+') => {
                        chars.next(); // consume +
                        Ok(Some(Token::OneORMore(Box::new(Token::Literal(c)))))
                    }
                    Some('?') => {
                        chars.next(); // consume ?
                        Ok(Some(Token::OneOrNone(Box::new(Token::Literal(c)))))
                    }
                    _ => Ok(Some(Token::Literal(c))),
                }
            }
            //Some('.') => Ok(Some(Token::WildCard)),
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
        let mut sub_pattern_matched = false;
        for sub_pattern in &self.sub_patterns {
            println!("input now ============================================== {input}");
            // just a Closure to call it  in the next match
            let exhaustive_match = |input: &str| -> bool {
                input
                    .char_indices()
                    .map(|(i, _)| &input[i..])
                    .any(|substr| sub_pattern.match_str(substr).0)
            };

            match &sub_pattern.tokens[0] {
                Token::SOL(_) => sub_pattern_matched |= sub_pattern.match_str(input).0,
                Token::EOL(_) => {
                    let reversed_str: String = input.chars().rev().collect();
                    sub_pattern_matched |= sub_pattern.match_str(reversed_str.as_str()).0;
                }
                Token::Exact(_) => {
                 let (mathced , remaining_str) = sub_pattern.match_str(input);
                 if remaining_str.len() != 0 {
                    sub_pattern_matched |= false;
                 }
                 else {
                    sub_pattern_matched |= mathced;
                 }                        
                }
                _ => sub_pattern_matched |= exhaustive_match(input),
            };

            if sub_pattern_matched {
                return true; // stop at first success
            }
        }
        false
    }
}

impl SubPattern {
    fn match_str<'a>(&self, input_line: &'a str) -> (bool , &'a str) {
        fn match_tokens<'a>(tokens: &[Token], input: &'a str) -> (bool, &'a str){
            if tokens.is_empty() {
                return (true , input);
            }

            let (first, rest_tokens) = tokens.split_first().unwrap();
            println!("First token = {:?}", first);
            println!("input = {}", input);
            let mut final_remaning : &str =""; 
            if let Some(remaining) = first._match(input) {
                match remaining {
                    Remaining::Single(Some(remaining_str)) => {
                        final_remaning = remaining_str;
                        return match_tokens(rest_tokens, remaining_str)
                    }
                    Remaining::Multiple(remaining_strs) => {
                        for remaining in remaining_strs {
                            if let Some(remaining_str) = remaining {
                                final_remaning = remaining_str;
                                let (matched, new_remaning_str) = match_tokens(rest_tokens, remaining_str);
                                if matched {
                                    return (true ,new_remaning_str); // found a successful match
                                }
                            }
                        }
                        return (false , final_remaning);
                    }
                    
                    _ => {
                        return (false , final_remaning)
                    },
                }
                // Try all remaining options
            }
            (false ,final_remaning) // no match found
        }

        match_tokens(&self.tokens, input_line)
    }
}

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

// ------------------------------------------------------------------------------//
//                                 Parsing Tests                                 //
// ------------------------------------------------------------------------------//

#[test]
fn test_parsing_digit_class() {
    let s = r"\d"; // raw string so the backslash is preserved
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![Token::CharClass(CharClass::Digit)],
        }],
    };
    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_identifier_class() {
    let s = r"\w"; // raw string so the backslash is preserved
    let parsed: Pattern = s.parse().unwrap();
    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![Token::CharClass(CharClass::Identifier)],
        }],
    };
    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_literals() {
    let s = "abc";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
            ],
        }],
    };

    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_one_class() {
    let s = "[abc]";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![Token::GroupClass(GroupClass::MatchOne(vec![
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
            ]))],
        }],
    };
    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_sol() {
    let s = "^abc\\d\\w";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![Token::SOL(vec![
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
                Token::CharClass(CharClass::Digit),
                Token::CharClass(CharClass::Identifier),
            ])],
        }],
    };

    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_eol() {
    let s = "abc\\d\\w$";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![Token::EOL(vec![
                Token::CharClass(CharClass::Identifier),
                Token::CharClass(CharClass::Digit),
                Token::Literal('c'),
                Token::Literal('b'),
                Token::Literal('a'),
            ])],
        }],
    };

    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_combination() {
    let s = r"a\d[bc]\w";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![
                Token::Literal('a'),
                Token::CharClass(CharClass::Digit),
                Token::GroupClass(GroupClass::MatchOne(vec![
                    Token::Literal('b'),
                    Token::Literal('c'),
                ])),
                Token::CharClass(CharClass::Identifier),
            ],
        }],
    };

    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_none_class() {
    let s = "[^abc]";
    let parsed: Pattern = s.parse().unwrap();
    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![Token::GroupClass(GroupClass::MatchNone(vec![
                Token::Literal('a'),
                Token::Literal('b'),
                Token::Literal('c'),
            ]))],
        }],
    };

    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_one_or_more() {
    let s = "abc+\\w\\d";
    let parsed: Pattern = s.parse().unwrap();
    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![
                Token::Literal('a'),
                Token::Literal('b'),
                Token::OneORMore(Box::new(Token::Literal('c'))),
                Token::CharClass(CharClass::Identifier),
                Token::CharClass(CharClass::Digit),
            ],
        }],
    };

    assert_eq!(parsed, expected);
}
#[test]
fn test_parsing_one_or_none() {
    let s = "abc?\\w\\d";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![SubPattern {
            tokens: vec![
                Token::Literal('a'),
                Token::Literal('b'),
                Token::OneOrNone(Box::new(Token::Literal('c'))),
                Token::CharClass(CharClass::Identifier),
                Token::CharClass(CharClass::Digit),
            ],
        }],
    };

    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_alternation() {
    let s = "cat|dog";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![
            SubPattern {
                tokens: vec![
                    Token::Literal('c'),
                    Token::Literal('a'),
                    Token::Literal('t'),
                ],
            },
            SubPattern {
                tokens: vec![
                    Token::Literal('d'),
                    Token::Literal('o'),
                    Token::Literal('g'),
                ],
            },
        ],
    };
    assert_eq!(parsed, expected);
}

#[test]
fn test_parsing_one_or_more_digit() {
    let s = "a\\d+c";
    let parsed: Pattern = s.parse().unwrap();

    let expected = Pattern {
        sub_patterns: vec![
            SubPattern {
                tokens: vec![
                    Token::Literal('a'),
                    Token::OneORMore(Box::new(Token::CharClass(CharClass::Digit))),
                    Token::Literal('c'),
                ],
            },
        ],
    };
    assert_eq!(parsed, expected);
}


