use crate::scanner::{self, Literal};
use crate::scanner::{Token, TokenType};
use crate::expr;
use crate::expr::Expr;
use crate::stmt::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Parser{
    tokens: Vec<Token>,
    current: usize,
}

impl Default for Parser{
    fn default() -> Parser{
        Parser{
            tokens: Vec::new(),
            current: 0,
        }
    }
}

/*
Current Parser Grammer:
program        → statement* EOF ;

declaration    → classDecl
               | funDecl
               | varDecl
               | statement ;

classDecl      → "class" IDENTIFIER ( "<" IDENTIFIER )?
                 "{" function* "}" ;
funDecl        → "fun" function ;
function       → IDENTIFIER "(" parameters? ")" block ;
parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
statement      → exprStmt
               | forStmt
               | ifStmt
               | printStmt
               | returnStmt
               | whileStmt
               | block ;

returnStmt     → "return" expression? ";" ;

forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
                 expression? ";"
                 expression? ")" statement ;

whileStmt      → "while" "(" expression ")" statement ;

block          → "{" declaration* "}" ;

exprStmt       → expression ";" ;
ifStmt         → "if" "(" expression ")" statement ( "else" statement )? ;     
printStmt      → "print" expression ";" ;
varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
expression     → assignment ;
assignment     → ( call "." )? IDENTIFIER "=" assignment
               | logic_or ;
logic_or       → logic_and ( "or" logic_and )* ;
logic_and      → equality ( "and" equality )* ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | call ;
call           → primary ( "(" arguments? ")" )* ;
arguments      → expression ( "," expression )* ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" 
               | IDENTIFIER ;


*/

