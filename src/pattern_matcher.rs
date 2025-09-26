use crate::Pattern::Pattern;

pub struct PatternMatcher {
    pub input_line: String,
    pub pattern: String,
}
impl PatternMatcher {
    pub fn match_pattern(&self) -> bool {
        let pattern_engine: Pattern = self.pattern.parse().unwrap();
        pattern_engine.matches(self.input_line.as_str())
    }
}

#[test]
fn test_match_pattern_on_literals() {
    let pattern_matcher = PatternMatcher {
        pattern: "abc".to_string(),
        input_line: "abc".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}
#[test]
fn test_match_pattern_on_digits() {
    let pattern_matcher = PatternMatcher {
        pattern: "\\d\\d".to_string(),
        input_line: "12".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}
#[test]
fn test_match_pattern_on_identifier() {
    let pattern_matcher = PatternMatcher {
        pattern: "\\w\\w\\w".to_string(),
        input_line: "a_Z".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}

#[test]
fn test_match_pattern_on_sol() {
    let pattern_matcher = PatternMatcher {
        pattern: "^abc\\d\\wfg\\d".to_string(),
        input_line: "abc5_fg5".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}
#[test]
fn test_match_pattern_on_eol() {
    let pattern_matcher = PatternMatcher {
        pattern: "abc\\d\\wfg\\d$".to_string(),
        input_line: "sadasd135abc5_fg5".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}

#[test]
fn test_match_pattern_on_combinations() {
    let pattern_matcher = PatternMatcher {
        pattern: "ac\\ddg\\w\\wf\\w".to_string(),
        input_line: "ac5dga_fW".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}
#[test]
fn test_match_pattern_on_exact_match() {
    let pattern_matcher = PatternMatcher {
        pattern: "^strawberry$".to_string(),
        input_line: "strawberry".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}

#[test]
fn test_match_pattern_on_one_or_more() {
    let pattern_matcher = PatternMatcher {
        pattern: "ca+t".to_string(),
        input_line: "act".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), false);
}

#[test]
fn test_match_pattern_on_one_or_none() {
    let pattern_matcher = PatternMatcher {
        pattern: "ca?at".to_string(),
        input_line: "cat".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}

#[test]
fn test_match_pattern_on_wild_card() {
    let pattern_matcher = PatternMatcher {
        pattern: "g.+gol".to_string(),
        input_line: "goøö0Ogol".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}

#[test]
fn test_match_pattern_on_alternation() {
    let pattern_matcher = PatternMatcher {
        pattern: "(cat|dog)".to_string(),
        input_line: "cat".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}

#[test]
fn test_match_pattern_on_one_or_more_digit(){
    let pattern_matcher = PatternMatcher {
        pattern: "^I see \\d+ (cat|dog)s?$".to_string(),
        input_line: "I see 42 dogs".to_string(),
    };
    assert_eq!(pattern_matcher.match_pattern(), true);
}
#[test]
fn test_trigger_hiiii() {
    let s = "aaa";
    let parsed: Pattern = s.parse().unwrap();
    assert!(parsed.matches("aaa"));
}
