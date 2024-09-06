use std::env;
use std::fs;
use std::process;
use std::io::Error;
use std::str;
use text_io::{read, scan};

fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() > 2{
        println!("Usage: rlox [script]");
        process::exit(0);
    }
    else if args.len() == 2{
        run_file(args[0].clone());
    }
    else{
        run_prompt();
    }
}

pub(crate) fn run_file(file_path: String){
    let file_contents: Result<String, Error> = fs::read_to_string(file_path.clone());
    let file_contents: String = match file_contents{
        Ok(file_string) => file_string,
        Err(error) => panic!("Problem opening the file: {error:?}")
    };
    run(file_contents);
}

pub(crate) fn run_prompt(){
    loop{
        println!("> ");
        let mut line: String = read!("{}\n");
        if line.trim().is_empty(){
            break;
        }
        run(line)
    }
}

pub(crate) fn run(source: String){
    
}

pub fn error(line: i32, message: String){
    report(line, "".to_string(), message);
}

pub(crate) fn report(line: i32, whereLocation: String, message: String){
    
}