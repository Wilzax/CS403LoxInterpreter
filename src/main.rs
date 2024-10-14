mod scanner;
mod expr;
mod parser;
mod interpreter;
mod stmt;
mod environment;
mod lox_callable;
use std::env::args;
use std::fs::File;
use std::io::Read;

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
        let file_path = &args[1];
        println!("Attempting to open file {}...", file_path);

        let input_file_result = File::open(file_path);

        let mut input_file = match input_file_result { 
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {error:?}"),
        };

        let mut file_contents = String::new();
        match input_file.read_to_string(&mut file_contents) {
            Ok(_) => {
                scanner::run(file_contents);
            },
            Err(error) => panic!("Problem reading the file: {error:?}"),
        };
    }

    println!("Thus ends the program.");
}
