pub struct PatternMatcher {
    pub input_line: String,
    pub pattern: String,
}
impl PatternMatcher {
    pub fn default(&self) -> bool {
        self.input_line.contains(&self.pattern)
    }
    pub fn match_any_digit(&self)-> bool {
        self.input_line.chars().any(|c| c.is_ascii_digit())
    }
    pub fn match_non_specail_char(&self)->bool{
        self.input_line.chars().any(|c| c.is_alphanumeric() || c == '_')
    }
}