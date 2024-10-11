use crate::expr::*;
use crate::scanner::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr{
        expression: Box<Expr>
    },
    Print{
        expression: Box<Expr>
    },
    Var{
        name: String,
        line: usize,
        column: i64,
        initializer: Option<Expr>
    },
    Block{
        statements: Vec<Stmt>
    }
}