use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt;
use crate::environment;
use crate::interpreter;
use crate::interpreter::*;
use crate::lox_instance::*;
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

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass{
    pub name: String,
    pub methods: HashMap<String, UserDefined>
}

impl LoxClass{
    pub fn to_string(&self) -> String{
        return self.name.clone();
    }

    pub fn arity(&self) -> usize{
        return 0;
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: &Vec<Value>) -> Result<Value, InterpreterError>{
        let instance = interpreter.create_instance(self.clone());
        return Ok(instance);
    }

    pub fn find_method(&self, name: String) -> Result<UserDefined, ()>{
        match self.methods.get(&name){
            Some(method) => return Ok(method.clone()),
            None => return Err(())
        }
    }
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
            let mut environment: Environment = Environment::default();
            //let mut environment: Environment = Environment::new(self.closure.clone());
            for entries in self.closure.values.clone(){
                environment.define(entries.0, 0, 0, entries.1.0);
            }
            let mut i = 0;
            while i < parameters.len() {
                let argument = args.get(i).unwrap().clone();
                environment.define_token(parameters.get(i).unwrap().clone(), argument);
                i += 1;
            }
            let mut block_env = Environment::full(environment.return_values(), interpreter.environment.clone());
            //let mut block_env = Environment::full(environment.return_values(), self.closure.clone());
            block_env.user_func = interpreter.environment.user_func.clone();
            let current_interp = interpreter.environment.clone();
            let res = interpreter.execute_block(*body.clone(), Some(block_env));
            interpreter.environment = current_interp;
            match res{
                Ok(nothing) => return Ok(Value::Nil),
                Err(err) => return Ok(err.value)
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

    pub fn bind(&mut self, instance: &Rc<LoxInstance>) -> UserDefined{
        let mut environment = Environment::new(self.closure.clone());
        environment.define(format!("this"), 0, 0, Some(Value::LoxInstance(instance.clone())));
        return UserDefined {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            declaration: self.declaration.clone(),
            closure: environment
        }
    }
}