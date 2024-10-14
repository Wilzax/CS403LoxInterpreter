use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use crate::environment;
use crate::interpreter;
use crate::interpreter::*;
use crate::parser::*;
use crate::environment::*;
use crate::expr::*;
use crate::scanner::Token;
use crate::stmt::*;

#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunction{
    pub name: String,
    pub arity: usize,
    pub callable: fn(&mut Interpreter, &[Value]) -> Result<Value, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserDefined{
    pub name: String,
    pub parameters: Vec<Token>,
    pub body: Vec<Stmt>,
    pub declaration: Stmt,
    pub closure: Environment
}

impl NativeFunction{
    pub fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, String>{
        (self.callable)(interpreter, args)
    }
    pub fn arity(&self) -> usize{
        return self.arity
    }
}

impl UserDefined{
    pub fn call(&self, interpreter: &mut Interpreter, args: &Vec<Value>) -> Result<Value, InterpreterError>{
        if let Stmt::Function { name: _ , parameters , body } = &self.declaration{
            //println!("Inside func");
            let mut environment: Environment = Environment::new(self.closure.clone());
            let mut i = 0;
            while i < parameters.len() {
                let argument = args.get(i).unwrap().clone();
                environment.define_token(parameters.get(i).unwrap().clone(), argument);
                i += 1;
            }
            let block_env = Environment::new(environment);
            interpreter.execute_block(*body.clone(), Some(block_env))?;
            match &interpreter.return_value{
                Some(val) =>{
                    return Ok(val.clone())
                }
                None => return Ok(Value::Nil)
            }
        }
        else{
            panic!("Unreachable Function Error");
        }
    }
    pub fn arity(&self) -> usize{
        return self.parameters.len()
    }

    pub fn to_string(&self) -> String{
        if let Stmt::Function { name, parameters: _ , body: _ } = &self.declaration{
            return format!("<fn {} >", name);
        }
        else{
            panic!("Unreachable error");
        }
    }
}