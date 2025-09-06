use std::env;
use std::io;
use std::process;

mod pattern_matcher;
use pattern_matcher::PatternMatcher;


fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let matcher = PatternMatcher {
        input_line : input_line.to_string(),
        pattern : pattern.to_string(),
    };
    if pattern.chars().count() == 1 {
         matcher.default()
    }else if pattern == "\\d" {
        matcher.match_any_digit()
    }else if pattern == "\\w" {
        matcher.match_non_specail_char()
    }else if  pattern.starts_with('[') && pattern.ends_with(']') && pattern.chars().count() > 2 {
        matcher.match_character_class()
    }else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    
    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
