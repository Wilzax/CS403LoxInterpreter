use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::environment::*;
use crate::lox_callable::*;
use crate::scanner;
use crate::scanner::{Scanner, Token, TokenType};
use crate::expr::{self, BinaryOpType, UnaryOpType}; //Did not want to type scanner::Token 8000 times
use crate::expr::Expr;
use crate::stmt::*;
use crate::parser;


#[derive(Debug, Clone, PartialEq)]
pub enum Value{
    Number(f64),
    String(String),
    Bool(bool),
    UserDefined(UserDefined),
    NativeFunction(NativeFunction),
    Nil,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Type{
    Number,
    String,
    Bool,
    UserDefined,
    NativeFunction,
    Nil
}
impl Value{
    pub fn value_type(value: Value) -> Type{
        match value{
            Value::Number(_) => Type::Number,
            Value::String(_) => Type::String,
            Value::Bool(_) => Type::Bool,
            Value::UserDefined(_) => Type::UserDefined,
            Value::NativeFunction(_) => Type::NativeFunction,
            Value::Nil => Type::Nil
            
        }
    }

    pub fn value_to_string(value: Value) -> String{
        match value{
            Value::Number(num) => format!("{}", num),
            Value::String(str) => format!("{}", str),
            Value::Bool(bool) => format!("{}", bool),
            Value::NativeFunction(nat) => format!("{}", nat.name),
            Value::UserDefined(user) => format!("{}", user.name),
            Value::Nil => format!("nil")
        }
    }
}

impl Type{
    pub fn type_to_string(in_type: Type) -> String{
        match in_type{
            Type::Number => format!("Number"),
            Type::String => format!("String"),
            Type::Bool => format!("Bool"),
            Type::NativeFunction => format!("Native Function"),
            Type::UserDefined => format!("User Defined Function"),
            Type::Nil => format!("Nil")
        }
    }
}

pub struct Interpreter{
    pub statements: Vec<Stmt>,
    pub globals: Environment,
    pub environment: Environment,
    pub return_value: Option<Value>
}

impl Default for Interpreter{
    fn default() -> Interpreter {
        // Interpreter{
        //     statements: Vec::new(),
        //     globals: Environment::default(),
        //     environment: Environment::default()
        // }
        let mut globals_env: HashMap<String, (Option<Value>, VarLocation)> = HashMap::new();
        globals_env.insert(String::from("clock"),
        (
            Some(Value::NativeFunction(NativeFunction{ 
                name: format!("clock"), 
                arity: 0, 
                callable: |_, _|{
                    let start_time = SystemTime::now();
                    let since_epoch = start_time.duration_since(UNIX_EPOCH).unwrap();
                    Ok(Value::Number(since_epoch.as_millis() as f64))
                }, 
            })),
            VarLocation{
                line: 0,
                col: 0
            }
        ));

        let mut globals = Environment::default();
        globals.set_values(globals_env);
        globals.set_enclosing(None);

        Interpreter { 
            statements: Vec::new(), 
            globals: globals.clone(), 
            environment: globals,
            return_value: None
        }   
    }
}

impl Interpreter{
    pub fn new(statements: Vec<Stmt>) -> Self{
        Interpreter{
            statements: statements,
            globals: Environment::default(),
            environment: Environment::default(),
            return_value: None
        }
    }

    pub fn interpret(statements: Vec<Stmt>) -> Result<(), InterpreterError>{
        let mut interp: Interpreter = Interpreter::new(statements.clone());
        //println!("We interpreting");
        for stmt in statements{
            let execution: Result<(), InterpreterError> = interp.execute(stmt);
            match execution{
                Ok(stmt) => (),
                Err(err) => return Err(err)
            }
        }
        return Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Expr { expression } = stmt{
            self.evaluate(*expression)?;
            return Ok(());
        }
        else{
            panic!("Unreacheable Expression Error");
        }
    }

    fn visit_function_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Function { name, parameters, body } = stmt.clone(){
            if self.globals.return_values().contains_key(&name){
                return Err(InterpreterError { 
                    error_message: format!("Function already defined"), 
                    line: 0, 
                    column: 0 
                })
            }
            else{
                //println!("Func def");
                let function = Value::UserDefined(UserDefined{
                    name: name.clone(),
                    parameters: parameters,
                    body: *body,
                    declaration: stmt.clone(),
                    closure: self.environment.clone()
                });
                self.environment.define(name.clone(), 0, 0, Some(function));
                //println!("Func def finish");
                return Ok(())
            }
        }
        else{
            panic!("Unreachable Function Error");
        }
    }

