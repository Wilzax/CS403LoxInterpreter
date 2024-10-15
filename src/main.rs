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
    scanner::run("fun fib(n) {{\nif (n <= 1) return n;\nelse{{return fib(n-2) + fib(n-1);}}}}\nfor (var i = 0; i < 21; i = i + 1)\n{{print fib(i);}}".to_string());
    //scanner::run("fun fib(n) {{n = n + 1;\nprint n;\nif (n > 15) {{print \"hiii\";\nreturn n;}}\n return fib(n);}}\nvar x = fib(12);\n print x;".to_string());
    //scanner::run("fun sayHi(first, last) {{\n print \"Hi \" + first + \" \" + last + \"!\";}}\nsayHi(\"Dear\", \"Reader\");".to_string());
    //scanner::run("for (var i = 0; i < 10; i = i + 1)\n{{print i;}}".to_string());
    //scanner::run("var i = 0; \n while (i < 10){{\nprint i;\n i = i + 1;\nprint i;}}".to_string());
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
