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
    pub fn match_character_class(&self)->bool {
        // get chars between [ ]   input [abc]  -> vec!['a','b','c']
        let chars : Vec<char> = self.pattern[1..self.pattern.len()-1].chars().collect(); 
        self.input_line.chars().any(|c| chars.contains(&c)) // check if any char in input_line is in chars
    }
}