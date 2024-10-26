use std::collections::HashMap;
use std::collections::VecDeque;
use std::mem;
use crate::stmt::*;
use crate::expr::*;
use crate::interpreter;
use crate::interpreter::*;
use crate::environment::*;

pub struct Resolver{
    pub interpreter: Interpreter,
    pub scopes: Vec<HashMap<String, bool>>,
    pub errors: Vec<String>,
    pub state: ResolverState,
    pub current_class: ClassState
}

#[derive(Debug)]
struct ResolverState{
    function: FunctionState
}

impl Default for ResolverState{
    fn default() -> Self {
        ResolverState{
            function: FunctionState::None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FunctionState{
    Function,
    Method,
    Init,
    None
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClassState{
    Class,
    SubClass,
    None
}

impl Resolver{
    pub fn new(interpreter: Interpreter) -> Self{
        Resolver { 
            interpreter: interpreter,
            scopes: Vec::new(),
            errors: Vec::new(),
            state: ResolverState::default(),
            current_class: ClassState::None
        }        
    }

    pub fn resolve(&mut self, stmts: Vec<Stmt>) -> (Result<bool, Vec<String>>, &Interpreter){
        self.begin_scope();
        self.resolve_vec_stmt(stmts);
        if self.errors.is_empty(){
            return (Ok(true), &self.interpreter);
        }
        self.end_scope();
        return (Err(self.errors.clone()), &self.interpreter);
    }

    fn resolve_vec_stmt(&mut self, stmts: Vec<Stmt>) -> (){
        for stmt in stmts{
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: Stmt) -> (){
        match stmt.clone(){
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_vec_stmt(statements);
                self.end_scope();
            }
            Stmt::Class { name, superclass , methods } => {
                let enclosing_class = self.current_class;
                self.begin_scope();
                self.current_class = ClassState::Class;
                let x = self.scopes.last_mut();
                match x{
                    Some(scop) => scop.insert(format!("this"), true),
                    None => return ()
                };
                self.declare(name.clone());
                self.define(name.clone());
                let class_name = name.clone();
                let mut is_super = false;
                match superclass{
                    Some(sup) => {
                        self.current_class = ClassState::SubClass;
                        is_super = true;
                        if let Expr::Variable { name, line: _ , col: _ } = sup.clone(){
                            if name.eq(&class_name){
                                self.errors.push(format!("A class can't inherit from itself."));
                            }
                            self.resolve_expr(sup);
                        }
                    }
                    None => ()
                }
                if is_super{
                    self.begin_scope();
                    let x = self.scopes.last_mut();
                    match x{
                        Some(scop) => scop.insert(format!("super"), true),
                        None => return ()
                    };
                }
                for method in *methods{
                    let mut declaration = FunctionState::Method;
                    if let Stmt::Function { name, parameters, body } = method.clone(){
                        if name.eq(&format!("init")){
                            declaration = FunctionState::Init
                        }
                    }
                    self.resolve_function(method, declaration);
                }
                self.end_scope();
                if is_super{
                    self.end_scope();
                }
                self.current_class = enclosing_class;
            }
            Stmt::Expr { expression } => {
                self.resolve_expr(*expression);
            }
            Stmt::Function { name, parameters: _ , body: _ } => {
                self.declare(name.clone());
                self.define(name);
                self.resolve_function(stmt, FunctionState::Function);
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.resolve_expr(*condition);
                self.resolve_stmt(*then_branch);
                if let Some(else_branch) = else_branch{
                    self.resolve_stmt(*else_branch);
                }
            }
            Stmt::Print { expression } => {
                self.resolve_expr(*expression);
            }
            Stmt::Return { keyword, value } => {
                if self.state.function == FunctionState::None{
                    self.errors.push(format!("Illegal return statement"));
                }
                if let Some(value) = value{
                    if self.state.function == FunctionState::Init{
                        self.errors.push(format!("Can't return a value from an initializer"));
                    }
                    self.resolve_expr(value);
                }
            }
            Stmt::Var { name, line, column, initializer } => {
                //println!("Resolving {} Definition", name.clone());
                self.declare(name.clone());
                if let Some(init) = initializer{
                    //println!("Innside {} init", name.clone());
                    self.resolve_expr(init);
                }
                self.define(name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*body);
            }
        }
    }

    fn resolve_expr(&mut self, expr: Expr) -> (){
        match expr.clone(){
            Expr::Assign { name, line, column, value } => {
                self.resolve_expr(*value);
                self.resolve_local(name.clone(), Expr::Variable { name: name.clone(), line: line, col: column });
            }
            Expr::Binary { left, operator: _ , right, line: _ , col: _ } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Call { callee, paren: _ , arguments } => {
                self.resolve_expr(*callee);
                for arg in *arguments{
                    self.resolve_expr(arg);
                }
            }
            Expr::Get { object, name } => {
                self.resolve_expr(*object);
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(*expression);
            }
            Expr::Literal { value: _ } => {
                ()
            }
            Expr::Logical { left, operator: _ , right } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Set { object, name: _ , value } => {
                self.resolve_expr(*object);
                self.resolve_expr(*value);
            }
            Expr::Super { keyword, method: _ } => {
                if self.current_class == ClassState::None{
                    self.errors.push(format!("Can't use 'super' outside of a class"));
                }
                else if self.current_class != ClassState::SubClass{
                    self.errors.push(format!("Can't use 'super' in a class with no superclass"));
                }
                self.resolve_local(keyword, expr);
            }
            Expr::This { keyword } => {
                if self.current_class == ClassState::None{
                    self.errors.push(format!("Can't use 'this' outside of class."));
                    return ();
                }
                self.resolve_local(keyword, expr);
            }
            Expr::Unary { operator: _ , right, line: _ , col: _ } => {
                self.resolve_expr(*right);
            }
            Expr::Variable { name, line: _ , col: _ } => {
                //println!("Resolving {} Expression", name.clone());
                if self.query(name.clone(), false){
                    self.errors.push(format!("Cannot read local variable in its own initializer"));
                    return ();
                }
                //println!("Resolving {} Expression Pt 2", name.clone());
                self.resolve_local(name, expr);
            }
            Expr::None => {
                self.errors.push(format!("Wtf are you doing here"));
            }
        }
    }

    fn declare(&mut self, name: String) -> (){
        if self.scopes.is_empty(){
            return ();
        }
        if let Some(scope) = self.scopes.last_mut(){
            scope.insert(name, false);
        }
    }

    fn define(&mut self, name: String) -> (){
        if self.scopes.is_empty(){
            return ();
        }
        if let Some(scope) = self.scopes.last_mut(){
            match scope.get(&name){
                Some(isInit) => {
                    scope.insert(name, true);
                    return ();
                }
                None => {
                    self.errors.push(format!("Variable {} is not defined", name));
                }
            }
        }
    }

    fn query(&mut self, name: String, state: bool) -> bool{
        return self.scopes.last().and_then(|scope| scope.get(&name)) == Some(&state);
    }

    fn resolve_local(&mut self, name: String, expr: Expr){
        for (depth, scope) in self.scopes.iter().rev().enumerate(){
            //println!("Depth: {}", depth);
            if scope.contains_key(&name){
                //println!("HERE {}", name.clone());
                self.interpreter.resolve_local(name, depth, expr);
                return ();
            }
        }
    }

    fn resolve_function(&mut self, stmt: Stmt, state: FunctionState){
        if let Stmt::Function { name, parameters, body } = stmt{
            let prior_state = mem::replace(&mut self.state.function, state);
            self.begin_scope();
            for param in parameters{
                self.declare(String::from_utf8(param.lexeme.clone()).unwrap());
                self.define(String::from_utf8(param.lexeme).unwrap());
            }
            self.resolve_vec_stmt(*body);
            self.end_scope();
            self.state.function = prior_state;
        }
        else{
            panic!("Unreachable function resolver");
        }
    }

    fn begin_scope(&mut self) -> (){
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) -> (){
        self.scopes.pop();
    }


    #[inline]
    fn scoped<I>(&mut self, inner: I)
    where I: FnOnce(&mut Self),
    {
        self.begin_scope();
        let res = inner(self);
        self.end_scope();
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Expr, LiteralType};
    use crate::stmt::{Stmt};
    use crate::interpreter::Interpreter;
    use crate::{resolver::Resolver};
    use crate::scanner::Literal;

    #[test]
    fn test_resolver_new() {
        let interpreter = Interpreter::new(Vec::<Stmt>::new());
        let resolver = Resolver::new(interpreter);

        assert!(resolver.scopes.is_empty(), "Expected scopes to be empty");
        assert!(resolver.errors.is_empty(), "Expected errors to be empty");
        assert_eq!(resolver.state.function, FunctionState::None, "Expected function state to be None");
        assert_eq!(resolver.current_class, ClassState::None, "Expected current class to be None");
    }

    #[test]
    fn test_resolve() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);

        let stmts = Vec::<Stmt>::new();
        let result = resolver.resolve(stmts).0;

        assert_eq!(result, Ok(true), "Expected resolve result to be Ok(true)");
        assert!(resolver.errors.is_empty(), "Expected no errors");
    }

    #[test]
    fn test_resolve_vec_stmt() {
        let stmt1 = Stmt::Print {
            expression: Box::new(Expr::Literal { value: LiteralType::Number(1.0) }),
        };
        let stmt2 = Stmt::Var {
            name: "x".to_string(),
            initializer: Some(Expr::Literal { value: LiteralType::Number(10.0) }),
            line: 1,
            column: 5,
        };
    
        let stmts = vec![stmt1, stmt2];
    
        let interpreter = Interpreter::new(Vec::new());
    
        let mut resolver = Resolver::new(interpreter);
    
        resolver.resolve_vec_stmt(stmts);
    
        assert!(resolver.errors.is_empty(), "Resolver encountered errors: {:?}", resolver.errors);
    }
    
    #[test]
    fn test_resolve_stmt() {
        let stmt_block = Stmt::Block {
            statements: vec![Stmt::Print {
                expression: Box::new(Expr::Literal { value: LiteralType::Number(1.0) }),
            }],
        };
        let stmt_class = Stmt::Class {
            name: "MyClass".to_string(),
            superclass: None,
            methods: Box::new(vec![]),
        };
        let stmt_var = Stmt::Var {
            name: "y".to_string(),
            initializer: Some(Expr::Literal { value: LiteralType::Number(20.0) }),
            line: 2,
            column: 5,
        };
    
        let interpreter = Interpreter::new(Vec::new());
    
        let mut resolver = Resolver::new(interpreter);
    
        resolver.resolve_stmt(stmt_block.clone());
        resolver.resolve_stmt(stmt_class.clone());
        resolver.resolve_stmt(stmt_var.clone());
    
        assert!(resolver.errors.is_empty(), "Resolver encountered errors: {:?}", resolver.errors);
    }
    
    #[test]
    fn test_resolve_expr() {
        let expr_assign = Expr::Assign {
            name: "a".to_string(),
            line: 1,
            column: 1,
            value: Box::new(Expr::Literal { value: LiteralType::Number(42.0) }),
        };
        let expr_binary = Expr::Binary {
            left: Box::new(Expr::Literal { value: LiteralType::Number(1.0) }),
            operator: BinaryOpType::Plus,
            right: Box::new(Expr::Literal { value: LiteralType::Number(2.0) }),
            line: 1,
            col: 3,
        };
        let expr_grouping = Expr::Grouping {
            expression: Box::new(Expr::Literal { value: LiteralType::Number(3.0) }),
        };
    
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);

        resolver.resolve_expr(expr_assign.clone());
        resolver.resolve_expr(expr_binary.clone());
        resolver.resolve_expr(expr_grouping.clone());

        assert!(resolver.errors.is_empty(), "Resolver encountered errors: {:?}", resolver.errors);
    }
    
    #[test]
    fn test_declare() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);

