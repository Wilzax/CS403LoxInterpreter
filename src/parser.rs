use crate::interpreter::Value;
use crate::scanner::{self, Literal};
use crate::scanner::{Scanner, Token, TokenType};
use crate::expr; //Did not want to type scanner::Token 8000 times
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
            //println!("Kill me");
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
        if self.matches(vec![TokenType::This]){
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

#[cfg(test)]
mod tests{
    use expr::BinaryOpType;

    use super::*;


}
