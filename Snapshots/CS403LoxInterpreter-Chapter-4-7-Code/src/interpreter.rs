use std::borrow::Borrow;
use std::fmt::format;
use std::ptr::null;

use crate::scanner;
use crate::scanner::{Scanner, Token, TokenType};
use crate::expr::{self, BinaryOpType, UnaryOpType}; //Did not want to type scanner::Token 8000 times
use crate::expr::Expr;
use crate::parser;


#[derive(Debug, Clone, PartialEq)]
pub enum Value{
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Type{
    Number,
    String,
    Bool,
    Nil
}

pub fn value_type(value: Value) -> Type{
    match value{
        Value::Number(_) => Type::Number,
        Value::String(_) => Type::String,
        Value::Bool(_) => Type::Bool,
        Value::Nil => Type::Nil
    }
}

pub fn value_to_string(value: Value) -> String{
    match value{
        Value::Number(num) => format!("{}", num),
        Value::String(str) => format!("'{}'", str),
        Value::Bool(bool) => format!("{}", bool),
        Value::Nil => format!("nil")
    }
}

pub fn type_to_string(in_type: Type) -> String{
    match in_type{
        Type::Number => format!("Number"),
        Type::String => format!("String"),
        Type::Bool => format!("Bool"),
        Type::Nil => format!("Nil")
    }
}

pub struct Interpreter{
    expressions: Vec<Expr>
}

impl Default for Interpreter{
    fn default() -> Interpreter {
        Interpreter{
            expressions: Vec::new()
        }
    }
}

impl Interpreter{
    pub fn interpret(expression: Expr) -> Result<Value, InterpreterError>{
        let mut interp: Interpreter = Interpreter::default();
        interp.expressions.push(expression);
        let val: Result<Value, InterpreterError> = interp.evaluate(interp.expressions[0].clone());
        match val{
            Ok(value) => {
                println!("{}", value_to_string(value.clone()));
                return Ok(value)
            }
            Err(err) => {
                println!("{}", err.error_message.clone());
                return Err(err)
            }
        }
    }

    fn visit_literal_expr(&mut self, literal: expr::LiteralType) -> Value{
        match literal{
            expr::LiteralType::Number(num) => return Value::Number(num),
            expr::LiteralType::String(str) => return Value::String(str),
            expr::LiteralType::True => return Value::Bool(true),
            expr::LiteralType::False => return Value::Bool(false),
            expr::LiteralType::Nil => return Value::Nil
        }
    }

    fn visit_unary_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Unary { operator , right , line , col } = expr{
            let right_val: Value = self.evaluate(*right)?;
            match (operator, right_val.clone()){
                (UnaryOpType::Minus, Value::Number(num)) => return Ok(Value::Number(-num)),
                (UnaryOpType::Bang, _) => return Ok(Value::Bool(!Interpreter::is_truthy(right_val.clone()))),
                (_, _) => return Err(InterpreterError { 
                    error_message: format!("Incorrect use of unary operator {:?} on object of type {:?} at line: {}, column: {}", 
                    operator, type_to_string(value_type(right_val)), line, col), 
                    line: line, 
                    column: col
                })
            }
        }
        else{
            panic!("Unreachable Unary Error");
        }
    }