        resolver.begin_scope();
        resolver.declare("test_var".to_string());

        let current_scope = resolver.scopes.last().expect("Expected at least one scope");
        assert!(current_scope.contains_key("test_var"), "Expected 'test_var' to be declared in the current scope");
        assert!(!current_scope["test_var"], "Expected 'test_var' to be declared but not defined");
    }
    
    #[test]
    fn test_define() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);
        resolver.begin_scope();
        resolver.declare("test_var".to_string());
        resolver.define("test_var".to_string());

        let current_scope = resolver.scopes.last().expect("Expected at least one scope");
        assert!(current_scope.contains_key("test_var"), "Expected 'test_var' to be defined in the current scope");
        assert!(current_scope["test_var"], "Expected 'test_var' to be defined");

        resolver.define("undefined_var".to_string());
        assert!(!resolver.errors.is_empty(), "Expected an error for undefined variable");
        assert_eq!(resolver.errors[0], "Variable undefined_var is not defined", "Expected error message for undefined variable");
    }
    
    #[test]
    fn test_query() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);
        resolver.begin_scope();
        resolver.declare("test_var".to_string());

        let query_result = resolver.query("test_var".to_string(), false);
        assert!(query_result, "Expected 'test_var' to be found with state false");

        resolver.define("test_var".to_string());

        let query_result = resolver.query("test_var".to_string(), true);
        assert!(query_result, "Expected 'test_var' to be found with state true");

        let query_result = resolver.query("undefined_var".to_string(), false);
        assert!(!query_result, "Expected 'undefined_var' not to be found");
    }
    
    #[test]
    fn test_resolve_local() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);
        resolver.begin_scope();
        resolver.declare("test_var".to_string());

        let expr = Expr::Literal { value: LiteralType::Number(1.0) };
        resolver.resolve_local("test_var".to_string(), expr);

        assert!(resolver.errors.is_empty(), "Expected no errors after resolving local");
    }
    
    #[test]
    fn test_resolve_function() {
        let function_stmt = Stmt::Function {
            name: "my_function".to_string(),
            parameters: vec![
                crate::scanner::Token {
                    token_type: crate::scanner::TokenType::Identifier,
                    lexeme: b"param".to_vec(),
                    literal: None,
                    line: 1,
                    column: 5,
                }
            ],
            body: Box::new(vec![
                Stmt::Print {
                    expression: Box::new(Expr::Literal { value: LiteralType::Number(42.0) }),
                }
            ]),
        };
    
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);
        resolver.resolve_function(function_stmt.clone(), FunctionState::Function);

        assert!(resolver.errors.is_empty(), "Resolver encountered errors: {:?}", resolver.errors);
    }

    #[test]
    fn test_begin_scope() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);
        resolver.begin_scope();

        assert_eq!(resolver.scopes.len(), 1, "Expected one scope to be added");
    }

    #[test]
    fn test_end_scope() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);
        resolver.begin_scope();

        assert_eq!(resolver.scopes.len(), 1, "Expected one scope to be added");

        resolver.end_scope();

        assert_eq!(resolver.scopes.len(), 0, "Expected the scope to be removed");
    }

    #[test]
    fn test_scoped() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);

        resolver.scoped(|res| {
            res.declare("scoped_var".to_string());
            res.define("scoped_var".to_string());
            assert!(res.scopes.last().unwrap().contains_key("scoped_var"), "Expected 'scoped_var' to be in the current scope");
            assert!(res.scopes.last().unwrap()["scoped_var"], "Expected 'scoped_var' to be defined");
        });

        assert!(resolver.scopes.is_empty(), "Expected all scopes to be removed after scoped block");
    }
}