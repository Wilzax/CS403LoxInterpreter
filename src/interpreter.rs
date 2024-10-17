use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::environment::*;
use crate::lox_callable::*;
use crate::lox_instance::LoxInstance;
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
    LoxClass(LoxClass),
    LoxInstance(Rc<LoxInstance>),
    Nil,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Type{
    Number,
    String,
    Bool,
    UserDefined,
    NativeFunction,
    LoxClass,
    LoxInstance,
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
            Value::LoxClass(_) => Type::LoxClass,
            Value::LoxInstance(_) => Type::LoxInstance,
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
            Value::LoxClass(class) => format!("{}", class.name),
            Value::LoxInstance(inst) => format!("{} instance", inst.klass.name),
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
            Type::LoxClass => format!("User Defined Class"),
            Type::LoxInstance => format!("User Defined Class Instance"),
            Type::Nil => format!("Nil")
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter{
    pub statements: Vec<Stmt>,
    pub globals: Environment,
    pub environment: Environment,
    pub return_value: Option<Value>,
    pub locals: HashMap<String, (Expr, usize)>,
    pub classes: HashMap<String, LoxClass>,
    pub instances: HashMap<String, LoxInstance>,
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
            return_value: None,
            locals: HashMap::new(),
            classes: HashMap::new(),
            instances: HashMap::new()
        }   
    }
}

impl Interpreter{
    pub fn new(statements: Vec<Stmt>) -> Self{
        // Interpreter{
        //     statements: statements,
        //     globals: Environment::default(),
        //     environment: Environment::default(),
        //     return_value: None,
        // }
        let mut interp = Interpreter::default();
        interp.statements = statements;
        return interp
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), InterpreterError>{
        //let mut interp: Interpreter = Interpreter::new(statements.clone());
        //println!("We interpreting");
        for stmt in statements{
            let execution: Result<(), InterpreterError> = self.execute(stmt);
            match execution{
                Ok(stmt) => (),
                Err(err) => return Err(err)
            }
        }
        return Ok(())
    }

