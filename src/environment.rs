use std::borrow::Borrow;
use std::collections::HashMap;
use std::env::var;
use crate::lox_callable::*;
use crate::expr::{Expr}; 
use crate::interpreter::{InterpreterError, Value};
use crate::scanner::{Token, TokenType};
use crate::lox_instance::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment{
    pub values: HashMap<String, (Option<Value>, VarLocation)>,
    pub enclosing: Option<Box<Environment>>,
    pub user_func: HashMap<String, UserDefined>,
    pub classes: HashMap<String, LoxClass>,
    pub instances: HashMap<String, LoxInstance>,
}

impl Default for Environment{
    fn default() -> Environment {
        Environment{
            values: HashMap::new(),
            enclosing: None,
            user_func: HashMap::new(),
            classes: HashMap::new(),
            instances: HashMap::new(),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct VarLocation{
    pub line: usize,
    pub col: i64
}

pub enum LookupResult{
    Ok(Value),
    UndefinedButDeclared{
        line: usize,
        col: i64
    },
    UndefinedAndUndeclared
}

impl Environment{
    pub fn new(enclosing: Environment) -> Self{
        Environment{
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
            user_func: HashMap::new(),
            classes: HashMap::new(),
            instances: HashMap::new()
        }
    }

    pub fn full(vals: HashMap<String, (Option<Value>, VarLocation)>, enclosing: Environment)-> Self{
        Environment{
            values: vals,
            enclosing: Some(Box::new(enclosing.clone())),
            user_func: enclosing.user_func,
            classes: enclosing.classes,
            instances: enclosing.instances
        }
    }

    pub fn set_enclosing(&mut self, enclosing: Option<Environment>) -> (){
        if enclosing.is_some(){
            self.enclosing = Some(Box::new(enclosing.unwrap()))
        }
        else{
            self.enclosing = None
        }
    }

    pub fn set_values(&mut self, values: HashMap<String, (Option<Value>, VarLocation)>) -> (){
        self.values = values;
    }

    pub fn return_enclosing(&self) -> Option<Box<Environment>>{
        return self.enclosing.clone();
    }

    pub fn return_values(&self) -> HashMap<String, (Option<Value>, VarLocation)>{
        return self.values.clone();
    }

    pub fn define(&mut self, name: String, line: usize, col: i64, possible_val: Option<Value>) -> (){
        self.values.insert(
            name,
            (
                possible_val,
                VarLocation{
                    line: line,
                    col: col
                }
            )
        );
        return ()
    }

    pub fn define_token(&mut self, token: Token, val: Value) -> (){
        self.values.insert(
            String::from_utf8(token.lexeme).unwrap(), 
            (
                Some(val),
                VarLocation{
                    line: token.line,
                    col: token.column
                }
            )
        );
        return ()
    }

    pub fn define_string(&mut self, name: String, val: Value){
        self.values.insert(
            name, 
            (
                Some(val),
                VarLocation{
                    line: 0,
                    col: 0
                }
            )
        );
        return ()
    }

    pub fn get(&self, expr: &Expr) -> Result<Value, InterpreterError>{
        if let Expr::Variable { name, line, col } = expr.clone(){
            match self.val_lookup(&expr) {
                LookupResult::Ok(val) => Ok(val),
                LookupResult::UndefinedButDeclared { line, col } => 
                Err(InterpreterError::new(format!("use of undefined variable '{}' at line: {}, column: {}",
                name, line, col),
                line,
                col,
                Value::Nil)),
                LookupResult::UndefinedAndUndeclared => {
                    match &self.enclosing {
                        Some(enclosing) => enclosing.get(expr),
                        None => Err(InterpreterError::new(
                                format!("use of undefined and undeclared variable '{}' at line: {}, column: {}",
                                name, line, col),
                                line,
                                col,
                                Value::Nil))
                    }
                }
            }
        }
        else if let Expr::This { keyword } = expr{
            match self.val_lookup(&expr){
                LookupResult::Ok(val) => Ok(val),
                _ => Err(InterpreterError::new(
                    format!("Fucked up 'this'"), 
                    0, 
                    0, 
                    Value::Nil
                ))
            }
        }
        else{
            panic!("Undefined hashmap error");
        }
    }

    pub fn assign(&mut self, name: String, line: usize, col: i64, val: &Value) -> Result<(), InterpreterError>{
        //println!("HERE");
        if self.values.contains_key(&name){
            //println!("NOPE");
            //self.values.remove(&name);
            self.define(name.clone(), line, col, Some(val.clone()));
            return Ok(());
        }
        match &mut self.enclosing {
            Some(enclosing) => return enclosing.assign(name.clone(), line, col, val),
            None => return Err(InterpreterError::new(
      format!("Attempting to assign undefined variable '{}' at line: {}, column: {}",
                    name, line, col), 
                    line, 
             col,
                    Value::Nil))
        }
    }

    pub fn assign_at(&mut self, name: String, line: usize, col: i64, val: &Value, distance: usize) -> Result<(), InterpreterError>{
        return self.assign(name, line, col, val);
        
    }

    pub fn val_lookup(&self, expr: &Expr) -> LookupResult{
        if let Expr::Variable { name, line , col } = expr{
            match self.values.get(name) {
                Some((maybe_val, var_location)) => match maybe_val{
                    Some(val) => LookupResult::Ok(val.clone()),
                    None => LookupResult::UndefinedButDeclared { 
                        line: var_location.line, 
                        col: var_location.col 
                    },
                },
                None => {
                    match self.user_func.get(name){
                        Some(func) => LookupResult::Ok(Value::UserDefined(func.clone())),
                        None => match self.classes.get(name){
                            Some(class) => LookupResult::Ok(Value::LoxClass(class.clone())),
                            None => LookupResult::UndefinedAndUndeclared
                        }
                        //None => LookupResult::UndefinedAndUndeclared
                    }
                }
            }
        }
        else if let Expr::This { keyword } = expr{
            let name = format!("this");
            match self.values.get(&name){
                Some((maybe_val, var_location)) => match maybe_val{
                    Some(val) => LookupResult::Ok(val.clone()),
                    None => LookupResult::UndefinedButDeclared { 
                        line: var_location.line, 
                        col: var_location.col 
                    },
                },
                None => LookupResult::UndefinedAndUndeclared
            }
        }
        else{
            panic!("Youre so cooked")
        }
    }

    // pub fn read_at(&self, distance: usize, name: String){
    //     self.ancestor(distance).borrow().
    // }
    pub fn get_at(&mut self, distance: usize, expr: Expr) -> Result<Value, InterpreterError>{
        return self.get(&expr);
    }
    pub fn ancestor(&self, distance: usize) -> Environment{
        let mut current: Environment = self.clone();
        for _ in 0..distance{
            //println!("Maybe {}", distance);
            current = *current.enclosing.unwrap();
            // match maybe_enclosing{
            //     Some(enc) => current = *enc,
            //     None => return current,
            // }
            //current = *maybe_enclosing.unwrap();
        }
        return current;
    }

    // pub fn assign_ancestor(&mut self, name: String, line: usize, col: i64, val: &Value, distance: usize) -> Result<(), InterpreterError>{
    //     let mut current: Environment = self.clone();
    //     for _ in 0..distance{
    //         println!("Maybe {}", distance);
    //         let maybe_enclosing = current.enclosing;
    //         match maybe_enclosing{
    //             Some(enc) => current = *enc,
    //             None => (),
    //         }
    //         //current = *maybe_enclosing.unwrap();
    //     }
    //     return Ok(current.assign(name, line, col, val))?;
    //}

}

