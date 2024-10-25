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
    use crate::expr::{Expr};
    use crate::stmt::{Stmt};
    use crate::interpreter::Interpreter;

    #[test]
    fn test_resolver_new() {
        let interpreter = Interpreter::new(Vec::<Stmt>::new()); // Pass an empty Vec<Stmt> as required
        let resolver = Resolver::new(interpreter);

        assert!(resolver.scopes.is_empty(), "Expected scopes to be empty");
        assert!(resolver.errors.is_empty(), "Expected errors to be empty");
        assert_eq!(resolver.state.function, FunctionState::None, "Expected function state to be None");
        assert_eq!(resolver.current_class, ClassState::None, "Expected current class to be None");
    }


    #[test]
    fn test_resolve_no_errors() {
        let interpreter = Interpreter::new(Vec::new());
        let mut resolver = Resolver::new(interpreter);

        let stmts = Vec::<Stmt>::new(); 
        let result = resolver.resolve(stmts).0; 

        assert_eq!(result, Ok(true), "Expected resolve result to be Ok(true)");
        assert!(resolver.errors.is_empty(), "Expected no errors");
    }

    // #[test]
    // fn test_resolve_with_errors() {
    //     let interpreter = Interpreter::new(Vec::new());
    //     let mut resolver = Resolver::new(interpreter);

    //     // Create a return statement with a Token instead of a String
    //     let return_token = Token::new(TokenType::Return, "return".to_string(), None, 1, 1); // Adjust line and col as needed
    //     let stmts = vec![Stmt::Return { keyword: return_token, value: None }];
    //     let result = resolver.resolve(stmts).0;

    //     // Check that it failed with an error
    //     assert!(result.is_err(), "Expected resolve result to be Err");
    //     assert!(!resolver.errors.is_empty(), "Expected at least one error");
    // }
}
