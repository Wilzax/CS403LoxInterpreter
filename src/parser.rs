use crate::scanner;
use crate::scanner::{Token, TokenType};
use crate::expr; //Did not want to type scanner::Token 8000 times
use crate::expr::Expr;

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
expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" ;
*/

impl Parser{
    pub fn parse(&mut self) -> Result<Expr, ParserError>{
        let expr = self.expression();
        match expr{
            Ok(expr) => return Ok(expr),
            Err(err) => return Err(err),
        }
    }

    fn expression(&mut self) -> Result<Expr, ParserError>{
            return self.equality();
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
            TokenType::Greater, 
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
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
            TokenType::Greater, 
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
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
                None => return Err(ParserError::MissingLiteralWhenParsingNumber { 
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
                None => return Err(ParserError::MissingLiteralWhenParsingString { 
                    token_type: TokenType::String, 
                    line: self.previous().line, 
                    column: self.previous().column 
                })
            }
        }
        if self.matches(vec![TokenType::LeftParen]){
            let mut expr: Expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.".to_string());
            return Ok(Expr::Grouping { expression: Box::new(expr)})
        }
        Err(ParserError::ExpectedExpression { 
            token_type: self.peek().token_type, 
            line: self.peek().line, 
            column: self.peek().column ,
            error_message: "Expected Expression at line: ".to_string() + &self.peek().line.to_string() + ", column: " + &self.peek().column.to_string(),
        })
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, ParserError>{
        if self.check(token_type) {
            return Ok(self.advance())
        }
        else{
            return Err(ParserError::MismatchedTokens { 
                expected: token_type, 
                found: self.peek().token_type, 
                error_line: self.peek().line,
                error_column: self.peek().column,
                error_message:  message + "Error located at line: " + &self.peek().line.to_string() + ", column: " + &self.peek().column.to_string(),
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
            Err(ParserError::InvalidBinaryOperator { 
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
            Err(ParserError::InvalidUnaryOperator { 
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

pub enum ParserError{
    InvalidBinaryOperator{
        token_type: TokenType,
        line: usize,
        column: i64,
    },
    InvalidUnaryOperator{
        token_type: TokenType,
        line: usize,
        column: i64,
    },
    MissingLiteralWhenParsingNumber{
        token_type: TokenType,
        line: usize,
        column: i64,
    },
    MissingLiteralWhenParsingString{
        token_type: TokenType,
        line: usize,
        column: i64,
    },
    ExpectedExpression{
        token_type: TokenType,
        line: usize,
        column: i64,
        error_message: String,
    },
    MismatchedTokens{
        expected: TokenType,
        found: TokenType,
        error_line: usize,
        error_column: i64,
        error_message: String,
    },
}

pub fn parse_begin(in_tokens: Vec<Token>){
    let mut parser: Parser = Parser{
        tokens: in_tokens,
        current: 0
    };
    parser.parse();
    
}
