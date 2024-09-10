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
    pub lexeme: Vec<u8>,
    pub literal: Option<Literal>,
    pub line: usize,
    pub column: i64
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

impl Scanner{
    fn scan_tokens(&mut self, input_file: String){
        //Starts scanning process, continues until eof or error
        self.source = input_file.into_bytes();
        while !self.is_finished(){
            self.start = self.current;
            self.scan_individual_tokens();
        }
        //Add error handling implementation
    }

    fn scan_individual_tokens(&mut self){
        //Main scanning function, all other functions are helpers
        let scanned_char: char = self.advance_char();

        match scanned_char {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            '[' => self.add_token(TokenType::LeftBracket, None),
            ']' => self.add_token(TokenType::RightBracket, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            ':' => self.add_token(TokenType::Colon, None),
            '*' => self.add_token(TokenType::Star, None),
            '%' => self.add_token(TokenType::Mod,None),
            '!' => {
                let is_equal = self.matches('=');
                if(is_equal){
                    self.add_token(TokenType::BangEqual, None);
                }
                else{
                    self.add_token(TokenType::Bang, None);
                }
            }
            '=' => {
                let is_equal = self.matches('=');
                if(is_equal){
                    self.add_token(TokenType::EqualEqual, None);
                }
                else{
                    self.add_token(TokenType::Equal, None);
                }
            }
            '<' => {
                let is_equal = self.matches('=');
                if(is_equal){
                    self.add_token(TokenType::LessEqual, None);
                }
                else{
                    self.add_token(TokenType::Less, None);
                }
            }
            '>' => {
                let is_equal = self.matches('=');
                if(is_equal){
                    self.add_token(TokenType::GreaterEqual, None);
                }
                else{
                    self.add_token(TokenType::Greater, None);
                }
            }
            '/' => {
                let is_equal = self.matches('/');
                if(is_equal){
                    //Implement comment recognition
                }
                else{
                    self.add_token(TokenType::Slash, None);
                }
            }
            '\n' => {
                self.line += 1;
                self.column = 0;
            }
            ' ' | '\r' | '\t' => {}
            //'"' => implement start of string
            //following is for all other characters
            _ => {
                //implement number, letter, and error
            }



        }
    }

    fn advance_char(&mut self) -> char {
        self.current += 1;
        self.column += 1;
        return char::from(self.source[self.current - 1]);
    }

    fn add_token(&mut self, add_token_type: TokenType, add_literal: Option<Literal>){
        let text = self.source[self.start..self.current].to_vec();
        self.tokens.push(Token { token_type: add_token_type, lexeme: text, literal: add_literal, line: self.line, column: self.column })
    }

    fn matches(&mut self, expected_char: char) -> bool{
        if(self.is_finished()){
            return false;
        }
        else if(char::from(self.source[self.current]) != expected_char){
            return false;
        }
        self.current += 1;
        self.column += 1;
        return true;
    }

    fn is_finished(&self) -> bool{
        return self.current >= self.source.len();
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
