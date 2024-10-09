use crate::expr::*;
use crate::scanner::Token;

pub trait AstVisitor<R> {
    fn visit_binary(&mut self, visitor: Expr) -> R;
    fn visit_grouping(&mut self, visitor: Expr) -> R;
    fn visit_literal(&mut self, visitor: Expr) -> R;
    fn visit_unary(&mut self, visitor: Expr) -> R;
}
pub trait Accept<R> {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R;
}
impl<R> Accept<R> for Expr {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Expr::Empty => {
                panic!("Cannot visit empty");
            }
            Expr::Binary(x) => visitor.visit_binary(x),
            Expr::Grouping(x) => visitor.visit_grouping(x),
            Expr::Literal(x) => visitor.visit_literal(x),
            Expr::Unary(x) => visitor.visit_unary(x),
        }
    }
}
impl<R> Accept<R> for Binary {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        visitor.visit_binary(self)
    }
}
impl<R> Accept<R> for Grouping {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        visitor.visit_grouping(self)
    }
}
impl<R> Accept<R> for Literal {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        visitor.visit_literal(self)
    }
}
impl<R> Accept<R> for Unary {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        visitor.visit_unary(self)
    }
}
