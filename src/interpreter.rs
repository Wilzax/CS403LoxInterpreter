use std::borrow::Borrow;

use crate::scanner;
use crate::scanner::{Scanner, Token, TokenType};
use crate::expr; //Did not want to type scanner::Token 8000 times
use crate::expr::Expr;
use crate::parser;

pub enum Value{
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

pub struct Interpreter{

}

impl Interpreter{
    fn visit_literal_expr(literal: expr::LiteralType) -> Value{
        match literal{
            expr::LiteralType::Number(num) => return Value::Number(num),
            expr::LiteralType::String(str) => return Value::String(str),
            expr::LiteralType::True => return Value::Bool(true),
            expr::LiteralType::False => return Value::Bool(false),
            expr::LiteralType::Nil => return Value::Nil
        }
    }

    fn visit_unary_expr(expr: Expr){

    }

    fn evaluate(expr: Expr) -> R{
        if let Expr::Grouping { expression: _ } = expr{

        }
        else if let Expr::Binary { left: _ , operator: _ , right: _ , line: _ , col: _ } = expr{

        }
        else if let Expr::Unary { operator: _ , right: _ , line: _ , col: _ } = expr{

        }
        else if let Expr::Literal { value } = expr{
            return Interpreter::visit_literal_expr(value);
        }
    }
}