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
        initializer: Option<Expr>
    }
}