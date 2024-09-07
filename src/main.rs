/**********
 * Starts the interpreter's execution by managing the arguments passed at runtime
 * Possible aruments:
 *  none: start in interactive mode using stdin
 *  1 or more: use first argument as filepath for code to interpret, ignore other arguments
 **********/

use std::env;
use std::fs::File;

mod scanner;

fn main() {
    println!("Lox interpreter in Rust");
    println!("Written by Wilson King, Alicia Reed, Kiera Schnell, and Brodye Stevens\n");

    
    let args: Vec<String> = env::args().collect();
    //println!("Detected {} main arguments", args.len());
    if args.len() < 2 {
        println!("No file supplied, starting in interactive mode...");
        //TODO: Call to scanner using stdio
        //dbg!(args);
        panic!("No scanner implemented for interactive mode") //Delete when scanner is implemented
    } 
    else {
        let input_file_result:Result<File, std::io::Error> = File::open(args[1].clone());
        println!("Attempting to open file {}...", args[1]);
        
        /* From https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html */
        let _input_file = match input_file_result { 
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {error:?}"),
        };
        /* End citation */

        //TODO: Call to scanner using input_file (remove leading underscore when implemented)
        panic!("No scanner implemented for file read mode") //Delete when scanner is implemented
    }
}
