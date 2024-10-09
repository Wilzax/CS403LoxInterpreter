use crate::scanner::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: BinaryOpType,
        right: Box<Expr>,
        line: usize,
        col: i64,
    },
    Grouping {
        expression: Box<Expr>
    },
    Literal {
        value: LiteralType
    },
    Unary {
        operator: UnaryOpType,
        right: Box<Expr>,
        line: usize,
        col: i64,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryOpType{
    Minus,
    Bang,
    Error
}

impl UnaryOpType{
    pub fn unary_match(token_type: TokenType) -> UnaryOpType{
        match token_type{
            TokenType:: Minus => UnaryOpType::Minus,
            TokenType::Bang => UnaryOpType::Bang,
            _ => UnaryOpType::Error,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOpType{
    Less,
    LessEqual,
    EqualEqual,
    NotEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Slash,
    Star,
    Error
}
impl BinaryOpType{
    pub fn binary_match(token_type: TokenType) -> BinaryOpType{
        match token_type{
            TokenType::Less => BinaryOpType::Less,
            TokenType::LessEqual => BinaryOpType::LessEqual,
            TokenType::EqualEqual => BinaryOpType::EqualEqual,
            TokenType::BangEqual => BinaryOpType::NotEqual,
            TokenType::Greater => BinaryOpType::Greater,
            TokenType::GreaterEqual => BinaryOpType::GreaterEqual,
            TokenType::Plus => BinaryOpType::Plus,
            TokenType::Minus => BinaryOpType::Minus,
            TokenType::Slash => BinaryOpType::Slash,
            TokenType::Star => BinaryOpType::Star,
            _ => BinaryOpType::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType{
    Number(f64),
    String(String),
    True,
    False,
    Nil
}

