use ::ast::Expression;
use ::ast::Expression::*;
use ::lexer::Token;

pub fn prefix_parser(tok: &Token) -> Option<Box<Fn(Expression) -> Expression>> {
    Some(match tok {
        &Token::Not => Box::new(|exp| Not(Box::new(exp))),
        &Token::Minus => Box::new(|exp| Neg(Box::new(exp))),
        _ => return None
    })
}

pub fn infix_parser(tok: &Token) -> Option<Box<Fn(Expression, Expression) -> Expression>> {
    Some(match tok {
        &Token::Plus => Box::new(|left, right| Plus(Box::new(left), Box::new(right))),
        &Token::Minus => Box::new(|left, right| Minus(Box::new(left), Box::new(right))),
        &Token::Div => Box::new(|left, right| Div(Box::new(left), Box::new(right))),
        &Token::Mul => Box::new(|left, right| Mul(Box::new(left), Box::new(right))),
        &Token::Eq => Box::new(|left, right| Eq(Box::new(left), Box::new(right))),
        &Token::Ne => Box::new(|left, right| Ne(Box::new(left), Box::new(right))),
        &Token::Lt => Box::new(|left, right| Lt(Box::new(left), Box::new(right))),
        &Token::Gt => Box::new(|left, right| Gt(Box::new(left), Box::new(right))),
        _ => return None
    })
}