impl Parser{
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError>{
        let mut statements:Vec<Stmt> = Vec::new();
        while !self.is_at_end(){
            let stmt: Result<Stmt, ParserError> = self.declaration();
            match stmt{
                Ok(stmt) => statements.push(stmt),
                Err(err) => return Err(err)
            }
        }
        return Ok(statements);
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError>{
        if self.matches(vec![TokenType::Var]){
            return self.var_declaration();
        }
        if self.matches(vec![TokenType::Fun]){
            return self.function(format!("function"));
        }
        if self.matches(vec![TokenType::Class]){
            return self.class_declaration();
        }
        return self.statement();
    }

    fn class_declaration(&mut self) -> Result<Stmt, ParserError>{
        let name = self.consume(TokenType::Identifier, format!("Expect class name"))?;
        let mut superclass: Option<Expr> = None;
        if self.matches(vec![TokenType::Less]){
            let sup = self.consume(TokenType::Identifier, format!("Expect superclass name"))?;
            superclass = Some(Expr::Variable { name:String::from_utf8(sup.lexeme).unwrap(), line: sup.line, col: sup.column })
        }
        self.consume(TokenType::LeftBrace, format!("Expect '{{' before class body"))?;
        let mut methods: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end(){
            methods.push(self.function(format!("method"))?);
        }
        self.consume(TokenType::RightBrace, format!("Expect '}}' after class body"))?;
        return Ok(Stmt::Class { 
            name: String::from_utf8(name.lexeme).unwrap(), 
            superclass: superclass, 
            methods: Box::new(methods) 
        })
    }

    fn function(&mut self, kind: String) -> Result<Stmt, ParserError>{
        let name: Token = self.consume(TokenType::Identifier, format!("Expect {} name", kind))?;
        let l_paren: Token = self.consume(TokenType::LeftParen, format!("Expect '(' after {} name", kind))?;
        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen){
            loop{
                if parameters.len() >= 255{
                    return Err(ParserError { 
                        message: format!("Cannot have more than 255 parameters"), 
                        token_type: TokenType::None, 
                        line: 0, 
                        column: 0 
                    })
                }
                parameters.push(self.consume(TokenType::Identifier, format!("Expect parameter name"))?);
                if !self.matches(vec![TokenType::Comma]){
                    break;
                }
            } 
        }
        let paren: Token = self.consume(TokenType::RightParen, format!("Expect ')' after parameters"))?;
        let brace: Token = self.consume(TokenType::LeftBrace, format!("Expect '{{' before {} body", kind))?;
        let body: Vec<Stmt> = self.block()?;
        return Ok(Stmt::Function { 
            name: String::from_utf8(name.lexeme).unwrap(), 
            parameters: parameters, 
            body: Box::new(body) 
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError>{
        let name = self.consume(TokenType::Identifier, format!("Expect variable name"))?;
        let init = if self.matches(vec![TokenType::Equal]) {
            //println!("Howdy");
            Some(self.expression()?)
        }
        else{
            None
        };
        self.consume(TokenType::Semicolon, format!("Expect ';' after variable declaration"))?;
        //println!("{}", String::from_utf8(name.lexeme.clone()).unwrap());
        return Ok(Stmt::Var { name: String::from_utf8(name.lexeme).unwrap(), line: name.line, column: name.column ,initializer: init });
    }

    fn statement(&mut self) -> Result<Stmt, ParserError>{
        if self.matches(vec![TokenType::Print]){
            return self.print_statement();
        }
        if self.matches(vec![TokenType::LeftBrace]){
            let statements: Vec<Stmt> = self.block()?;
            return Ok(Stmt::Block { statements: statements });
        }
        if self.matches(vec![TokenType::If]){
            return self.if_statement();
        }
        if self.matches(vec![TokenType::While]){
            return self.while_statement();
        }
        if self.matches(vec![TokenType::For]){
            return self.for_statement();
        }
        if self.matches(vec![TokenType::Return]){
            return self.return_statement();
        }
        return self.expression_statement();
    }

    fn for_statement(&mut self) -> Result<Stmt, ParserError>{
        let start_condition: Token = self.consume(TokenType::LeftParen, format!("Expect '(' after 'for'"))?;
        let initializer: Option<Stmt>;
        if self.matches(vec![TokenType::Semicolon]){
            initializer = None;
        }
        else if self.matches(vec![TokenType::Var]){
            initializer = Some(self.var_declaration()?);
        }
        else{
            initializer = Some(self.expression_statement()?);
        }
        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::Semicolon){
            condition = Some(self.expression()?);
        }
        let semi = self.consume(TokenType::Semicolon, format!("Expect ';' after loop condition"))?;
        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen){
            increment = Some(self.expression()?);
            //println!("Right here");
        }
        let semi = self.consume(TokenType::RightParen, format!("Expect ')' after for clauses"))?;
        let mut body: Stmt = self.statement()?;
        if increment != None{
            body = Stmt::Block { statements: vec![body, Stmt::Expr { expression: Box::new(increment.unwrap()) }] };
        }
        if condition == None{
            condition = Some(Expr::Literal { value: expr::LiteralType::True });
        }
        body = Stmt::While { condition: condition.unwrap(), body: Box::new(body) };
        if initializer != None{
            body = Stmt::Block { statements: vec![initializer.unwrap(), body] };
        }
        let final_body = body;
        return Ok(final_body);
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError>{
        let value: Expr = self.expression()?;
        let correct_end: Result<Token, ParserError> = self.consume(TokenType::Semicolon, format!("Expect ';' after value"));
        match correct_end{
            Ok(token) => return Ok(Stmt::Print { expression: Box::new(value) }),
            Err(err) => return Err(err)
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, ParserError>{
        let keyword: Token = self.previous();
        let mut value = Expr::None;
        if !self.check(TokenType::Semicolon){
            value = self.expression()?;
        }
        let semi = self.consume(TokenType::Semicolon, format!("Expect ';' after return value"))?;
        if let Expr::None = value.clone(){
            return Ok(Stmt::Return { keyword: keyword, value: None })
        }
        else{
            return Ok(Stmt::Return { keyword: keyword, value: Some(value) })
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError>{
        let value: Expr = self.expression()?;
        let correct_end: Result<Token, ParserError> = self.consume(TokenType::Semicolon, format!("Expect ';' after value"));
        match correct_end{
            Ok(token) => return Ok(Stmt::Expr { expression: Box::new(value) }),
            Err(err) => return Err(err)
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParserError>{
        let start_condition: Result<Token, ParserError> = self.consume(TokenType::LeftParen, format!("Expect '(' after 'if'"));
        let condition: Expr = self.expression()?;
        let end_condition: Result<Token, ParserError> = self.consume(TokenType::RightParen, format!("Expect ')' after conditional statement"));
        let then_branch: Stmt = self.statement()?;
        let else_branch:Option<Box<Stmt>>;
        if self.matches(vec![TokenType::Else]){
            else_branch = Some(Box::new(self.statement()?));
        }
        else{
            else_branch = None;
        };
        return Ok(Stmt::If { condition: Box::new(condition), then_branch: Box::new(then_branch), else_branch: else_branch })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParserError>{
        let begin = self.consume(TokenType::LeftParen, format!("Expect '(' after 'while'."));
        let condition: Expr = self.expression()?;
        let end = self.consume(TokenType::RightParen, format!("Expect ')' after condition."));
        let body = self.statement()?;
        return Ok(Stmt::While { condition: condition, body: Box::new(body) });
    }
    fn block(&mut self) -> Result<Vec<Stmt>, ParserError>{
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end(){
            statements.push(self.declaration()?);
            //println!("Death please");
        }
        self.consume(TokenType::RightBrace, format!("Expect '}}' after block."))?;
        return Ok(statements);
    }

    fn assignment(&mut self) -> Result<Expr, ParserError>{
        let expr: Expr = self.or()?;
        //println!("Howdy23");
        //println!("{}", self.peek().column);
        if self.matches(vec![TokenType::Equal]){
            //println!("Howdy222");
            let equals: Token = self.previous();
            let value: Expr = self.assignment()?;
            //println!("Howdy2");
            if let Expr::Variable { name, line, col } = expr.clone(){
                return Ok(Expr::Assign { 
                    name: name, 
                    line: line, 
                    column: col, 
                    value: Box::new(value) 
                })
            }
            else if let Expr::Get { object, name } = expr.clone(){
                //println!("HERE");
                return Ok(Expr::Set { 
                    object: object, 
                    name: name, 
                    value: Box::new(value) 
                })
            }
            return Err(ParserError { 
                message: format!("Invalid assignment target at line: {}, column: {}",
                equals.line, equals.column), 
                token_type: equals.token_type, 
                line: equals.line, 
                column: equals.column 
            })
        }
        return Ok(expr);    
    }

    fn or(&mut self) -> Result<Expr, ParserError>{
        let mut expr: Expr = self.and()?;
        while self.matches(vec![TokenType::Or]){
            let operator: Token = self.previous();
            let right: Expr = self.equality()?;
            expr = Expr::Logical { 
                left: Box::new(expr), 
                operator: operator,
                right: Box::new(right) 
            }
        }
        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr, ParserError>{
        let mut expr: Expr = self.equality()?;
        while self.matches(vec![TokenType::And]){
            let operator: Token = self.previous();
            let right: Expr = self.equality()?;
            expr = Expr::Logical { 
                left: Box::new(expr), 
                operator: operator,
                right: Box::new(right) 
            }
        }
        return Ok(expr);
    }

    fn expression(&mut self) -> Result<Expr, ParserError>{
            return self.assignment();
    }

    fn equality(&mut self) -> Result<Expr, ParserError>{
        let mut expr: Expr = self.comparison()?;
        while self.matches(vec![
            TokenType::BangEqual, 
            TokenType::EqualEqual
            ]){
            let operator: Token = self.previous();
            let right: Expr = self.comparison()?;
            let binary_expr: Result<Expr, ParserError> = Parser::binary_expression_match(expr, operator, right);
            match binary_expr{
                Ok(binary_expr) => expr = binary_expr,
                Err(err) => return Err(err),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError>{
        let mut expr: Expr = self.term()?;
        while self.matches(vec![
            TokenType::Greater, 
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
            ]){
            let operator: Token = self.previous();
            let right: Expr = self.term()?;
            let binary_expr: Result<Expr, ParserError> = Parser::binary_expression_match(expr, operator, right);
            match binary_expr{
                Ok(binary_expr) => expr = binary_expr,
                Err(err) => return Err(err),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError>{
        let mut expr: Expr = self.factor()?;
        while self.matches(vec![
            TokenType::Plus, 
            TokenType::Minus,
            ]){
            let operator: Token = self.previous();
            let right: Expr = self.factor()?;
            let binary_expr: Result<Expr, ParserError> = Parser::binary_expression_match(expr, operator, right);
            match binary_expr{
                Ok(binary_expr) => expr = binary_expr,
                Err(err) => return Err(err),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError>{
        let mut expr: Expr = self.unary()?;
        while self.matches(vec![
            TokenType::Slash, 
            TokenType::Star,
            ]){
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;
            let binary_expr: Result<Expr, ParserError> = Parser::binary_expression_match(expr, operator, right);
            match binary_expr{
                Ok(binary_expr) => expr = binary_expr,
                Err(err) => return Err(err),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError>{
        if self.matches(vec![
            TokenType::Bang,
            TokenType::Minus
        ]){
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;
            let unary_expr: Result<Expr, ParserError> = Parser::unary_expression_match(operator, right);
            match unary_expr{
                Ok(unary_expr) => return Ok(unary_expr),
                Err(err) => return Err(err),
            }
        }
        return self.call();
    }

    fn call(&mut self) -> Result<Expr, ParserError>{
        let mut expr: Expr = self.primary()?;
        loop{
            if self.matches(vec![TokenType::LeftParen]){
                expr = self.finish_call(expr)?;
            }
            else if self.matches(vec![TokenType::Dot]){
                let name = self.consume(TokenType::Identifier, format!("Expect property name after '.'"))?;
                expr = Expr::Get { object: Box::new(expr), name: String::from_utf8(name.lexeme).unwrap()  }
            }
            else{
                break;
            }
        }
        return Ok(expr);
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParserError>{
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(TokenType::RightParen){
            loop{
                if arguments.len() >= 255{
                    return Err(ParserError { 
                        message: format!("Cannot have more than 255 arguments"), 
                        token_type: TokenType::None, 
                        line: 0, 
                        column: 0 
                    })
                }
                arguments.push(self.expression()?);
                if !self.matches(vec![TokenType::Comma]){
                    break;
                }
            }
        }
        let paren: Token = self.consume(TokenType::RightParen, format!("Expect ')' after arguments"))?;
        return Ok(Expr::Call { 
            callee: Box::new(callee), 
            paren: paren, 
            arguments: Box::new(arguments) 
        })
    }

    fn primary(&mut self) -> Result<Expr, ParserError>{
        if self.matches(vec![TokenType::False]){
            return Ok(Expr::Literal { value: expr::LiteralType::False,})
        }
        if self.matches(vec![TokenType::True]){
            return Ok(Expr::Literal { value: expr::LiteralType::True,})
        }
        if self.matches(vec![TokenType::Nil]){
            return Ok(Expr::Literal { value: expr::LiteralType::Nil,})
        }
        if self.matches(vec![TokenType::Number]){
            match self.previous().literal{
                Some(scanner::Literal::Number(num)) => {
                    return Ok(Expr::Literal { value: expr::LiteralType::Number(num) })
                }
                Some(_) => panic!("Internal parser error when parsing number"),
                None => return Err(ParserError { 
                    message: format!("Missing literal when parsing number at line: {}, column: {}",
                    self.previous().line, self.previous().column),
                    token_type: TokenType::Number, 
                    line: self.previous().line, 
                    column: self.previous().column 
                })
            }
        }
        if self.matches(vec![TokenType::String]){
            match self.previous().literal{
                Some(scanner::Literal::String(str)) => {
                    return Ok(Expr::Literal { value: expr::LiteralType::String(str) })
                }
                Some(_) => panic!("Internal parser error when parsing string"),
                None => return Err(ParserError{ 
                    message: format!("Missing literal when parsing string at line: {}, column: {}",
                    self.previous().line, self.previous().column),
                    token_type: TokenType::String, 
                    line: self.previous().line, 
                    column: self.previous().column 
                })
            }
        }
        if self.matches(vec![TokenType::LeftParen]){
            let expr: Expr = self.expression()?;
            let correct_end = self.consume(TokenType::RightParen, "Expect ')' after expression.".to_string());
            match correct_end{
                Ok(token) => return Ok(Expr::Grouping { expression: Box::new(expr)}),
                Err(err) => return Err(err)
            }
        }
        if self.matches(vec![TokenType::This]) {
            return Ok(Expr::This { keyword: String::from_utf8(self.previous().lexeme).unwrap() })
        }
        if self.matches(vec![TokenType::Identifier]){
            //println!("IN HERE");
            match self.previous().literal{
                Some(Literal::Identifier(str)) => {
                    //println!("YUP");
                    return Ok(Expr::Variable { 
                        name: str.clone(), 
                        line: self.previous().line, 
                        col: self.previous().column 
                    })
                },
                Some(_) => panic!("Internal parser error when parsing Identifier"),
                None => panic!("Found no literal when parsing Identifier")
            }
        }
        if self.matches(vec![TokenType::Super]){
            let keyword: String = String::from_utf8(self.previous().lexeme).unwrap();
            self.consume(TokenType::Dot, format!("Expect '.' after 'super'"))?;
            let method: Token = self.consume(TokenType::Identifier, format!("Expect superclass method name"))?;
            return Ok(Expr::Super { keyword: keyword, method: String::from_utf8(method.lexeme).unwrap() })
        }
        Err(ParserError {
            message: format!("Expected expression at line: {}, column{}",
            self.peek().line, self.peek().column), 
            token_type: self.peek().token_type, 
            line: self.peek().line, 
            column: self.peek().column 
        })
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, ParserError>{
        if self.check(token_type) {
            return Ok(self.advance())
        }
        else{
            return Err(ParserError{
                message: format!("{} at line: {}, column: {}",
                message, self.peek().line, self.peek().column), 
                token_type: token_type,
                line: self.peek().line,
                column: self.peek().column
            })
        }
    }

    fn binary_expression_match(left_expr: Expr, op_token: Token, right_expr: Expr) -> Result<Expr, ParserError>{
        let binary_type: expr::BinaryOpType = expr::BinaryOpType::binary_match(op_token.token_type);
        if binary_type != expr::BinaryOpType::Error{
            Ok(Expr::Binary { 
                left: Box::new(left_expr), 
                operator: binary_type, 
                right: Box::new(right_expr), 
                line: op_token.line, 
                col: op_token.column 
            })
        }
        else{
            Err(ParserError {
                message: format!("Invalid Binary Operator at line: {}, column {}",
                op_token.line, op_token.column), 
                token_type: op_token.token_type, 
                line: op_token.line, 
                column: op_token.column 
            })
        }
    }

    fn unary_expression_match(op_token: Token, right_expr: Expr) -> Result<Expr, ParserError>{
        let unary_type: expr::UnaryOpType = expr::UnaryOpType::unary_match(op_token.token_type);
        if unary_type != expr::UnaryOpType::Error{
            Ok(Expr::Unary { 
                operator: unary_type, 
                right: Box::new(right_expr), 
                line: op_token.line, 
                col: op_token.column 
            })
        }
        else{
            Err(ParserError {
                message: format!("Invalid Unary Operator at line: {}, column {}",
                op_token.line, op_token.column), 
                token_type: op_token.token_type, 
                line: op_token.line, 
                column: op_token.column 
            })
        }
    }

    fn synchronize(&mut self) -> (){
        self.advance();
        while !self.is_at_end(){
            if self.previous().token_type == TokenType::Semicolon{
                return;
            }
            match self.peek().token_type{
                TokenType::Class => return,
                TokenType::Fun => return,
                TokenType::Var => return,
                TokenType::For => return,
                TokenType::If => return,
                TokenType::While => return,
                TokenType::Print => return,
                TokenType::Return => return,
                _ => ()
            }
            self.advance();
        }
    }

    fn peek(&self) -> Token{
        return self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token{
        return self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool{
        return self.peek().token_type == scanner::TokenType::Eof
    }

    fn matches(&mut self, types: Vec<TokenType>) -> bool{
        for token_type in types{
            if self.check(token_type){
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, passed_type: TokenType) ->bool{
        if self.is_at_end(){
            return false;
        }
        return (self.peek().token_type == passed_type);
    }

    fn advance(&mut self) -> Token{
        if !self.is_at_end(){
            self.current += 1;
        }
        return self.previous();
    }
}

pub struct ParserError{
    message: String,
    token_type: TokenType,
    line: usize,
    column: i64
}

impl ParserError{
    pub fn return_error(&self) -> String{
        return self.message.clone()
    }
}

pub fn parse_begin(in_tokens: Vec<Token>) -> Result<Vec<Stmt>, ParserError>{
    let mut parser: Parser = Parser{
        tokens: in_tokens,
        current: 0
    };
    let expr = parser.parse();
    match expr{
        Ok(expr) => return Ok(expr),
        Err(err) => return Err(err)
    };
    
}

//used for tests
pub fn stmt_ident(in_stmt: Stmt) -> String{
    match in_stmt {
            Stmt::Expr { expression: _ } => return "Expr".to_string(),
            Stmt::Print { expression: _ } => return "Print".to_string(),
            Stmt::Var { name: _ , line: _ , column: _ , initializer: _ } => return "Var".to_string(),
            Stmt::Block { statements: _ } => return "Block".to_string(),
            Stmt::If { condition: _ , then_branch: _ , else_branch: _ } => return "If".to_string(),
            Stmt::While { condition: _ , body: _ } => return "While".to_string(),
            Stmt::Function { name: _ , parameters: _ , body: _ } => return "Function".to_string(),
            Stmt::Return { keyword: _ , value: _ } => return "Return".to_string(),
            Stmt::Class { name: _ , superclass: _ , methods: _ } => return "Class".to_string(),
    }
}

pub fn expr_ident(in_expr: Expr) -> String{
    match in_expr {
        Expr::Binary { left:_, operator:_, right:_, line:_, col:_ } => return "Binary".to_string(),
        Expr::Grouping { expression:_ } => return "Grouping".to_string(),
        Expr::Literal { value:_ } => return "Literal".to_string(),
        Expr::Unary { operator:_, right:_, line:_, col:_ } => return "Unary".to_string(),
        Expr::Variable { name:_, line:_, col:_ } => return "Variable".to_string(),
        Expr::Assign { name:_, line:_, column:_, value:_ } => return "Assign".to_string(),
        Expr::Logical { left:_, operator:_, right:_ } => return "Logical".to_string(),
        Expr::Call { callee:_, paren:_, arguments:_ } => return "Call".to_string(),
        Expr::Get { object:_, name:_ } => return "Get".to_string(),
        Expr::Set { object:_, name:_, value:_ } => return "Set".to_string(),
        Expr::This { keyword:_ } => return "This".to_string(),
        Expr::Super { keyword:_, method:_ } => return "Super".to_string(),
        Expr::None => return "None".to_string(),
    }
}

pub fn tokentype_ident(in_token: TokenType) -> String{
    match in_token {
        TokenType::LeftParen => return "LeftParen".to_string(),
        TokenType::RightParen => return "RightParen".to_string(),
        TokenType::LeftBrace => return "LeftBrace".to_string(),
        TokenType::RightBrace => return "RightBrace".to_string(),
        TokenType::LeftBracket => return "LeftBracket".to_string(),
        TokenType::RightBracket => return "RightBracket".to_string(),
        TokenType::Comma => return "Comma".to_string(),
        TokenType::Dot => return "Dot".to_string(),
        TokenType::Minus => return "Minus".to_string(),
        TokenType::Plus => return "Plus".to_string(),
        TokenType::Semicolon => return "Semicolon".to_string(),
        TokenType::Colon => return "Colon".to_string(),
        TokenType::Slash => return "Slash".to_string(),
        TokenType::Star => return "Star".to_string(),
        TokenType::Mod => return "Mod".to_string(),
        TokenType::Bang => return "Bang".to_string(),
        TokenType::BangEqual => return "BangEqual".to_string(),
        TokenType::Equal => return "Equal".to_string(),
        TokenType::EqualEqual => return "EqualEqual".to_string(),
        TokenType::Greater => return "Greater".to_string(),
        TokenType::GreaterEqual => return "GreaterEqual".to_string(),
        TokenType::Less => return "Less".to_string(),
        TokenType::LessEqual => return "LessEqual".to_string(),
        TokenType::Identifier => return "Identifier".to_string(),
        TokenType::String => return "String".to_string(),
        TokenType::Number => return "Number".to_string(),
        TokenType::And => return "And".to_string(),
        TokenType::Class => return "Class".to_string(),
        TokenType::Else => return "Else".to_string(),
        TokenType::False => return "False".to_string(),
        TokenType::Fun => return "Fun".to_string(),
        TokenType::For => return "For".to_string(),
        TokenType::If => return "If".to_string(),
        TokenType::Nil => return "Nil".to_string(),
        TokenType::Or => return "Or".to_string(),
        TokenType::Print => return "Print".to_string(),
        TokenType::Return => return "Return".to_string(),
        TokenType::Super => return "Super".to_string(),
        TokenType::This => return "This".to_string(),
        TokenType::True => return "True".to_string(),
        TokenType::Var => return "Var".to_string(),
        TokenType::While => return "While".to_string(),
        TokenType::Lambda => return "Lamba".to_string(),
        TokenType::Eof => return "Eof".to_string(),
        TokenType::None => return "None".to_string(),
    }
}

#[cfg(test)]
mod tests{
    use crate::scanner::*;
    use crate::parser::*;

    //done
    #[test]
    fn test_expr() {
        let source = "x + 2;".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());
        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "Expr".to_string());

                if let Stmt::Expr { expression: expr_expr } = &stmt[0] {
                    //expression
                    let exprception = *expr_expr.clone();
                    assert_eq!(expr_ident(exprception), "Binary");
                }

            },
            Err(_) => {
                panic!("Test_expr match for 'stmt' is empty.");
            }
        }
    }

    //done
    #[test]
    fn test_print() {
        let source = "print \"Hello\";".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());
        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "Print".to_string());

                if let Stmt::Print { expression: print_expr } = &stmt[0] {
                    //expression
                    let expr_expr = *print_expr.clone();
                    assert_eq!(expr_ident(expr_expr), "Literal");
                }
            },

            Err(_) => {
                panic!("Test_print match for 'stmt' is empty.");
            }
        }
    }

    //done
    #[test]
    fn test_var() {
        let source = "var three = 3;".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());

        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "Var".to_string());

                if let Stmt::Var { name: var_name, line: var_line, column: var_col, initializer: var_init } = &stmt[0] {
                    //name
                    assert_eq!(var_name, "three");

                    //line
                    let assert_var: usize = 1;
                    assert_eq!(var_line, &assert_var);

                    //column
                    let assert_col: i64 = 8;
                    assert_eq!(var_col, &assert_col);

                    //initializer
                    match var_init.clone() {
                        Some(n) => {
                            assert_eq!(expr_ident(n), "Literal");
                        },
                        None => {
                            panic!("Test_block match for 'var_init' is empty.");
                        }
                    }
                }
            },

            Err(_) => {
                panic!("Test_var match for 'stmt' is empty.");
            }  
        }
    }

    //done
    #[test]
    fn test_block() {
        let source = "{var x = 3;}".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());

        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "Block");

                if let Stmt::Block { statements: block_stmts } = &stmt[0] {
                    assert_eq!(block_stmts.len(), 1);
                    assert_eq!(stmt_ident(block_stmts[0].clone()), "Var");
                }
            },
            Err(_) => {
                panic!("Test_block match for 'stmt' is empty.");
            }
        }
    }

    //done
    #[test]
    fn test_if() {
        let source = "if (x == 1) {print \"yes\";} else {x = 2;}".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());

        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "If".to_string());

                if let Stmt::If { condition: if_cond, then_branch: if_then, else_branch: if_else } = &stmt[0] {
                    //condition
                    let cond_expr = *if_cond.clone();
                    assert_eq!(expr_ident(cond_expr.clone()), "Binary");

                    //then branch
                    let then_stmt = *if_then.clone();
                    assert_eq!(stmt_ident(then_stmt.clone()), "Block");

                    //else branch
                    match if_else {
                        Some(p) => {
                            let else_stmt = *p.clone();
                            assert_eq!(stmt_ident(else_stmt.clone()), "Block");
                        },
                        None => {
                            panic!("Test_if match for 'if_else' is empty.");
                        }
                    }
                }
            },

            Err(_) => {
                panic!("Test_if match for 'stmt' is empty.");
            }
            
        }
    }

    //done
    #[test]
    fn test_while() {
        let source = "while (x < 2) {print \"yes\";}".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());

        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "While".to_string());

                if let Stmt::While { condition: while_cond, body: while_body } = &stmt[0]{
                    //condition
                    assert_eq!(expr_ident(while_cond.clone()), "Binary");
                
                    //body
                    let while_stmt = *while_body.clone();
                    assert_eq!(stmt_ident(while_stmt.clone()), "Block");
                }
            },

            Err(_) => {
            }
        }
    }
    //diff statement types, expr types, go at least 1 layer in on a few statements, and on all of them check the count

    //done
    #[test]
    fn test_return() {
        let source = "return true;".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());

        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "Return".to_string());

                if let Stmt::Return { keyword: ret_key , value: ret_value } = &stmt[0]{
                    //keyword
                    let tokentype_str = tokentype_ident( ret_key.token_type.clone());
                    assert_eq!(tokentype_str, "Return");
                    
                    //value
                    match ret_value {
                        Some(l) => {
                            assert_eq!(expr_ident(l.clone()), "Literal");
                        },
                        None => {
                            panic!("Test_return match for 'ret_value' is empty.");
                        },
                    }
                }
            },
            Err(_) => {
                panic!("Test_return match for 'stmt' has errored.");
            }
        }
    }