    // pub fn get_user_func(&self, name: String) -> UserDefined{
    //     match self.user_func.get(&name){
    //         Some(func) => func.clone(),
    //         None => panic!("Could not find function with name {}", name)
    //     }
    // }

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
                    column: 0,
                    value: Value::Nil 
                })
            }
            else{
                //println!("Func def");
                let function_inside = UserDefined{
                    name: name.clone(),
                    parameters: parameters,
                    body: *body,
                    declaration: stmt.clone(),
                    closure: self.environment.clone(),
                    is_init: false
                };
                let function = Value::UserDefined(function_inside.clone());
                self.environment.define(name.clone(), 0, 0, Some(function));
                self.environment.user_func.insert(name.clone(), function_inside.clone());
                self.globals.user_func.insert(name.clone(), function_inside);
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
            while Interpreter::is_truthy(self.evaluate(condition.clone())?){
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
                self.environment.define(name.clone(), line, column, opt.clone());
            }
            else{
                opt.insert(val.clone());
                self.environment.define(name.clone(), line, column, opt.clone());
            }
            // if self.environment == self.globals{
            //     if val == Value::Nil{
            //         self.globals.define(name.clone(), line, column, opt);
            //     }
            //     else{
            //         opt.insert(val);
            //         self.globals.define(name.clone(), line, column, opt);
            //     }
            // }
            return Ok(())
        }
        else{
            panic!("Unreachable Var Error")
        }
    }

    fn visit_return_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Return { keyword, value } = stmt{
            match value.clone(){
                Some(ret) => (),
                None => return Err(InterpreterError::new(format!("RETURN"), 0, 0, Value::Nil))
            }
            let retval = self.evaluate(value.unwrap());
            match retval{
                Ok(val) => Err(InterpreterError::new(format!("RETURN"), 0, 0, val)),
                Err(none) => Err(InterpreterError::new(format!("RETURN"), 0, 0, Value::Nil)) 
            }
        }
        else{
            panic!("Unreachable return error");
        }
    }

    fn visit_assign_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Assign { name, line, column, value } = expr{
            let val: Value = self.evaluate(*value)?;
            //let expression = self.environment.assign(name, line, column, &val.clone());

            let expression: Result<(), InterpreterError>;
            //println!("No");
            if let Some(result) = self.locals.get(&name){
                //println!("Checking for {} at depth {}", name.clone(), result.1.clone());
                expression = self.environment.assign_at(name, line, column, &val, result.1);
            }
            else{
                //println!("Will commit suicide for cash");
                expression = self.globals.assign(name, line, column, &val);
            }
            
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
                    column: col,
                    value: Value::Nil
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
                            column: col,
                            value: Value::Nil 
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
                        column: col,
                        value: Value::Nil 
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
            //println!("Callin");
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
                            column: 0,
                            value: Value::Nil 
                        })
                    }
                    else{

                        let func =  function.call(self, &args);
                        match func{
                            Ok(func) => return Ok(func),
                            Err(err) => return Err(InterpreterError { 
                                error_message: err, 
                                line: 0, 
                                column: 0,
                                value: Value::Nil  
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
                            column: 0,
                            value: Value::Nil 
                        })
                    }
                    else{
                        println!("User func call {}", args.len());
                        //let current_interp = self.environment.enclosing.clone();
                        let func =  function.call(self, &args);
                        //self.environment.enclosing = current_interp;
                        match func{
                            Ok(func) => return Ok(func),
                            Err(err) => return Err(err)
                        }
                    }
                }
                Value::LoxClass(class) =>{
                    if args.len() != class.arity() {
                        return Err(InterpreterError { 
                            error_message: format!("Expected {} arguments but got {}",
                            class.arity(), args.len()), 
                            line: 0, 
                            column: 0,
                            value: Value::Nil 
                        })
                    }
                    else{
                        let klas = class.call(self, &args);
                        match klas{
                            Ok(klas) => return Ok(klas),
                            Err(err) => return Err(err)
                        }
                    }
                }
                _ => {
                    return Err(InterpreterError { 
                        error_message: format!("Can only call functions and classes"), 
                        line: 0, 
                        column: 0,
                        value: Value::Nil 
                    });
                }
            }

        }
        else{
            panic!("Unreachable Call Error");
        }
    }

    fn visit_this_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::This { keyword } = expr.clone(){
            return self.lookup_variable(keyword, expr);
        }
        else{
            panic!("Unreachable This Error");
        }
    }

    fn visit_get_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Get { object, name } = expr{
            let value = self.evaluate(*object)?;
            let inst = Self::ensure_instance(value)?;
            return Ok(inst.get(name))?;
            // if let Value::LoxInstance(val) = value{
            //     return Ok(val.)?;
            // }
            // else{
            //     return Err(InterpreterError { 
            //         error_message: format!("Only objects have properties"), 
            //         line: 0, 
            //         column: 0, 
            //         value: value 
            //     })
            // }
        }
        else{
            panic!("Unreachable Get Error");
        }
    }

    fn visit_set_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Set { object, name, value } = expr{
            println!("Inside Set");
            let old_value = self.evaluate(*object)?;
            println!("Even farther");
            let mut instance = Interpreter::ensure_instance(old_value.clone())?;
            //let new_val = self.evaluate(*value)?;
            //instance.set(name.clone(), new_val.clone());
            //self.environment.instances.insert(name.clone(), instance);
            //return Ok(new_val)
            //if let Value::LoxInstance( val) = old_value{
                println!("Eeeeeeven farther");
                let new_val = self.evaluate(*value)?;
                println!("The farthest");
                instance.set(name, new_val.clone());
                return Ok(new_val);
            //}
        }
        else{
            panic!("Unreachable Set Error");
        }
    }

    fn ensure_instance(val: Value) -> Result<Rc<LoxInstance>, InterpreterError>{
        if let Value::LoxInstance(inst) = val{
            Ok(inst)
        }
        else{
            return Err(InterpreterError { 
                error_message: format!("Only objects have properties"), 
                line: 0, 
                column: 0, 
                value: Value::Nil 
            })
        }
    }

    fn visit_variable_expr(&mut self, expr: Expr) -> Result<Value, InterpreterError>{
        if let Expr::Variable { name , line: _ , col: _ } = expr.clone(){
            //return self.environment.get(&expr);
            return self.lookup_variable(name, expr);
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
        else if let Expr::Get { object: _ , name: _ } = expr{
            return Ok(self.visit_get_expr(expr))?;
        }
        else if let Expr::Set { object: _ , name: _ , value: _ } = expr{
            return Ok(self.visit_set_expr(expr))?;
        }
        else if let Expr::This { keyword: _ } = expr{
            return Ok(self.visit_this_expr(expr))?;
        }
        else{
            return Err(InterpreterError { 
                error_message: format!("We dont have that expression type yet bud"), 
                line: 0, 
                column: 0,
                value: Value::Nil 
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
        else if let Stmt::Class { name: _ , superclass: _ , methods: _ } = stmt{
            return Ok(self.visit_class_stmt(stmt))?;
        }
        else{
            //println!("Kys");
            return Err(InterpreterError { 
                error_message: format!("We dont have that statement type yet bud"), 
                line: 0, 
                column: 0,
                value: Value::Nil 
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
        if let Some(enclosing) = self.environment.enclosing.clone(){
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

    fn visit_class_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError>{
        if let Stmt::Class { name, superclass, methods } = stmt{
            self.environment.define(name.clone(), 0, 0, None);
            let mut method_hash: HashMap<String, UserDefined> = HashMap::new();
            let method_vec = *methods;
            for method in method_vec{
                if let Stmt::Function { name, parameters, body } = method.clone(){
                    let insert_method = UserDefined {
                        name: name.clone(),
                        parameters: parameters,
                        body: *body,
                        declaration: method.clone(),
                        closure: self.environment.clone(),
                        is_init: name.clone().eq(&format!("init"))
                    };
                    method_hash.insert(name, insert_method);
                }
            } 
            let klass: LoxClass = LoxClass { name: name.clone(), methods: method_hash };
            self.environment.assign(name.clone(), 0, 0, &Value::LoxClass(klass.clone()))?;
            self.environment.classes.insert(name.clone(), klass.clone());
            self.globals.classes.insert(name.clone(), klass.clone());
            return Ok(());
        }
        else{
            panic!("Unreachable class error");
        }
    }

    pub fn resolve_local(&mut self, name: String, depth: usize, expr: Expr) -> (){
        self.locals.insert(name, (expr, depth));
    }

    fn lookup_variable(&mut self, name: String, expr: Expr) -> Result<Value, InterpreterError>{
        if let Some(result) = self.locals.get(&name){
            return Ok(self.environment.get_at(result.1, result.0.clone()))?;
        }
        else{
            return Ok(self.globals.get(&expr))?;
        }
    }

    pub fn create_instance(&mut self, class: LoxClass) -> Value{
        let inst = LoxInstance::new(class);
        self.environment.instances.insert(format!("Test"), inst.clone());
        //POSSIBLE ISSUE
        return Value::LoxInstance(Rc::new(inst.clone()));
    }
}

pub struct InterpreterError{
    error_message: String,
    line: usize,
    column: i64,
    pub value: Value
}

impl InterpreterError{
    pub fn new(error_message: String, line: usize, column: i64, value: Value) -> Self{
        InterpreterError {
            error_message: error_message,
            line: line,
            column: column,
            value: value
        }
    }

    pub fn return_error(&self) -> String{
        return self.error_message.clone();
    }
}

// #[cfg(test)]
// mod tests{
//     use super::*;
//     use crate::scanner::*;
//     use crate::parser::*;
//     use crate::expr::*;

//     #[cfg(test)]
//     mod tests {
//         use super::*;
//         use crate::expr::{Expr, LiteralType};
//         use crate::stmt::{Stmt};
    
//         #[test]
//         fn simple_addition() {
//             let expr = Expr::Binary {
//                 left: Box::new(Expr::Literal { value: LiteralType::Number(3.0) }),
//                 operator: BinaryOpType::Plus,
//                 right: Box::new(Expr::Literal { value: LiteralType::Number(4.0) }),
//                 line: 1,
//                 col: 1,
//             };
//             let stmt = Stmt::Expr { expression: Box::new(expr) };
//             let result = Interpreter::interpret(vec![stmt]);
            
//             match result {
//                 Ok(_) => assert!(true, "Expected no errors during addition"),
//                 Err(err) => panic!("Error when interpreting: {}", err.return_error()),
//             }
//         }

//         #[test]
//         fn simple_subtraction() {
//             let expr = Expr::Binary {
//                 left: Box::new(Expr::Literal { value: LiteralType::Number(3.0) }),
//                 operator: BinaryOpType::Minus,
//                 right: Box::new(Expr::Literal { value: LiteralType::Number(4.0) }),
//                 line: 1,
//                 col: 1,
//             };
//             let stmt = Stmt::Expr { expression: Box::new(expr) };
//             let result = Interpreter::interpret(vec![stmt]);
            
//             match result {
//                 Ok(_) => assert!(true, "Expected no errors during subtraction"),
//                 Err(err) => panic!("Error when interpreting: {}", err.return_error()),
//             }
//         }

//         #[test]
//         fn simple_multiplication() {
//             let expr = Expr::Binary {
//                 left: Box::new(Expr::Literal { value: LiteralType::Number(3.0) }),
//                 operator: BinaryOpType::Star,
//                 right: Box::new(Expr::Literal { value: LiteralType::Number(4.0) }),
//                 line: 1,
//                 col: 1,
//             };
//             let stmt = Stmt::Expr { expression: Box::new(expr) };
//             let result = Interpreter::interpret(vec![stmt]);
            
//             match result {
//                 Ok(_) => assert!(true, "Expected no errors during multiplication"),
//                 Err(err) => panic!("Error when interpreting: {}", err.return_error()),
//             }
//         }

//         #[test]
//         fn simple_division() {
//             let expr = Expr::Binary {
//                 left: Box::new(Expr::Literal { value: LiteralType::Number(3.0) }),
//                 operator: BinaryOpType::Slash,
//                 right: Box::new(Expr::Literal { value: LiteralType::Number(4.0) }),
//                 line: 1,
//                 col: 1,
//             };
//             let stmt = Stmt::Expr { expression: Box::new(expr) };
//             let result = Interpreter::interpret(vec![stmt]);
            
//             match result {
//                 Ok(_) => assert!(true, "Expected no errors during division"),
//                 Err(err) => panic!("Error when interpreting: {}", err.return_error()),
//             }
//         }
    
//         #[test]
//         fn variable_assignment() {
//             let var_stmt = Stmt::Var {
//                 name: "x".to_string(),
//                 line: 1,
//                 column: 1,
//                 initializer: Some(Expr::Literal { value: LiteralType::Number(42.0) }),  
//             };
//             let print_stmt = Stmt::Print {
//                 expression: Box::new(Expr::Variable { name: "x".to_string(), line: 1, col: 1 }),
//             };
//             let result = Interpreter::interpret(vec![var_stmt, print_stmt]);
    
//             match result {
//                 Ok(_) => assert!(true, "Expected no errors during variable assignment"),
//                 Err(err) => panic!("Error when interpreting: {}", err.return_error()),
//             }
//         }
    
//         #[test]
//         fn print_statement() {
//             let print_stmt = Stmt::Print {
//                 expression: Box::new(Expr::Literal { value: LiteralType::String("Hello, World!".to_string()) }),
//             };
//             let result = Interpreter::interpret(vec![print_stmt]);
    
//             match result {
//                 Ok(_) => assert!(true, "Expected no errors during printing"),
//                 Err(err) => panic!("Error when interpreting: {}", err.return_error()),
//             }
//         }
    
//         #[test]
//         fn division_by_zero() {
//             let expr = Expr::Binary {
//                 left: Box::new(Expr::Literal { value: LiteralType::Number(10.0) }),
//                 operator: BinaryOpType::Slash,
//                 right: Box::new(Expr::Literal { value: LiteralType::Number(0.0) }),
//                 line: 1,
//                 col: 1,
//             };
//             let stmt = Stmt::Expr { expression: Box::new(expr) };
//             let mut interpreter = Interpreter::new(stmt.clone());
//             let result = interpreter.interpret(vec![stmt]);
    
//             match result {
//                 Ok(_) => panic!("Expected an error during division by zero"),
//                 Err(err) => assert_eq!(err.error_message, "Divide by zero error at line: 1, column: 1"),
//             }
//         }
        

//         #[test]
//         fn addition_produces_correct_response() {
//             let expr = Expr::Binary {
//                 left: Box::new(Expr::Literal { value: LiteralType::Number(130.0) }),
//                 operator: BinaryOpType::Plus,
//                 right: Box::new(Expr::Literal { value: LiteralType::Number(58.0) }),
//                 line: 1,
//                 col: 1,
//             };
            
//             let stmt = Stmt::Expr { expression: Box::new(expr) };
//             let result = Interpreter::interpret(vec![stmt]);
        
//             match result {
//                 Ok(_) => {
//                     let expected_value = Value::Number(188.0);
//                     assert_eq!(expected_value, Value::Number(188.0)); 
//                 },
//                 Err(err) => panic!("Expected correct addition, but got an error: {}", err.return_error()),
//             }
//         }

//         #[test]
//         fn operator_precedence() {
//             let expr = Expr::Binary {
//                 left: Box::new(Expr::Binary {
//                     left: Box::new(Expr::Literal { value: LiteralType::Number(3.0) }),
//                     operator: BinaryOpType::Plus,
//                     right: Box::new(Expr::Literal { value: LiteralType::Number(2.0) }),
//                     line: 1,
//                     col: 1,
//                 }),
//                 operator: BinaryOpType::Star,
//                 right: Box::new(Expr::Literal { value: LiteralType::Number(4.0) }),
//                 line: 1,
//                 col: 1,
//             };
//             let stmt = Stmt::Expr { expression: Box::new(expr) };
//             let result = Interpreter::interpret(vec![stmt]);
//             assert!(result.is_ok());
//         }
        
//     }
    
// }
