use crate::scanner::Token;
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

impl Parser{
    pub fn parse(&mut self){

    }
    fn expression(&mut self) -> Result<Expr, Error>{

    }
    fn equality(&mut self) -> Result<Expr, Error>{

    }
    fn comparison(&mut self) -> Result<Expr, Error>{
        
    }
    fn peek(&self) -> Token{
        return self.tokens[self.current]
    }
    fn current_token(&self) -> Token{
        return self.tokens[self.current - 1]
    }
    fn is_at_end(&self) -> bool{
        return self.peek().TokenType == Token::TokenType::Eof
    }
}

fn parse_begin(in_tokens: Vec<Token>){
    let mut parser: Parser = Parser{
        tokens: in_tokens,
        current: 0
    };
    //p.parse();
    
}
