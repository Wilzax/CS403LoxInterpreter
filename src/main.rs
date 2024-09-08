//! Starts the interpreter's execution by managing the arguments passed at runtime
//! Possible aruments:
//!   0 - start in interactive mode using stdin
//!   1 - use first argument as filepath for code to interpret
//!   2 or more - undefined, should use 1st argument only and ignore the rest

use std::env;
mod scanner;

fn main() {
    
    //Print version and authors before executing
    println!("Lox interpreter v0.0.2:pre-scanning in Rust");
    println!("Written by Wilson King, Alicia Reed, Kiera Schnell, and Brodye Stevens\n");
    
    //Decide on interactive or file read mode
    let args: Vec<String> = env::args().collect();
    //println!("Detected {} main arguments", args.len());
    if args.len() < 2 {
        println!("No file supplied, starting in interactive mode...");
        //TODO: Call to scanner using stdio
        //dbg!(args);
        println!("No scanner implemented for interactive mode"); //Delete when scanner is implemented
    } 
    else {
        println!("Attempting to open file {}...", args[1]);

        //TODO: Call to scanner using input_file (remove leading underscore when implemented)
        println!("No scanner implemented for file read mode") //Delete when scanner is implemented
    }
    std::process::exit(1)
}
