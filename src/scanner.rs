use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::env;
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

pub struct Lexer<'source> {
    source: &'source str,
    start: usize,
    current: usize,
    line: usize,
}

pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
}

impl<'source>Iterator for Lexer<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_at_end() {
            self.start = self.current;
            let character = self.advance();
            let token = if let Some(character) = character {
                match character {
                    '(' => {
                        Some(self.yield_token(LeftParen))
                    }
                    ')' => {
                        Some(self.yield_token(RightParen))
                    }
                    '{' => {
                        Some(self.yield_token(LeftBrace))
                    }
                    '}' => {
                        Some(self.yield_token(RightBrace))
                    }
                    ',' => {
                        Some(self.yield_token(Comma))
                    }
                    '.' => {
                        Some(self.yield_token(Dot))
                    }
                    '-' => {
                        Some(self.yield_token(Minus))
                    }
                    '+' => {
                        Some(self.yield_token(Plus))
                    }
                    ';' => {
                        Some(self.yield_token(Semicolon))
                    }
                    ':' => {
                        Some(self.yield_token(Colon))
                    }
                    '%' => {
                        Some(self.yield_token(Mod))
                    }
                    '*' => {
                        Some(self.yield_token(Star))
                    }
                    '!' => {
                        let token = if self.char_matches('=') {
                            BangEqual
                        } else {
                            Bang
                        };
                        Some(self.yield_token(token))
                    }
                    '=' => {
                        let token = if self.char_matches('='){
                            EqualEqual
                        } else {
                            Equal
                        };
                        Some(self.yield_token(token))
                    }
                    '<' => {
                        let token = if self.char_matches('=') {
                            LessEqual
                        } else {
                            Less
                        };
                        Self(self.yield_token(token))
                    }
                    '>' => {
                        let token = if self.char_matches('=') {
                            GreaterEqual
                        } else {
                            Greater
                        };
                        Some(self.yield_token(token))
                    }
                    '/' => {
                        if self.char_matches('/') {
                            let comment_value = self.take_while(|ch| ch != '\n');
                            match comment_value{
                                Some((comment, _)) => {
                                    Some(Token::new(TokenType::Comment, comment.to_string(), Literal::String(comment.to_string()), self.line))
                                }
                                None => {
                                        Some(Token::new(TokenType::Error, "Error fetchingcomment tokens".into() Literal::None, self.line))
                                }
                            }
                        } else if self.char_matches('*') {
                            let mut found_closing_pair = false;
                            let mut comment_buffer = String::new();
                            while let (Some(ch), Some(next_ch)) = (self.peek(),self.peek_next()) {
                                if ch == '*' && next_ch == '/' {
                                    found_closing_pair = true;
                                    break;
                                }
                                else {
                                    let char = self.advance();
                                    if let Some(ch) = char {
                                        comment_buffer.push(ch);
                                    }
                                }
                            }
                            if !found_closing_pair {
                                panic!("Found an unclosed comment");
                            }
                            self.advance();
                            self.advance();
                            Some(Token::new(TokenType::Comment,comment_buffer.clone(), Literal::String(comment_buffer), self.line))
                        } else {
                            Some(self.yield_token(Slash))
                        }
                    }
                    '\n' => {
                        self.line += 1;
                        Some(self.yield_token(Newline))
                    }
                    '"' => {
                        let sox_string = self.yield_string();
                        self.token_from_result(sox_string)
                    }
                    'A'..='Z' | 'a'..='z' | '_' => {
                        let ident_val = self.yield_identifier();
                        self.token_from_result(ident_val)
                    }
                    '0'..='9' => {
                        let number_val = self.yield_number();
                        self.token_from_result(numer_val)
                    }
                    ' ' => {
                        Some(self.yield_token(TokenType::Whitespace))
                    }
                    _ => {
                        debug!("Token -{character} - not allowed set of valid tokens");
                        Some(Token::new(TokenType::Error, "Token -{character} - not in allowed set of valid tokens".into(), Literal::None, self.line))
                    }
                }
            } else {
                Some(Token::new(TokenType::Error, "No more characters to lex".into(), Literal::None, self.line))
            };
            token
        } else {
            None
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
    Vec<Token> tokens = 
}

pub fn error(line: i32, message: String){
    report(line, "".to_string(), message);
}

pub(crate) fn report(line: i32, whereLocation: String, message: String){
    
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
            println!("[]", line);
        }
    }
}
