mod scanner;
mod expr;
mod parser;
mod interpreter;
mod stmt;
mod environment;
mod lox_callable;
mod resolver;
mod lox_instance;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

fn main() {
    println!("");
    //scanner::run("fun fib(n) {{\nif (n <= 1) return n;\nreturn fib(n-2) + fib(n-1);}}\nfor (var i = 0; i < 21; i = i + 1)\n{{print fib(i);}}".to_string());
    //scanner::run("class Bagel{}\nvar bag = Bagel();\nbag.name = 4;\nprint bag.name;".to_string());
    //scanner::run("var hi = 2; hi = 10; print hi;".to_string());
    //scanner::run("for (var i = 0; i < 10; i = i + 1)\n{{print i;}}".to_string());
//      scanner::run("class Bacon {
//   eat() {
//     print this.int + \" go crunch crunch crunch!\";
//     return x;
//   }
// }

// var y = Bacon();
// print y;

// print 123;

// var x = Bacon();
// print x;
// x.int = \"I\";
// x.howdythere = 10;
// print x.int;
// x.eat();".to_string());
    
    
    
    
    
    
    
    let args: Vec<String> = args().collect();
    println!("Detected {} main arguments", args.len());
    if args.len() < 2 {
        println!("No file supplied, starting in interactive mode...");
        interactive_mode(); 
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

    fn interactive_mode() {
        println!("Welcome to the Lox interpreter! Type 'exit' to quit.");
        
        let mut accumulated_input = String::new(); // Accumulator for inputs
        
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
    
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
    
            if input.trim() == "exit" {
                break;
            }
    
            accumulated_input.push_str(&input);
            
            scanner::run(accumulated_input.clone());
        }
    } 

    println!("Thus ends the program.");
}