    fn visit_binary_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Binary { left, operator , right, line , col } = expr{
            let left_val: Value = self.evaluate(*left)?;
            let right_val: Value = self.evaluate(*right)?;
            match(operator, left_val.clone(), right_val.clone()){
                (BinaryOpType::Plus, Value::Number(num1), Value::Number(num2)) => {
                    return Ok(Value::Number(num1 + num2))
                },
                (BinaryOpType::Plus, Value::String(str1), Value::String(str2)) => {
                    return Ok(Value::String(format!("{}{}", str1, str2)))
                },
                (BinaryOpType::Minus, Value::Number(num1), Value::Number(num2)) => {
                    return Ok(Value::Number(num1 - num2))
                },
                (BinaryOpType::Slash, Value::Number(num1), Value::Number(num2)) => {
                    if num2 == 0.0 {
                        return Err(InterpreterError { 
                            error_message: format!("Divide by zero error at line: {}, column: {}", line, col), 
                            line: line, 
                            column: col 
                        })
                    }
                    else {
                        return Ok(Value::Number(num1 / num2))
                    }
                },
                (BinaryOpType::Star, Value::Number(num1), Value::Number(num2)) => {
                    return Ok(Value::Number(num1 * num2))
                },
                (BinaryOpType::Greater, Value::Number(num1), Value::Number(num2)) => {
                    return Ok(Value::Bool(num1 > num2))
                },
                (BinaryOpType::GreaterEqual, Value::Number(num1), Value::Number(num2)) => {
                    return Ok(Value::Bool(num1 >= num2))
                },
                (BinaryOpType::Less, Value::Number(num1), Value::Number(num2)) => {
                    return Ok(Value::Bool(num1 < num2))
                },
                (BinaryOpType::LessEqual, Value::Number(num1), Value::Number(num2)) => {
                    return Ok(Value::Bool(num1 <= num2))
                },
                (BinaryOpType::EqualEqual, _, _) => {
                    return Ok(Value::Bool(Interpreter::is_equal(left_val.clone(), right_val.clone())))
                },
                (BinaryOpType::NotEqual, _, _) => {
                    return Ok(Value::Bool(!Interpreter::is_equal(left_val.clone(), right_val.clone())))
                },
                (_, _, _) => {
                    return Err(InterpreterError { 
                        error_message: format!("Incorrect use of unary operator {:?} on objects of type {:?} and {:?} at line: {}, column: {}", 
                        operator, type_to_string(value_type(left_val)), type_to_string(value_type(right_val)), line, col), 
                        line: line, 
                        column: col 
                    })
                }
            }
        }
        else{
            panic!("Unreachable Binary Error");
        }
    }

    fn is_truthy(val: Value) -> bool{
        match val{
            Value::Nil => false,
            Value::Bool(bool_val) => bool_val,
            _ => true
        }
    }

    fn is_equal(left_value: Value, right_value: Value) -> bool{
        match(left_value, right_value){
            (Value::Nil, Value::Nil) => true,
            (Value::Number(num1), Value::Number(num2)) => return num1.eq(&num2),
            (Value::String(str1), Value::String(str2)) => return str1 == str2,
            (Value::Bool(bool1), Value::Bool(bool2)) => bool1 == bool2,
            (_, _) => false
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Grouping { expression } = expr{
            return self.evaluate(*expression);
        }
        else if let Expr::Binary { left: _ , operator: _ , right: _ , line: _ , col: _ } = expr{
            return self.visit_binary_expr(expr);
        }
        else if let Expr::Unary { operator: _ , right: _ , line: _ , col: _ } = expr{
            return self.visit_unary_expr(expr);
        }
        else if let Expr::Literal { value } = expr{
            return Ok(self.visit_literal_expr(value));
        }
        else{
            return Err(InterpreterError { 
                error_message: format!("We dont have that expression type yet bud"), 
                line: 0, 
                column: 0 
            })    
        }
    }
}

pub struct InterpreterError{
    error_message: String,
    line: usize,
    column: i64
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::scanner::*;
    use crate::parser::*;
    use crate::expr::*;

    #[test]
    fn simple_addition(){
        let expr: Expr = Expr::Grouping { 
            expression: Box::new(Expr::Binary { 
                left: Box::new(Expr::Literal { value: expr::LiteralType::Number(3.0)}), 
                operator: BinaryOpType::Plus, 
                right: Box::new(Expr::Literal { value: expr::LiteralType::Number(4.0)}), 
                line: 0, 
                col: 0 
            }) 
        };
        let val: Result<Value, InterpreterError> = Interpreter::interpret(expr);
        match val{
            Ok(val) => assert_eq!("7", value_to_string(val), "Error interpreting 3 + 4"),
            Err(err) => panic!("Error when interpreting")
        }
    }
}
