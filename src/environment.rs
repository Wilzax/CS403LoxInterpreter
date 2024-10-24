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

#[derive(Debug, Clone, PartialEq)]
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
                    format!("Incorrect use of 'this'"), 
                    0, 
                    0, 
                    Value::Nil
                ))
            }
        }
        else if let Expr::Super { keyword, method } = expr{
            match self.val_lookup(&expr){
                LookupResult::Ok(val) => Ok(val),
                _ => Err(InterpreterError::new(
                    format!("Incorrect use of 'super'"), 
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
        if self.values.contains_key(&name){
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
        else if let Expr::Super { keyword, method } = expr{
            let name = format!("super");
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

    pub fn get_at(&mut self, distance: usize, expr: Expr) -> Result<Value, InterpreterError>{
        return self.get(&expr);
    }
    pub fn ancestor(&self, distance: usize) -> Environment{
        let mut current: Environment = self.clone();
        for _ in 0..distance{
            current = *current.enclosing.unwrap();
        }
        return current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lox_callable::*;
    use crate::lox_instance::*;
    use crate::expr::{Expr};
    use crate::interpreter::{Value};
    use crate::scanner::{Token, TokenType};
    use std::collections::HashMap;
    use crate::lox_callable::{UserDefined, LoxClass};
    use crate::stmt::Stmt;


    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert_eq!(env.values, HashMap::new());
        assert!(env.enclosing.is_none());
        assert_eq!(env.user_func, HashMap::new());
        assert_eq!(env.classes, HashMap::new());
        assert_eq!(env.instances, HashMap::new());
    }

    #[test]
    fn test_environment_new() {
        let enclosing_env = Environment::default();

        let new_env = Environment::new(enclosing_env.clone());

        assert_eq!(new_env.values, HashMap::new());
        assert!(new_env.enclosing.is_some());
        let unwrapped_enclosing = new_env.enclosing.unwrap();
        assert_eq!(*unwrapped_enclosing, enclosing_env);
        assert_eq!(new_env.user_func, HashMap::new());
        assert_eq!(new_env.classes, HashMap::new());
        assert_eq!(new_env.instances, HashMap::new());
    }

    #[test]
    fn test_environment_full() {
        let testing_value = Value::Nil; 
        let testing_location = VarLocation { line: 1, col: 1 };

        let mut vals = HashMap::new();
        vals.insert(
            "testing_var".to_string(),
            (Some(testing_value.clone()), testing_location.clone())
        );

        let testing_token = Token {
            token_type: TokenType::Identifier,
            lexeme: vec![], 
            line: 1,
            column: 1,
            literal: None, 
        };

        let testing_func = UserDefined {
            name: "testing_func".to_string(),
            parameters: vec![testing_token.clone()],
            body: vec![], 
            declaration: Stmt::Function {
                name: "testing_func".to_string(),
                parameters: vec![testing_token.clone()],
                body: Box::new(vec![]), 
            },
            closure: Environment::default(),
            is_init: false,
        };

        let testing_class = LoxClass {
            name: "testing_class".to_string(),
            superclass: Box::new(None),
            methods: HashMap::new(),
        };

        let mut enclosing_env = Environment::default();
        enclosing_env.user_func.insert("testing_func".to_string(), testing_func.clone());
        enclosing_env.classes.insert("testing_class".to_string(), testing_class.clone());
        enclosing_env.instances.insert("testing_instance".to_string(), LoxInstance::new(testing_class.clone()));

        let new_env = Environment::full(vals.clone(), enclosing_env.clone());

        assert_eq!(new_env.values, vals);
        assert!(new_env.enclosing.is_some());
        let unwrapped_enclosing = new_env.enclosing.unwrap();
        assert_eq!(*unwrapped_enclosing, enclosing_env);
        assert_eq!(new_env.user_func, enclosing_env.user_func);
        assert_eq!(new_env.classes, enclosing_env.classes);
        assert_eq!(new_env.instances, enclosing_env.instances);
    }

    #[test]
    fn test_set_enclosing() {
        let mut base_env = Environment::default();

        let mut enclosing_env = Environment::default();
        
        let testing_value = Value::Nil; 
        let testing_location = VarLocation { line: 1, col: 1 };
        enclosing_env.define("enclosing_var".to_string(), testing_location.line, testing_location.col, Some(testing_value.clone()));

        base_env.set_enclosing(Some(enclosing_env.clone()));
        
        assert!(base_env.enclosing.is_some());
        let unwrapped_enclosing = base_env.enclosing.as_ref().unwrap();
        assert!(unwrapped_enclosing.values.contains_key("enclosing_var"));
        base_env.set_enclosing(None);
        assert!(base_env.enclosing.is_none());
    }

    #[test]
    fn test_set_values() {
        let mut env = Environment::default();

        let mut new_values = HashMap::new();
        let testing_value = Value::Nil; 
        let testing_location = VarLocation { line: 1, col: 1 };
        
        new_values.insert("var1".to_string(), (Some(testing_value.clone()), testing_location.clone()));
        new_values.insert("var2".to_string(), (Some(Value::Nil), VarLocation { line: 2, col: 2 }));
        
        env.set_values(new_values.clone());

        assert_eq!(env.values.len(), 2);
        assert_eq!(env.values.get("var1").unwrap(), new_values.get("var1").unwrap());
        assert_eq!(env.values.get("var2").unwrap(), new_values.get("var2").unwrap()); 
        
        let new_value = Value::Number(42.0); 
        new_values.insert("var1".to_string(), (Some(new_value.clone()), VarLocation { line: 3, col: 3 }));
        env.set_values(new_values.clone());

        assert_eq!(env.values.get("var1").unwrap().0, Some(new_value));
        assert_eq!(env.values.get("var1").unwrap().1.line, 3);
        assert_eq!(env.values.get("var1").unwrap().1.col, 3);
    }

    #[test]
    fn test_return_enclosing() {
        let env_without_enclosing = Environment::default();
        assert_eq!(env_without_enclosing.return_enclosing(), None);

        let mut env_with_enclosing = Environment::new(env_without_enclosing.clone());

        let returned_enclosing = env_with_enclosing.return_enclosing();
        assert!(returned_enclosing.is_some());

        let enclosed_env = returned_enclosing.unwrap();
        assert_eq!(*enclosed_env, env_without_enclosing);

        let second_env = Environment::new(env_with_enclosing.clone());
        let second_env_enclosing = second_env.return_enclosing();
        assert!(second_env_enclosing.is_some());

        let inner_enclosed_env = second_env_enclosing.unwrap();
        assert_eq!(*inner_enclosed_env, env_with_enclosing);
    }

    #[test]
    fn test_return_values() {
        let mut env = Environment::default();
        
        let testing_value_1 = (Some(Value::Number(42.0)), VarLocation { line: 1, col: 0 });
        let testing_value_2 = (Some(Value::String("test".to_string())), VarLocation { line: 2, col: 1 });

        let mut values_map = HashMap::new();
        values_map.insert("var1".to_string(), testing_value_1.clone());
        values_map.insert("var2".to_string(), testing_value_2.clone());

        env.set_values(values_map.clone());

        let returned_values = env.return_values();

        assert_eq!(returned_values.get("var1").unwrap(), &testing_value_1);
        assert_eq!(returned_values.get("var2").unwrap(), &testing_value_2);

        let testing_value_3 = (Some(Value::Bool(true)), VarLocation { line: 3, col: 2 });

        let mut values_map_modified = values_map.clone();
        values_map_modified.insert("var3".to_string(), testing_value_3.clone());

        env.set_values(values_map_modified.clone());

        let updated_returned_values = env.return_values();

        assert_eq!(updated_returned_values.get("var1").unwrap(), &testing_value_1);
        assert_eq!(updated_returned_values.get("var2").unwrap(), &testing_value_2);
        assert_eq!(updated_returned_values.get("var3").unwrap(), &testing_value_3);
    }

    #[test]
    fn test_define() {
        let mut env = Environment::default();

        let var_name_1 = "var1".to_string();
        let value_1 = Some(Value::Number(42.0));
        let var_loc_1 = VarLocation { line: 1, col: 0 };

        let var_name_2 = "var2".to_string();
        let value_2 = Some(Value::String("test".to_string()));
        let var_loc_2 = VarLocation { line: 2, col: 1 };

        env.define(var_name_1.clone(), var_loc_1.line, var_loc_1.col, value_1.clone());
        env.define(var_name_2.clone(), var_loc_2.line, var_loc_2.col, value_2.clone());

        let stored_value_1 = env.values.get(&var_name_1).unwrap();
        assert_eq!(stored_value_1, &(value_1, var_loc_1));

        let stored_value_2 = env.values.get(&var_name_2).unwrap();
        assert_eq!(stored_value_2, &(value_2, var_loc_2));

        let var_name_3 = "var3".to_string();
        let value_3 = None;
        let var_loc_3 = VarLocation { line: 3, col: 2 };

        env.define(var_name_3.clone(), var_loc_3.line, var_loc_3.col, value_3.clone());

        let stored_value_3 = env.values.get(&var_name_3).unwrap();
        assert_eq!(stored_value_3, &(value_3, var_loc_3));
    }

    #[test]
    fn test_define_token() {
        let mut env = Environment::default();

        let token_1 = Token {
            token_type: TokenType::Identifier,
            lexeme: b"var1".to_vec(),
            line: 1,
            column: 0,
            literal: None
        };

        let value_1 = Value::Number(42.0);

        env.define_token(token_1.clone(), value_1.clone());

        let stored_value_1 = env.values.get("var1").unwrap();

        assert_eq!(stored_value_1, &(Some(value_1), VarLocation { line: token_1.line, col: token_1.column }));

        let token_2 = Token {
            token_type: TokenType::Identifier,
            lexeme: b"var2".to_vec(),
            line: 2,
            column: 1,
            literal: None
        };

        let value_2 = Value::String("test".to_string());

        env.define_token(token_2.clone(), value_2.clone());

        let stored_value_2 = env.values.get("var2").unwrap();
        assert_eq!(stored_value_2, &(Some(value_2), VarLocation { line: token_2.line, col: token_2.column }));
    }

    #[test]
    fn test_define_string() {
        let mut env = Environment::default();

        let var_name_1 = "var1".to_string();
        let value_1 = Value::Number(42.0);

        env.define_string(var_name_1.clone(), value_1.clone());

        let stored_value_1 = env.values.get(&var_name_1).unwrap();

        assert_eq!(stored_value_1, &(Some(value_1), VarLocation { line: 0, col: 0 }));

        let var_name_2 = "var2".to_string();
        let value_2 = Value::String("test".to_string());

        env.define_string(var_name_2.clone(), value_2.clone());

        let stored_value_2 = env.values.get(&var_name_2).unwrap();
        assert_eq!(stored_value_2, &(Some(value_2), VarLocation { line: 0, col: 0 }));
    }  

    #[test]
    fn test_get_variable() {
        let mut env = Environment::default();
        let var_name = "x".to_string();
        let value = Value::Number(42.0);

        env.define_string(var_name.clone(), value.clone());

        let expr = Expr::Variable {
            name: var_name.clone(),
            line: 1,
            col: 1,
        };

        let result = env.get(&expr);

        match result {
            Ok(val) => assert_eq!(val, value),
            Err(_) => panic!("Expected Ok, got an error"),
        }
    }

    #[test]
    fn test_get_undefined_variable() {
        let env = Environment::default();

        let expr = Expr::Variable {
            name: "y".to_string(),
            line: 1,
            col: 1,
        };

        let result = env.get(&expr);

        match result {
            Err(err) => {
                assert_eq!(err.value, Value::Nil);
            },
            _ => panic!("Expected error, got {:?}", result),
        }
    }

    #[test]
    fn test_get_this() {
        let mut env = Environment::default();
        let value = Value::Number(100.0);
        env.define_string("this".to_string(), value.clone());

        let expr = Expr::This {
            keyword: "this".to_string(),
        };

        let result = env.get(&expr);

        match result {
            Ok(val) => assert_eq!(val, value),
            Err(_) => panic!("Expected Ok, got an error"),
        }
    }

    #[test]
    fn test_get_super() {
        let mut env = Environment::default();
        let value = Value::Number(200.0);
        env.define_string("super".to_string(), value.clone());

        let expr = Expr::Super {
            keyword: "super".to_string(),
            method: "someMethod".to_string(),
        };

        let result = env.get(&expr);

        match result {
            Ok(val) => assert_eq!(val, value),
            Err(_) => panic!("Expected Ok, got an error"),
        }
    }

    #[test]
    fn test_get_variable_undefined_but_declared() {
        let mut env = Environment::default();
        let var_name = "z".to_string();
        env.define(var_name.clone(), 2, 3, None); 

        let expr = Expr::Variable {
            name: var_name.clone(),
            line: 2,
            col: 3,
        };

        let result = env.get(&expr);

        match result {
            Err(err) => {
                assert_eq!(err.value, Value::Nil); 
            },
            _ => panic!("Expected error, got {:?}", result),
        }
    }

    #[test]
    fn test_assign_defined_variable() {
        let mut env = Environment::default();
        let var_name = "x".to_string();
        let initial_value = Value::Number(10.0);

        env.define(var_name.clone(), 1, 1, Some(initial_value.clone()));

        let new_value = Value::Number(20.0);
        let result = env.assign(var_name.clone(), 2, 1, &new_value);

        assert!(result.is_ok());

        let updated_value = env.get(&Expr::Variable {
            name: var_name.clone(),
            line: 2,
            col: 1,
        }).unwrap();

        assert_eq!(updated_value, new_value);
    }

    #[test]
    fn test_assign_undefined_variable() {
        let mut env = Environment::default();
    
        let result = env.assign("y".to_string(), 3, 1, &Value::Number(15.0));
    
        match result {
            Err(err) => {
                assert_eq!(err.value, Value::Nil); 
                assert_eq!(err.value, Value::Nil); 
            },
            _ => panic!("Expected error, got {:?}", result),
        }
    }    

    #[test]
    fn test_assign_variable_in_nested_environment() {
        let mut outer_env = Environment::default();
        outer_env.define("x".to_string(), 1, 1, Some(Value::Number(5.0)));
    
        let mut inner_env = Environment::new(outer_env.clone()); 
    
        inner_env.assign("x".to_string(), 2, 1, &Value::Number(30.0)).unwrap();
    
        let value_inner = inner_env.get(&Expr::Variable { name: "x".to_string(), line: 2, col: 1 }).unwrap();

        assert_eq!(value_inner, Value::Number(30.0));

        let value_outer = outer_env.get(&Expr::Variable { name: "x".to_string(), line: 1, col: 1 }).unwrap();
        
        assert_eq!(value_outer, Value::Number(5.0));
    }

    
    #[test]
    fn test_assign_at() {
        let mut outer_env = Environment::default();
        outer_env.define("x".to_string(), 1, 1, Some(Value::Number(10.0)));

        let mut inner_env = Environment::new(outer_env);

        let result = inner_env.assign_at("x".to_string(), 2, 1, &Value::Number(20.0), 1);

        assert!(result.is_ok());

        let updated_value = inner_env.get(&Expr::Variable { name: "x".to_string(), line: 1, col: 1 }).unwrap();

        assert_eq!(updated_value, Value::Number(20.0));
    }

    #[test]
    fn test_val_lookup_super() {
        let mut env = Environment::default();
        env.define("super".to_string(), 1, 1, Some(Value::Number(100.0)));
    
        let keyword_token = Token { 
            lexeme: b"super".to_vec(), 
            line: 1, 
            column: 1, 
            literal: None, 
            token_type: TokenType::Super 
        };
    
        let method_token = Token { 
            lexeme: b"method".to_vec(), 
            line: 1, 
            column: 1, 
            literal: None, 
            token_type: TokenType::Identifier 
        };
    
        let keyword_str = String::from_utf8(keyword_token.lexeme.clone()).unwrap();
        let method_str = String::from_utf8(method_token.lexeme.clone()).unwrap();
    
        let expr = Expr::Super { keyword: keyword_str, method: method_str };
        let result = env.val_lookup(&expr);
    
        match result {
            LookupResult::Ok(val) => assert_eq!(val, Value::Number(100.0)),
            _ => panic!("Expected Ok with Value::Number(100.0), got {:?}", result),
        }
    }
    

    #[test]
    fn test_get_at() {
        let mut outer_env = Environment::default();
        outer_env.define("x".to_string(), 1, 1, Some(Value::Number(42.0)));
    
        let mut inner_env = Environment::new(outer_env);
    
        inner_env.define("y".to_string(), 2, 1, Some(Value::Number(100.0)));

        let expr_x = Expr::Variable {
            name: "x".to_string(),
            line: 1,
            col: 1,
        };

        let expr_y = Expr::Variable {
            name: "y".to_string(),
            line: 2,
            col: 1,
        };

        let result_y = inner_env.get_at(0, expr_y.clone());
        assert!(result_y.is_ok());
        assert_eq!(result_y.unwrap(), Value::Number(100.0));

        let result_x = inner_env.get_at(1, expr_x.clone());
        assert!(result_x.is_ok());
        assert_eq!(result_x.unwrap(), Value::Number(42.0));
    }
    
    #[test]
    fn test_get_at_and_ancestor() {
        let mut outer_env = Environment::default();
        outer_env.define("x".to_string(), 1, 1, Some(Value::Number(42.0)));
    
        let mut middle_env = Environment::new(*Box::new(outer_env.clone()));
        middle_env.define("y".to_string(), 2, 1, Some(Value::Number(100.0)));
    
        let mut inner_env = Environment::new(*Box::new(middle_env.clone()));
        inner_env.define("z".to_string(), 3, 1, Some(Value::Number(200.0)));
    
        let expr_x = Expr::Variable {
            name: "x".to_string(),
            line: 1,
            col: 1,
        };
    
        let expr_y = Expr::Variable {
            name: "y".to_string(),
            line: 2,
            col: 1,
        };
    
        let expr_z = Expr::Variable {
            name: "z".to_string(),
            line: 3,
            col: 1,
        };
    
        let result_z = inner_env.get_at(0, expr_z.clone());
        assert!(result_z.is_ok());
        assert_eq!(result_z.unwrap(), Value::Number(200.0));
    
        let result_y = inner_env.get_at(1, expr_y.clone());
        assert!(result_y.is_ok());
        assert_eq!(result_y.unwrap(), Value::Number(100.0));
    
        let result_x = inner_env.get_at(2, expr_x.clone());
        assert!(result_x.is_ok());
        assert_eq!(result_x.unwrap(), Value::Number(42.0));
    
        let ancestor_1 = inner_env.ancestor(1);
        assert_eq!(ancestor_1.get(&expr_y).unwrap(), Value::Number(100.0));
    
        let ancestor_2 = inner_env.ancestor(2);
        assert_eq!(ancestor_2.get(&expr_x).unwrap(), Value::Number(42.0));
    }
    
}
