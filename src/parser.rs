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

declaration    → varDecl
               | statement ;

statement      → exprStmt
               | printStmt
               | block ;

block          → "{" declaration* "}" ;

exprStmt       → expression ";" ;
printStmt      → "print" expression ";" ;
varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
expression     → assignment ;
assignment     → IDENTIFIER "=" assignment
               | equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
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
        return self.statement();
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError>{
        let name = self.consume(TokenType::Identifier, format!("Expect variable name"))?;
        let init = if self.matches(vec![TokenType::Equal]) {
            println!("Howdy");
            Some(self.expression()?)
        }
        else{
            None
        };
        self.consume(TokenType::Semicolon, format!("Expect ';' after variable declaration"))?;
        return Ok(Stmt::Var { name: String::from_utf8(name.lexeme).unwrap(), line: name.line, column: name.column ,initializer: init });
    }

    fn statement(&mut self) -> Result<Stmt, ParserError>{
        if self.matches(vec![TokenType::Print]){
            return self.print_statement();
        }
        if self.matches(vec![TokenType::LeftBrace]){
            println!("Kill me");
            let statements: Vec<Stmt> = self.block()?;
            return Ok(Stmt::Block { statements: statements });
        }
        return self.expression_statement();
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError>{
        let value: Expr = self.expression()?;
        let correct_end: Result<Token, ParserError> = self.consume(TokenType::Semicolon, format!("Expect ';' after value"));
        match correct_end{
            Ok(token) => return Ok(Stmt::Print { expression: Box::new(value) }),
            Err(err) => return Err(err)
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

    fn block(&mut self) -> Result<Vec<Stmt>, ParserError>{
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end(){
            statements.push(self.declaration()?);
            println!("Death please");
        }
        self.consume(TokenType::RightBrace, format!("Expect '}}' after block."))?;
        return Ok(statements);
    }

    fn assignment(&mut self) -> Result<Expr, ParserError>{
        let expr: Expr = self.equality()?;
        println!("Howdy23");
        println!("{}", self.peek().column);
        if self.matches(vec![TokenType::Equal]){
            println!("Howdy222");
            let equals: Token = self.previous();
            let value: Expr = self.assignment()?;
            println!("Howdy2");
            if let Expr::Variable { name, line, col } = expr.clone(){
                return Ok(Expr::Assign { 
                    name: name, 
                    line: line, 
                    column: col, 
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
        println!("Freak off");
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
        return self.primary();
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
        if self.matches(vec![TokenType::Identifier]){
            match self.previous().literal{
                Some(Literal::Identifier(str)) => {
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
                message: format!("Mismatched tokens found at line: {}, column: {}",
                self.peek().line, self.peek().column), 
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

    // #[test]
    // fn simple_addition(){
    //     let input = "(3 + 4)".to_string();
    //     let mut test_scanner: Scanner = Scanner::default();
    //     let tokens: Vec<Token> = test_scanner.scan_tokens(input);
    //     let expr: Result<Expr, ParserError> = parse_begin(tokens);
    //     match expr{
    //         Ok(express) =>{
    //             if let Expr::Grouping { expression } = express{
    //                 if let Expr::Binary { left  , operator , right, line: _, col: _ } = *expression{
    //                     assert_eq!(*left, Expr::Literal { value: expr::LiteralType::Number(3.0)}, "Error parsing left hand expression");
    //                     assert_eq!(operator, BinaryOpType::Plus, "Error parsing operand");
    //                     assert_eq!(*right, Expr::Literal { value: expr::LiteralType::Number(4.0)}, "Error parsing right hand expression");
    //                 }
    //             } 
    //         }
    //         Err(err) => panic!("Parser Error In Grouping")
    //     }

    // }
}
