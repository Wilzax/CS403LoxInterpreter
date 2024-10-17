use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::*;
use crate::expr::*;
use crate::lox_callable::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance{
    //pub name: String,
    pub klass: Rc<LoxClass>,
    pub fields: RefCell<HashMap<String, Value>>
}

// impl Default for LoxInstance{
//     fn default() -> Self {
//         LoxInstance{
            
//         }
//     }
// }

impl LoxInstance{
    pub fn new(klass: LoxClass) -> Self{
        LoxInstance { 
            klass: Rc::new(klass),
            fields: RefCell::new(HashMap::new()) 
        }
    }

    pub fn get(self: &Rc<Self>, name: String) -> Result<Value, InterpreterError>{
        match self.fields.borrow_mut().get(&name){
            Some(val) => return Ok(val.clone()),
            None => {
                let method = self.klass.find_method(name.clone());
                match method{
                    Ok(mut ret_method) => return Ok(Value::UserDefined(ret_method)),
                    Err(err) => return Err(InterpreterError::new(
                                    format!("Undefined property '{}'", name),
                                    0,
                                    0,
                                    Value::Nil))
                }
            }
        }

    }

    pub fn set(&self, name: String, value: Value) -> (){
        self.fields.borrow_mut().insert(name, value);
        println!("insert");
    }

    pub fn to_string(&self) -> String{
        return format!("{} instance", self.klass.name);
    }
}