use crate::scanner::Token;

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: BinaryOpType,
        right: Box<Expr>,
        line: usize,
        col: i64,
    },
    Grouping {
        expression: Box<Expr>,
        line: usize,
        col: i64,
    },
    Literal {
        value: LiteralType,
        line: usize,
        col: i64,
    },
    Unary {
        operator: UnaryOpType,
        right: Box<Expr>,
        line: usize,
        col: i64,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum UnaryOpType{
    Minus,
    Bang,
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOpType{
    Less,
    LessEqual,
    EqualEqual,
    NotEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash
}

#[derive(Debug, Clone)]
pub enum LiteralType{
    Number(f64),
    String(String),
    True,
    False,
    Nil
}

