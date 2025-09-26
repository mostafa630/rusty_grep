use anyhow::Error;
use std::io::BufRead;
use std::result::Result::Ok;

use crate::pattern_matcher::match_input;
use std::{fs::File, io};
pub struct Line {
    content: String,
}

pub struct _File {
    name: String,
    lines: Vec<Line>,
}

impl _File {
    pub fn new(file_name: String) -> Result<Self, Error> {
        let file_res = File::open(&file_name);
        match file_res {
            Ok(file) => {
                let reader = io::BufReader::new(file);

                let lines_res: io::Result<Vec<Line>> = reader
                    .lines()
                    .map(|line_result| line_result.map(|line| Line { content: line }))
                    .collect();
                match lines_res {
                    Ok(lines) => Ok(_File {
                        name: file_name,
                        lines,
                    }),
                    _ => Err(Error::msg("failed to read file content")),
                }
            }
            _ => Err(Error::msg("failed to open that file")),
        }
    }
    pub fn match_file<'a>(&'a self, pattern: &str) -> Vec<&'a String> {
        let mut file_lines: Vec<&Line> = Vec::new();
        for line in &self.lines {
            if line.match_line(pattern) {
                file_lines.push(line);
            }
        }
        get_lines_content(&file_lines)
    }
}


impl Line {
    fn match_line(&self, pattern: &str) -> bool {
        match_input(&self.content, pattern)
    }
}

fn get_lines_content<'a>(lines: &[&'a Line]) -> Vec<&'a String> {
    let mut file_content = vec![];
    for line in lines {
        file_content.push(&line.content);
    }
    file_content
}