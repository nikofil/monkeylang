use super::Parser;
use ::ast::Expression;
use ::ast::Expression::*;
use ::lexer::Token;
use std::collections::HashMap;

fn prefix_minus(e: Expression) -> Expression {
    Neg(Box::new(e))
}

fn prefix_not(e: Expression) -> Expression {
    Not(Box::new(e))
}

fn infix_plus(e1: Expression, e2: Expression) -> Expression {
    Plus(Box::new(e1), Box::new(e2))
}

pub fn get_prefix_fns() -> HashMap<Token, fn(Expression) -> Expression> {
    let mut rv: HashMap<Token, fn(Expression) -> Expression> = HashMap::new();
    rv.insert(Token::Minus, prefix_minus);
    rv.insert(Token::Not, prefix_not);
    rv
}

pub fn get_infix_fns() -> HashMap<Token, fn(Expression, Expression) -> Expression> {
    let mut rv: HashMap<Token, fn(Expression, Expression) -> Expression> = HashMap::new();
    rv.insert(Token::Plus, infix_plus);
    rv
}
