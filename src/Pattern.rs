use std::str::FromStr;

enum PatternComponent {
    // Define the fields for PatternComponent here
    Literal(char) ,   //any hardcoded character
    Class(Class) , 
}
enum Class {
    Digit,   // \d
    Identifier, // \w
}
pub struct Pattern {
    pub components: Vec<PatternComponent>,
}

impl FromStr for Pattern {
   
    type Err = String; 
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!("Implement parsing logic here");
    }

}

