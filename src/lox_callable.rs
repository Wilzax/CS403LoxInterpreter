use std::array::IntoIter;
use std::borrow::BorrowMut;
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
use crate::scanner::*;
use crate::stmt::*;
//fixing commit messages

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
    pub closure: Environment,
    pub is_init: bool
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass{
    pub name: String,
    pub superclass: Box<Option<LoxClass>>,
    pub methods: HashMap<String, UserDefined>
}

impl LoxClass{
    pub fn to_string(&self) -> String{
        return self.name.clone();
    }

    pub fn arity(&self) -> usize{
        let initializer = self.find_method(format!("init"));
        match initializer{
            Ok(init_method) => return init_method.arity(),
            Err(none) => return 0
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: &Vec<Value>) -> Result<Value, InterpreterError>{
        let instance = interpreter.create_instance(self.clone());
        let initializer = self.find_method(format!("init"));
        match initializer{
            Ok(mut init) => {
                match instance.clone(){
                    Value::LoxInstance(inst) => {
                        let init_func = init.bind(&inst).call(interpreter, args)?;
                        if let Value::UserDefined(def) = init_func{
                            let mut class_method = self.clone();
                            class_method.methods.insert(format!("init"), def);
                            let final_inst = LoxInstance::new(class_method);
                            interpreter.environment.instances.insert(format!("Test"), final_inst.clone());
                            let final_inst = Value::LoxInstance(Rc::new(final_inst));
                            return Ok(final_inst);
                            //inst.klass.methods.insert(format!("init"), def);
                        }
                        //inst.klass.methods.insert(format!("init"), )
                        //return Ok(instance)
                    }
                    _ => panic!("Unreachable init error")
                };
            }
            Err(err) => ()
        }
        return Ok(instance);
    }

    pub fn find_method(&self, name: String) -> Result<UserDefined, ()>{
        match self.methods.get(&name){
            Some(method) => return Ok(method.clone()),
            None => ()//return Err(())
        }
        match *self.superclass.clone(){
            Some(super_class) => return super_class.find_method(name),
            None => return Err(())
            // None => return Err(InterpreterError::new(
            //     format!("Can't find method {}", name), 
            //     0, 
            //     0, 
            //     Value::Nil
            // ))
        }
    }

    pub fn find_superclass_method(&mut self, name: String) -> Result<UserDefined, ()>{
        match *self.superclass.clone(){
            Some(mut super_class) => return super_class.find_superclass_method(name),
            None => ()
            // None => return Err(InterpreterError::new(
            //     format!("Can't find method {}", name), 
            //     0, 
            //     0, 
            //     Value::Nil
            // ))
        }
        match self.methods.get(&name){
            Some(method) => return Ok(method.clone()),
            None => return Err(())//return Err(())
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
                Ok(nothing) => {
                    if self.is_init{
                        return Ok(self.closure.get(&Expr::This { keyword: format!("this") }))?;
                    }
                    return Ok(Value::Nil)
                }
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
        let inst = instance.clone();
        let inst2 = inst.as_ref();
        let class = inst2.klass.as_ref();
        let super_c = *class.superclass.clone();
        match super_c{
            Some(sup) => environment.define(format!("super"), 0, 0, Some(Value::LoxClass(sup))),
            None => ()
        }
        return UserDefined {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            declaration: self.declaration.clone(),
            closure: environment,
            is_init: self.is_init
        }
    }
}