    fn visit_if_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::If { condition, then_branch, else_branch } = stmt{
            if Interpreter::is_truthy(self.evaluate(*condition)?){
                self.execute(*then_branch)?;
            }
            else{
                if else_branch.is_some(){
                    self.execute(*else_branch.unwrap())?;
                }
            }
            return Ok(());
        }
        else{
            panic!("Unreachable If Expression Error");
        }
    }

    fn visit_while_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::While { condition, body } = stmt{
            while Interpreter::is_truthy(self.evaluate(*condition.clone())?){
                self.execute(*body.clone())?;
            }
            return Ok(());
        }
        else{
            panic!("Unreachable While Error");
        }
    }

    fn visit_print_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Print { expression } = stmt{
            let value = self.evaluate(*expression)?;
            println!("{}", Value::value_to_string(value));
            return Ok(());
        }
        else{
            panic!("Unreachable Print Error");
        }
    }

    fn visit_var_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Var { name, line, column, initializer } = stmt{
            let mut val: Value = Value::Nil;
            let mut opt: Option<Value> = None;
            if initializer.is_some(){
                val = self.evaluate(initializer.unwrap())?;
            }
            if val == Value::Nil{
                self.environment.define(name, line, column, opt);
            }
            else{
                opt.insert(val);
                self.environment.define(name, line, column, opt);
            }
            return Ok(())
        }
        else{
            panic!("Unreachable Var Error")
        }
    }

    fn visit_return_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Return { keyword, value } = stmt{
            let retval = self.evaluate(value.unwrap());
            match retval{
                Ok(val) => Ok(self.return_value = Some(val)),
                Err(none) => Ok(self.return_value = None) 
            }
        }
        else{
            panic!("Unreachable return error");
        }
    }

    fn visit_assign_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Assign { name, line, column, value } = expr{
            let val: Value = self.evaluate(*value)?;
            let expression = self.environment.assign(name, line, column, &val.clone());
            match expression{
                Ok(ex) => return Ok(val),
                Err(err) => Err(err)
            }
        }
        else{
            panic!("Unreachable assignment error");
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
                    operator, Type::type_to_string(Value::value_type(right_val)), line, col), 
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
                        operator, Type::type_to_string(Value::value_type(left_val)), Type::type_to_string(Value::value_type(right_val)), line, col), 
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

    fn visit_call_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Call { callee , paren , arguments } = expr{
            println!("Callin");
            let callee_value: Value = self.evaluate(*callee)?;

            let argument_values: Result<Vec<Value>, InterpreterError> = arguments
            .into_iter()
            .map(|expr| self.evaluate(expr))
            .collect();

            let args = argument_values?;

            match callee_value{
                Value::NativeFunction(function) =>{
                    if args.len() != function.arity() {
                        return Err(InterpreterError { 
                            error_message: format!("Expected {} arguments but got {}",
                            function.arity(), args.len()), 
                            line: 0, 
                            column: 0 
                        })
                    }
                    else{

                        let func =  function.call(self, &args);
                        match func{
                            Ok(func) => return Ok(func),
                            Err(err) => return Err(InterpreterError { 
                                error_message: err, 
                                line: 0, 
                                column: 0  
                            })
                        }
                    }
                }
                Value::UserDefined(function) =>{
                    if args.len() != function.arity() {
                        return Err(InterpreterError { 
                            error_message: format!("Expected {} arguments but got {}",
                            function.arity(), args.len()), 
                            line: 0, 
                            column: 0 
                        })
                    }
                    else{
                        println!("User func call");
                        let func =  function.call(self, &args);
                        match func{
                            Ok(func) => return Ok(func),
                            Err(err) => return Err(err)
                        }
                    }
                }
                _ => {
                    return Err(InterpreterError { 
                        error_message: format!("Can only call functions and classes"), 
                        line: 0, 
                        column: 0 
                    });
                }
            }

        }
        else{
            panic!("Unreachable Call Error");
        }
    }

    fn visit_variable_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Variable { name , line: _ , col: _ } = expr.clone(){
            return self.environment.get(&expr);
        }
        panic!("Unreachable Variable Error");
    }

    fn visit_logical_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Logical { left, operator, right } = expr{
            let left_val = self.evaluate(*left)?;
            match operator.return_token_type(){
                TokenType::Or => {
                    match Interpreter::is_truthy(left_val.clone()){
                        true => return Ok(left_val),
                        false => (),
                    }
                }
                _ => {
                    match Interpreter::is_truthy(left_val.clone()){
                        false => return Ok(left_val),
                        true => (),
                    }
                }
            }
            return Ok(self.evaluate(*right)?);
        }
        else{
            panic!("Unreachable Logical Error");
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

    pub fn evaluate(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
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
        else if let Expr::Assign { name: _ , line: _, column: _, value: _ } = expr{
            return Ok(self.visit_assign_expr(expr))?;
        }
        else if let Expr::Variable { name: _ , line: _ , col: _ } = expr{
            return Ok(self.visit_variable_expr(expr))?;
        }
        else if let Expr::Logical { left: _ , operator: _ , right: _ } = expr{
            return Ok(self.visit_logical_expr(expr))?;
        }
        else if let Expr::Call { callee: _ , paren: _ , arguments: _ } = expr{
            return Ok(self.visit_call_expr(expr))?;
        }
        else{
            return Err(InterpreterError { 
                error_message: format!("We dont have that expression type yet bud"), 
                line: 0, 
                column: 0 
            })    
        }
    }

    pub fn execute(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Expr { expression: _ } = stmt{
            //println!("Aw shucks");
            return Ok(self.visit_expression_stmt(stmt)?);
        }
        else if let Stmt::Print { expression: _ } = stmt{
            //println!("Yeppers");
            return Ok(self.visit_print_stmt(stmt))?;
        }
        else if let Stmt::Var { name: _, line: _ , column: _ , initializer: _ } = stmt{
            return Ok(self.visit_var_stmt(stmt))?;
        }
        else if let Stmt::Block { statements: _ } = stmt{
            return Ok(self.visit_block_stmt(stmt))?;
        }
        else if let Stmt::If { condition: _ , then_branch: _ , else_branch: _ } = stmt{
            return Ok(self.visit_if_stmt(stmt))?;
        }
        else if let Stmt::While { condition: _ , body: _ } = stmt{
            return Ok(self.visit_while_stmt(stmt))?;
        }
        else if let Stmt::Function { name: _ , parameters: _ , body: _ } = stmt{
            return Ok(self.visit_function_stmt(stmt))?;
        }
        else if let Stmt::Return { keyword: _ , value: _ } = stmt{
            return Ok(self.visit_return_stmt(stmt))?;
        }
        else{
            //println!("Kys");
            return Err(InterpreterError { 
                error_message: format!("We dont have that statement type yet bud"), 
                line: 0, 
                column: 0 
            })
        }
    }

    pub fn execute_block(&mut self, statements: Vec<Stmt>, env: Option<Environment>) -> Result<(), InterpreterError>{
        match env{
            Some(enviro) => self.environment = enviro,
            None => self.environment = Environment::new(self.environment.clone()),
        }
        for stmt in statements{
            let execute: Result<(), InterpreterError> = self.execute(stmt);
            match execute{
                Ok(void) => (),
                Err(err) => return Err(err)
            }
        }
        //println!("Problem after execute");
        if let Some(enclosing) = self.environment.return_enclosing(){
            self.environment = *enclosing;
        }
        else{
            panic!("Unreachable hell");
        }
        //println!("Post enclosing");
        return Ok(());
    }

    fn visit_block_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Block { statements } = stmt{
            let execute = self.execute_block(statements, None);
            match execute{
                Ok(exec) => return Ok(()),
                Err(err) => return Err(err)
            }
        }
        else{
            panic!("Unreachable block error");
        }
    }
}

pub struct InterpreterError{
    error_message: String,
    line: usize,
    column: i64
}

impl InterpreterError{
    pub fn new(error_message: String, line: usize, column: i64) -> Self{
        InterpreterError {
            error_message: error_message,
            line: line,
            column: column
        }
    }

    pub fn return_error(&self) -> String{
        return self.error_message.clone();
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::scanner::*;
    use crate::parser::*;
    use crate::expr::*;

}
