use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::fs;
use std::io::Error;
use std::str;
use text_io::read;

use crate::interpreter::Interpreter;
use crate::parser::{self};
use crate::resolver::Resolver;

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
    None
}


#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Vec<u8>,
    pub literal: Option<Literal>,
    pub line: usize,
    pub column: i64
}

impl Token {
    pub fn return_token_type(&self) -> TokenType{
        return self.token_type.clone();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScannerError{
    pub error: String,
    pub line: usize,
    pub column: i64,
}

pub struct Scanner{
    tokens: Vec<Token>,
    source: Vec<u8>,
    error: Option<ScannerError>,
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
    pub fn scan_tokens(&mut self, input_file: String) -> Vec<Token>{
        //Starts scanning process, continues until eof or error
        self.source = input_file.into_bytes();
        while !self.is_done_with_error(){
            self.start = self.current;
            self.scan_individual_tokens();
        }
        match self.error{
            Some(_) => {
                print_error(self.error.clone().unwrap());
                return Vec::new();
            },
            None => self.tokens.push(Token {
                token_type: TokenType::Eof,
                lexeme: Vec::new(),
                literal: None,
                line: self.line,
                column: self.column })
        }
        return self.tokens.clone();
    }
    
    fn scan_individual_tokens(&mut self) -> (){
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
                let is_equal: bool = self.matches('=');
                if is_equal {
                    self.add_token(TokenType::BangEqual, None);
                }
                else{
                    self.add_token(TokenType::Bang, None);
                }
            }
            '=' => {
                let is_equal: bool = self.matches('=');
                if is_equal {
                    self.add_token(TokenType::EqualEqual, None);
                }
                else{
                    self.add_token(TokenType::Equal, None);
                }
            }
            '<' => {
                let is_equal: bool = self.matches('=');
                if is_equal {
                    self.add_token(TokenType::LessEqual, None);
                }
                else{
                    self.add_token(TokenType::Less, None);
                }
            }
            '>' => {
                let is_equal: bool = self.matches('=');
                if is_equal {
                    self.add_token(TokenType::GreaterEqual, None);
                }
                else{
                    self.add_token(TokenType::Greater, None);
                }
            }
            '/' => {
                let is_equal: bool = self.matches('/');
                if is_equal {
                    //Implement comment recognition
                    self.discard_comment();
                }
                else if self.matches('*'){
                    self.discard_block_comment();
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
            '"' => self.string(),
            //following is for all other characters
            _ => {
                //implement number, letter, and error
                if scanned_char.is_ascii_digit() {
                    self.number();
                }
                else if scanned_char.is_ascii_alphabetic() {
                    self.identifier();
                }
                else {
                    self.error = Some(ScannerError{
                        error: format!("Scanner can not process {}", scanned_char),
                        line: self.line,
                        column: self.column,
                    });
                }                
            }



        }
    }

    fn advance_char(&mut self) -> char {
        self.current += 1;
        self.column += 1;
        return char::from(self.source[self.current - 1]);
    }

    fn is_digit(c: char) -> bool{
        return c.is_ascii_digit();
    }

    fn is_alpha(c: char) -> bool{
        return c.is_ascii_alphabetic();
    }

    fn is_alpha_num(c: char) -> bool{
        return Scanner::is_digit(c) || Scanner::is_alpha(c);
    }

    fn peek(&mut self) -> char{
        if self.is_finished(){
            return '\0';
        }
        return char::from(self.source[self.current]);
    }

    fn peek_next(&mut self) -> char{
        if self.current + 1 >= self.source.len(){
            return '\0';
        }
        return char::from(self.source[self.current + 1]);
    }

    fn add_token(&mut self, add_token_type: TokenType, add_literal: Option<Literal>) -> (){
        let text: Vec<u8> = self.source[self.start..self.current].to_vec();
        self.tokens.push(Token {
            token_type: add_token_type,
            lexeme: text,
            literal: add_literal,
            line: self.line,
            column: self.column })
    }
    
    fn string (&mut self) -> (){
        while self.peek() != '"' && !self.is_finished(){
            if self.peek() == '\n'{
                self.line += 1;
                self.column = 0;
            }
            self.advance_char();
        }
            if self.is_finished(){
                self.error = Some(ScannerError{
                    error: format!("Unterminated string"),
                    line: self.line,
                    column: self.column,
                });
                return;
            }
            self.advance_char();
            let value: String = String::from_utf8(self.source[self.start + 1..self.current - 1].to_vec()).unwrap();
            self.add_token(TokenType::String, Some(Literal::String(value)));
    }

    fn number (&mut self) -> (){
        while Scanner::is_digit(self.peek()){
            self.advance_char();
        }
        if self.peek() == '.' && Scanner::is_digit(self.peek_next()){
            self.advance_char();
        }
        while Scanner::is_digit(self.peek()){
            self.advance_char();
        }
        let value: f64 = String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap().parse().unwrap();
        self.add_token(TokenType::Number, Some(Literal::Number(value)));
    }

    fn identifier (&mut self) -> (){
        while Scanner::is_alpha_num(self.peek()){
            self.advance_char();
        }
        let value: String = String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap();
        let token_type: TokenType = match self.keywords.get(&value){
            Some(key_token_type) => *key_token_type,
            None => TokenType::Identifier,
        };
        
        match token_type{
            TokenType::Identifier => self.add_token(TokenType::Identifier, Some(Literal::Identifier(value))),
            _ => self.add_token(token_type, None),
        }
    }
    
    fn discard_comment(&mut self) {
        while !self.is_finished() && self.peek() != '\n' {
            self.advance_char();
        }
        if self.peek() == '\n' {
            self.advance_char(); 
            self.line += 1;
            self.column = 0; 
        }
    }    
    

    fn discard_block_comment(&mut self) -> (){
        while !self.is_finished(){
            if self.peek() == '\n'{
                self.line += 1;
                self.column = 0; 
            }
            let current_char: char = self.advance_char();
            if current_char == '*'{
                if self.peek() == '/'{
                    self.advance_char();
                    self.column += 1;
                    return;
                }
            }
        }
        self.error = Some(ScannerError{
            error: format!("Unclosed block comment"),
            line: self.line,
            column: self.column,
        })
    }
    
    fn matches(&mut self, expected_char: char) -> bool{
        if self.is_finished() {
            return false;
        }
        else if self.peek() != expected_char {
            return false;
        }
        self.current += 1;
        self.column += 1;
        return true;
    }

    fn is_finished(&self) -> bool{
        return self.current >= self.source.len();
    }

    fn is_done_with_error(&self) -> bool{
        return self.is_finished() || self.error.is_some();
    }
}

pub(crate) fn run_file(file_path: String) -> (){
    let file_contents: Result<String, Error> = fs::read_to_string(file_path.clone());
    let file_contents: String = match file_contents{
        Ok(file_string) => file_string,
        Err(error) => panic!("Problem opening the file: {error:?}")
    };
    run(file_contents);
}

pub(crate) fn run_prompt() ->(){
    loop{
        println!("> ");
        let line: String = read!("{}\n");
        if line.trim().is_empty(){
            break;
        }
        run(line)
    }
}

pub(crate) fn run(source: String) ->(){
    let mut scanner: Scanner = Scanner::default();
    let tokens: Vec<Token> = scanner.scan_tokens(source);
    // for tok in tokens.clone(){
    //     println!("{}", String::from_utf8(tok.lexeme).unwrap());
    // }
    let stmt = parser::parse_begin(tokens.clone());
    match stmt{
        Ok(stmt) => {
            let interpreter = Interpreter::new(stmt.clone());
            let mut resolver = Resolver::new(interpreter);
            let resolved = resolver.resolve(stmt.clone()); 
            match resolved.0{
                Ok(_good) => {
                    //println!("Made through resolving");
                    let mut inter = resolved.1.clone();
                    let interp = inter.interpret(stmt);
                    match interp{
                        Ok(()) => return (),
                        Err(err) => println!("{}\n", err.return_error())
                    }
                }
                Err(err) => {
                    for str in err{
                        println!("{}", str);
                    }
                }
            }
        },
        Err(err) => println!("{}\n", err.return_error())
    }

    //for token in tokens.clone(){
    //   println!("{}", String::from_utf8(token.lexeme.to_vec()).unwrap());
    //}
}

pub fn print_error(scanner_error: ScannerError) ->(){
    println!("Error Occurred: {} at line {}, column {}", 
    scanner_error.error, scanner_error.line, scanner_error.column);
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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn print_token(){
        let tokens = "/ and *".to_string();
        run(tokens);
    }

    #[test]
    fn scan_single_character_tokens() {
        let source = "( ) { } [ ] , . - + ; : * %".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        let expected_tokens = vec![
            TokenType::LeftParen, TokenType::RightParen,
            TokenType::LeftBrace, TokenType::RightBrace,
            TokenType::LeftBracket, TokenType::RightBracket,
            TokenType::Comma, TokenType::Dot,
            TokenType::Minus, TokenType::Plus,
            TokenType::Semicolon, TokenType::Colon,
            TokenType::Star, TokenType::Mod,
            TokenType::Eof
        ];

        let actual_tokens: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(expected_tokens, actual_tokens);
    }

    #[test]
    fn scan_two_character_tokens() {
        let source = "!= == >= <=".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        let expected_tokens = vec![
            TokenType::BangEqual, TokenType::EqualEqual,
            TokenType::GreaterEqual, TokenType::LessEqual,
            TokenType::Eof
        ];

        let actual_tokens: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(expected_tokens, actual_tokens);
    }

    #[test]
    fn scan_numbers() {
        let source = "123 45.67".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].literal, Some(Literal::Number(123.0)));
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].literal, Some(Literal::Number(45.67)));
    }

    #[test]
    fn scan_string_literal() {
        let source = "\"hello world\"".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].literal, Some(Literal::String("hello world".to_string())));
    }

    #[test]
    fn scan_keywords() {
        let source = "class var fun".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        let expected_tokens = vec![
            TokenType::Class, TokenType::Var, TokenType::Fun,
            TokenType::Eof
        ];

        let actual_tokens: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(expected_tokens, actual_tokens);
    }
    
    #[test]
    fn unterminated_string_error() {
        let source = "\"hello world".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        assert!(scanner.error.is_some());
        assert_eq!(scanner.error.unwrap().error, "Unterminated string");
    }

    #[test]
    fn skip_single_line_comment() {
        let source = "// this is a comment\n123".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].literal, Some(Literal::Number(123.0)));
    }

    #[test]
    fn skip_block_comment() {
        let source = "/* this is a block comment */123".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].literal, Some(Literal::Number(123.0)));
    }

    #[test]
    fn unterminated_block_comment_error() {
        let source = "/* unclosed comment".to_string();
        let mut scanner = Scanner::default();
        scanner.scan_tokens(source);

        assert!(scanner.error.is_some());
        assert_eq!(scanner.error.unwrap().error, "Unclosed block comment");
    }

    #[test]
    fn scan_mixed_tokens() {
        let source = "123 + variable * 4.56".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        let expected_tokens = vec![
            TokenType::Number, TokenType::Plus,
            TokenType::Identifier, TokenType::Star,
            TokenType::Number, TokenType::Eof
        ];

        let actual_tokens: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(expected_tokens, actual_tokens);
    }      

    #[test]
    fn scan_empty_source() {
        let source = "".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);

        assert_eq!(tokens.len(), 1); 
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn scan_with_whitespace() {
        let source = "( )   \t\n{ }\n\t[ ] \r\n ,\t. - + ; : * %\n".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        let expected_tokens = vec![
            TokenType::LeftParen, TokenType::RightParen,
            TokenType::LeftBrace, TokenType::RightBrace,
            TokenType::LeftBracket, TokenType::RightBracket,
            TokenType::Comma, TokenType::Dot,
            TokenType::Minus, TokenType::Plus,
            TokenType::Semicolon, TokenType::Colon,
            TokenType::Star, TokenType::Mod,
            TokenType::Eof
        ];
    
        let actual_tokens: Vec<TokenType> = tokens.into_iter().map(|t| t.token_type).collect();
        assert_eq!(expected_tokens, actual_tokens);
    }

    #[test]
    fn test_advance_char() {
        let source = "abc".to_string();
        let mut scanner = Scanner::default();
        scanner.source = source.into_bytes();
    
        let first_char = scanner.advance_char();
        assert_eq!(first_char, 'a');
        assert_eq!(scanner.current, 1);
        assert_eq!(scanner.column, 0);

        let second_char = scanner.advance_char();
        assert_eq!(second_char, 'b');
        assert_eq!(scanner.current, 2);
        assert_eq!(scanner.column, 1);

        let third_char = scanner.advance_char();
        assert_eq!(third_char, 'c');
        assert_eq!(scanner.current, 3);
        assert_eq!(scanner.column, 2);
    }
    
    #[test]
    fn test_add_token() {
        let mut scanner = Scanner::default();
        scanner.start = 0;
        scanner.current = 3;
        scanner.line = 1;
        scanner.column = 2;
        scanner.source = "abc".to_string().into_bytes();
    
        scanner.add_token(TokenType::Identifier, Some(Literal::Identifier("abc".to_string())));
    
        assert_eq!(scanner.tokens.len(), 1);
        let token = &scanner.tokens[0];
        assert_eq!(token.token_type, TokenType::Identifier);
        assert_eq!(token.lexeme, b"abc".to_vec());
        assert_eq!(token.literal, Some(Literal::Identifier("abc".to_string())));
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 2);
    }

    #[test]
    fn test_matches() {
        let source = "== !=".to_string();
        let mut scanner = Scanner::default();
        scanner.source = source.into_bytes();
    
        scanner.advance_char();
        let is_match = scanner.matches('=');
        assert!(is_match);
        assert_eq!(scanner.current, 2);
        assert_eq!(scanner.column, 1);
    
        scanner.advance_char(); 
        scanner.advance_char(); 
        let is_match = scanner.matches('=');
        assert!(is_match);
        assert_eq!(scanner.current, 5);
        assert_eq!(scanner.column, 4);
    }

    #[test]
    fn test_discard_comment() {
        let source = "// this is a comment\n123".to_string();
        let mut scanner = Scanner::default();
        scanner.source = source.into_bytes();

        scanner.advance_char();
        scanner.advance_char(); 
        scanner.discard_comment();

        // Verify the scanner is now at the next line, ready to process '123'
        assert_eq!(scanner.line, 2);
        assert_eq!(scanner.column, 0);
        let next_char = scanner.advance_char();
        assert_eq!(next_char, '1');
        assert_eq!(scanner.column, 1);
    }

    #[test]
    fn test_discard_block_comment() {
        let source = "/* this is a block comment */123".to_string();
        let mut scanner = Scanner::default();
        scanner.source = source.into_bytes();
    
        scanner.advance_char(); 
        scanner.advance_char(); 
        scanner.discard_block_comment();
    
        assert_eq!(scanner.line, 1);
        assert_eq!(scanner.column, 29);
        
        let next_char = scanner.advance_char();
        assert_eq!(next_char, '1');
        assert_eq!(scanner.column, 30);
    }
    
    #[test]
    fn test_invalid_character_error() {
        let source = "@".to_string(); 
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert!(scanner.error.is_some());
        assert_eq!(scanner.error.unwrap().error, "Scanner can not process @");
        
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_identifier_edge_cases() {
        let source = "var1 varWithNumbers123 var123AndSymbols".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert_eq!(tokens.len(), 4);
    
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].literal, Some(Literal::Identifier("var1".to_string())));
    
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].literal, Some(Literal::Identifier("varWithNumbers123".to_string())));
    
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].literal, Some(Literal::Identifier("var123AndSymbols".to_string())));
    
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_whitespace_and_mixed_whitespace() {
        let source = "   \t\nvar1  \tvar2\n   var3\t\n".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert_eq!(tokens.len(), 4); 
    
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].literal, Some(Literal::Identifier("var1".to_string())));
    
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].literal, Some(Literal::Identifier("var2".to_string())));
    
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].literal, Some(Literal::Identifier("var3".to_string())));
    
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }
    
    #[test]
    fn test_keyword_identifier_edge_cases() {
        let source = "class var variableClass className fun functionName".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert_eq!(tokens.len(), 7); 
    
        assert_eq!(tokens[0].token_type, TokenType::Class);
        assert_eq!(tokens[1].token_type, TokenType::Var);
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].literal, Some(Literal::Identifier("variableClass".to_string())));
    
        assert_eq!(tokens[3].token_type, TokenType::Identifier);
        assert_eq!(tokens[3].literal, Some(Literal::Identifier("className".to_string())));
    
        assert_eq!(tokens[4].token_type, TokenType::Fun);
        assert_eq!(tokens[5].token_type, TokenType::Identifier);
        assert_eq!(tokens[5].literal, Some(Literal::Identifier("functionName".to_string())));
    
        assert_eq!(tokens[6].token_type, TokenType::Eof);
    }

    #[test]
    fn test_string_escapes() {
        let source = "\"hello\nworld\" \"escape\\sequence\"".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert_eq!(tokens.len(), 3); 
    
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].literal, Some(Literal::String("hello\nworld".to_string())));
    
        assert_eq!(tokens[1].token_type, TokenType::String);
        assert_eq!(tokens[1].literal, Some(Literal::String("escape\\sequence".to_string())));
    
        assert_eq!(tokens[2].token_type, TokenType::Eof);
    }

    #[test]
    fn test_incomplete_tokens() {
        let source = "! = < >!".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert_eq!(tokens.len(), 6);
    
        assert_eq!(tokens[0].token_type, TokenType::Bang);
        assert_eq!(tokens[1].token_type, TokenType::Equal);
        assert_eq!(tokens[2].token_type, TokenType::Less);
        assert_eq!(tokens[3].token_type, TokenType::Greater);
        assert_eq!(tokens[4].token_type, TokenType::Bang);
        assert_eq!(tokens[5].token_type, TokenType::Eof);
    }

    #[test]
    fn test_number_edge_cases() {
        let source = "0 123 45.67 0.001 123.0".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert_eq!(tokens.len(), 6); 
    
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].literal, Some(Literal::Number(0.0)));
    
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].literal, Some(Literal::Number(123.0)));
    
        assert_eq!(tokens[2].token_type, TokenType::Number);
        assert_eq!(tokens[2].literal, Some(Literal::Number(45.67)));
    
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[3].literal, Some(Literal::Number(0.001)));
    
        assert_eq!(tokens[4].token_type, TokenType::Number);
        assert_eq!(tokens[4].literal, Some(Literal::Number(123.0)));
    
        assert_eq!(tokens[5].token_type, TokenType::Eof);
    }

    #[test]
    fn test_boundary_with_extremely_long_input() {
        let source = "a".repeat(1_000_000); 
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
    
        assert_eq!(tokens.len(), 2); 
    
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].literal, Some(Literal::Identifier("a".repeat(1_000_000))));
        assert_eq!(tokens[1].token_type, TokenType::Eof);
    }

    #[test]
    fn test_unterminated_block_comment_with_new_lines() {
        let source = "/* This is an unterminated block comment\n with multiple lines\n and more text".to_string();
        let mut scanner = Scanner::default();
        scanner.source = source.clone().into_bytes();
        scanner.scan_tokens(source);
    
        assert!(scanner.error.is_some());
        assert_eq!(scanner.error.unwrap().error, "Unclosed block comment");
    
        assert_eq!(scanner.tokens.len(), 0);
    }
    
}


