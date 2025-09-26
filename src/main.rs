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
        let file_name = args[3].clone();
        process_file(file_name, pattern);
    }
    else{
        // process input from stdin
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        process_input_from_stdin(input_line, pattern);
    }
}

fn process_file(file_name:String , pattern: String){
let _file = _File::new(file_name);
    match _file {
        Ok(file)=>{
            let lines_matched = file.match_file(pattern.as_str());
            if lines_matched.len()!=0{
                print_lines(lines_matched);
                process::exit(0)
            }
             process::exit(1)
        },
        Err(e)=>{
            println!("{e}");
            process::exit(1)
        }
    }

    fn print_lines(lines_content : Vec<&String>){
        for line_content in lines_content{
            println!("{line_content}");
        }
    }
}

fn process_input_from_stdin(input_line : String ,pattern : String ){
 if match_input(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}