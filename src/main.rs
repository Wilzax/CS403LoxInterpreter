mod scanner;
mod expr;
mod parser;
mod interpreter;
mod stmt;
mod environment;
mod lox_callable;
use std::env::args;
use std::fs::File;

fn main() {
    println!("");
    scanner::run("fun sayHi(x) {{return 10;}}\nprint sayHi(12);".to_string());

    let args: Vec<String> = args().collect();
    println!("Detected {} main arguments", args.len());
    if args.len() < 2 {
        println!("No file supplied, starting in interactive mode...");
        dbg!(args);
    } 
    else {
        let input_file_result = File::open(args[1].clone());
        println!("Attempting to open file {}...", args[1]);
        
        //From https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
        let _input_file = match input_file_result { 
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {error:?}"),
        };
        //End citation
    }
    println!("Thus ends the program.")
}
