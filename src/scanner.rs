//! The purpose of a scanner is to take an input stream and tokenize it.
//! The scanner: 
//!   defines the enum for list of possible tokens,
//!   opens an iostream,
//!     if interactive, use stdio       ##TODO: find actual package name
//!     if file supplied, use std::fs::File
//!   breaks the [stream? buffer?] into Strings,
//!   breaks Strings into tokens,
//!   matches tokens from stream to tokens in enum
//!     in the case of literals or identifiers, passes the value as well
//!   pushes matched tokens into vector
//!   calls the parser on the vector of token enums

mod scanner{
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;

    /// From https://craftinginterpreters.com/scanning.html
    /// All possible tokens in Lox language, reformatted from Java enum to Rust enum
    enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    
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
    Identifier(String),
    String(String),
    Number(f64),
    
    // Keywords.
    And,
    Delete,
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
    
    Eof,
    }
    // End citation for enum TokenType

    fn read_line_stdin() {
        
    }
    /// From https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html
    /// Opens a file from a given filename, and returns an iterable type for each line of the file
    fn read_file_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
    // End citation for fn read_file_lines
}
