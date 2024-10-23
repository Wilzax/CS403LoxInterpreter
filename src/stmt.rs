use crate::expr::*;
use crate::scanner::Token;
//fixing commit messages

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
    },
    If{
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>
    },
    While{
        condition: Expr,
        body: Box<Stmt>
    },
    Function{
        name: String,
        parameters: Vec<Token>,
        body: Box<Vec<Stmt>>
    },
    Return{
        keyword: Token,
        value: Option<Expr>
    },
    Class{
        name: String,
        superclass: Option<Expr>,
        methods: Box<Vec<Stmt>>
    }
}