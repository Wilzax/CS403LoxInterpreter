use std::env;
use std::fs;
use std::process;
use std::io::Error;
use std::str;
use text_io::{read, scan};

// fn main(){
//     let args: Vec<String> = env::args().collect();
//     if args.len() > 2{
//         println!("Usage: rlox [script]");
//         process::exit(0);
//     }
//     else if args.len() == 2{
//         run_file(args[0].clone());
//     }
//     else{
//         run_prompt();
//     }
// }
