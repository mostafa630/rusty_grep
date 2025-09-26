use std::env;
use std::io;
use std::process;

mod Pattern;
mod pattern_matcher;

mod File;

use pattern_matcher::match_input;

use crate::File::_File;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "-E" {
        println!("the input is not correct");
        process::exit(1);
    }

    let pattern = args[2].clone();

    if args.len() >=4 {
        let files = get_files_names_from_args(args);
        process_files(files, pattern);
    }
    else{
        // process input from stdin
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        process_input_from_stdin(input_line, pattern);
    }
}


fn process_files(files_names: Vec<String>, pattern: String) {
    let multiple_files = files_names.len() > 1;
    let mut any_match = false;

    for file_name in files_names {
        if let Ok(file) = _File::new(file_name.clone()) {
            let lines_matched = file.match_file(pattern.as_str()); // Vec<&String>
            if !lines_matched.is_empty() {
                any_match = true;
                for line in lines_matched {
                    if multiple_files {
                        println!("{file_name}:{line}");
                    } else {
                        println!("{line}");
                    }
                }
            }
        }
    }

    std::process::exit(if any_match { 0 } else { 1 });
}

fn print_lines(lines_content: Vec<&String>, file_name: String) {
    for line_content in lines_content {
        if file_name.is_empty() {
            println!("{line_content}");
        } else {
            println!("{file_name}:{line_content}");
        }
    }
}
fn get_files_names_from_args(args : Vec<String>) -> Vec<String>{
    let mut files_names  =vec![];
    for file_name in &args[3..]{
        files_names.push(file_name.clone());
    }
    files_names
}

fn process_input_from_stdin(input_line : String ,pattern : String  ,){
 if match_input(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}