    //done
    #[test]
    fn test_function() {
        let source = "fun addTest(a,b) {if (a == 1) {return false;}} ".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());

        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "Function".to_string());

                if let Stmt::Function { name: fun_name , parameters: fun_param , body: fun_body } = &stmt[0]{
                    assert_eq!(fun_name, "addTest");

                    assert_eq!(fun_param.len(), 2);

                    let body_vect = *fun_body.clone();
                    assert_eq!(body_vect.len(), 1);
                    let body_vect_str = stmt_ident(body_vect[0].clone());
                    assert_eq!(body_vect_str, "If");
                }
            },
            Err(_) => {
                panic!("Test_function match for 'stmt' has errored.");
            }
        }
    }

    //done
    #[test]
    fn test_class() {
        let source = "class Test \n{\nex() \n{print a;}}".to_string();
        let mut scanner = Scanner::default();
        let tokens = scanner.scan_tokens(source);
        let stmt = parse_begin(tokens.clone());

        match stmt{
            Ok(stmt) => {
                let count = stmt.iter().count();
                assert_eq!(count, 1);
                assert_eq!(stmt.len(), 1);
                assert_eq!(stmt_ident(stmt[0].clone()), "Class".to_string());
                
                if let Stmt::Class { name:class_name, superclass:class_super, methods:class_mthd }= &stmt[0]{
                    //name
                    assert_eq!(class_name, "Test");

                    //superclass
                    match class_super {
                        Some(_) => {
                            panic!("Test_class match for 'class_super' is not empty.");
                        },
                        None => {},
                    }

                    //method
                    let vector_mthd = *class_mthd.clone();
                    assert_eq!(vector_mthd.len(), 1);
                    let vector_mthd_str = stmt_ident(vector_mthd[0].clone());
                    assert_eq!(vector_mthd_str, "Function");
                }
            },
            Err(_) => {
                panic!("Test_class match for 'stmt' has errored.");
            }
        }
    }

}
