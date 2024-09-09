use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{default, env};
use std::fs;
use std::process;
use std::io::Error;
use std::str;
use text_io::{read, scan};

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Colon,
    Slash,
    Star,
    Mod,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Lambda,
    Eof,
}


#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    None,
}

pub struct Scanner{
    tokens: Vec<Token>,
    source: Vec<u8>,
    error: Option<Error>,
    start: usize,
    current: usize,
    line: usize,
    column: i64,
    keywords: HashMap<String, TokenType>,
}

impl Default for Scanner{
    fn default() -> Scanner {
        Scanner{
            tokens: Vec::new(),
            source: Vec::new(),
            error: None,
            start: 0,
            current: 0,
            line: 1,
            column: -1,
            keywords: vec![
                ("and".to_string(), TokenType::And),
                ("class".to_string(), TokenType::Class),
                ("else".to_string(), TokenType::Else),
                ("false".to_string(), TokenType::False),
                ("for".to_string(), TokenType::For),
                ("fun".to_string(), TokenType::Fun),
                ("if".to_string(), TokenType::If),
                ("nil".to_string(), TokenType::Nil),
                ("or".to_string(), TokenType::Or),
                ("print".to_string(), TokenType::Print),
                ("return".to_string(), TokenType::Return),
                ("super".to_string(), TokenType::Super),
                ("this".to_string(), TokenType::This),
                ("true".to_string(), TokenType::True),
                ("var".to_string(), TokenType::Var),
                ("while".to_string(), TokenType::While),
                ("lambda".to_string(), TokenType::Lambda)
            ].into_iter().map(|(k, v)| (k, v)).collect()
        }
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
//    Vec<Token> tokens = 
// TODO: get scanner functionality
}

pub fn error(line: i32, message: String){
    report(line, "".to_string(), message);
}

pub(crate) fn report(line: i32, where_location: String, message: String){
    
}

// Returns an iterator to the read of the lines of the file. Output is wrapped in a Result to allow matching on errors
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    if let Ok(lines) = read_lines("./hosts.txt") {
        for line in lines.flatten() {
        //    println!("[]", line);
        }
    }
}
