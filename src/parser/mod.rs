mod exprs;

use lexer::{ Lexer, Token };
use ast::*;
use std::mem;
use std::collections::HashMap;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Debug)]
enum OpPrecedence {
    Lowest,
    Eq,
    LtGt,
    Sum,
    Prod,
    Prefix,
    Call,
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer,
    cur_tok: Token,
    next_tok: Token,
}

lazy_static! {
    static ref OP_PRECEDENCE: HashMap<Token, OpPrecedence> = {
        let mut opp = HashMap::new();
        opp.insert(Token::Eq, OpPrecedence::Eq);
        opp.insert(Token::Ne, OpPrecedence::Eq);
        opp.insert(Token::Lt, OpPrecedence::LtGt);
        opp.insert(Token::Gt, OpPrecedence::LtGt);
        opp.insert(Token::Plus, OpPrecedence::Sum);
        opp.insert(Token::Minus, OpPrecedence::Sum);
        opp.insert(Token::Mul, OpPrecedence::Prod);
        opp.insert(Token::Div, OpPrecedence::Prod);
        opp
    };
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &mut Lexer) -> Parser {
        lexer.read_char();
        let cur_tok = lexer.next_token();
        let next_tok = lexer.next_token();
        Parser{ lexer, cur_tok, next_tok }
    }

    fn assert_ident(&mut self) -> String {
        match self.next_token().clone() {
            Token::Ident(s) => s,
            _ => panic!("Expected Ident, got {:?}", self.cur_tok),
        }
    }

    fn assert_int(&mut self) -> i32 {
        match self.next_token().clone() {
            Token::Int(i) => i,
            _ => panic!("Expected Int, got {:?}", self.cur_tok),
        }
    }

    fn next_token(&mut self) -> &Token {
        self.cur_tok = mem::replace(&mut self.next_tok, self.lexer.next_token());
        &self.cur_tok
    }

    pub fn parse_program(&mut self) -> Program {
        let mut prog = Program::new();
        loop {
            match self.cur_tok {
                Token::Eof => break,
                Token::Illegal => panic!("Illegal token"),
                _ => prog.push(self.parse_statement()),
            }
            self.next_token();
        }
        prog
    }

    fn parse_statement(&mut self) -> Statement {
        match self.cur_tok {
            Token::Let => self.parse_let(),
            Token::Ret => self.parse_ret(),
            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_let(&mut self) -> Statement {
        let ident = self.assert_ident();
        assert_eq!(self.next_token(), &Token::Assign);
        self.next_token();
        let rv = Statement::Let(ident.clone(), self.parse_expression(OpPrecedence::Lowest));
        assert_eq!(self.next_token(), &Token::Semicolon);
        rv
    }

    fn parse_ret(&mut self) -> Statement {
        self.next_token();
        let rv = Statement::Ret(self.parse_expression(OpPrecedence::Lowest));
        assert_eq!(self.next_token(), &Token::Semicolon);
        rv
    }

    fn cur_precedence(&self) -> OpPrecedence {
        OP_PRECEDENCE.get(&self.cur_tok).unwrap_or(&OpPrecedence::Lowest).clone()
    }

    fn peek_precedence(&self) -> OpPrecedence {
        OP_PRECEDENCE.get(&self.next_tok).unwrap_or(&OpPrecedence::Lowest).clone()
    }

    fn parse_expression(&mut self, op_prec: OpPrecedence) -> Expression {
        let mut left = match self.cur_tok.clone() {
            Token::Int(i) => Expression::Int(i),
            Token::Ident(s) => Expression::Ident(s),
            other => exprs::prefix_parser(&other).map(|prefix_fn| {
                self.next_token();
                prefix_fn(self.parse_expression(OpPrecedence::Prefix))
            }).unwrap_or_else(|| panic!("Prefix operator not found: {:?}", &other))
        };
        println!("left {:?} {:?}", &left, &op_prec);

        while &self.next_tok != &Token::Semicolon && op_prec < self.peek_precedence() {
            let infix = match exprs::infix_parser(&self.next_tok) {
                None => break,
                Some(i) => i,
            };
            self.next_token();
            let prec = self.cur_precedence();
            self.next_token();
            println!("next {:?}", &self.cur_tok);
            left = infix(left, self.parse_expression(prec));
        }
        println!("ret {:?}", &left);
        left
    }

    fn parse_expression_stmt(&mut self) -> Statement {
        let rv = Statement::ExprStatement(self.parse_expression(OpPrecedence::Lowest));
        if self.next_tok == Token::Semicolon {
            self.next_token();
        }
        rv
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_let() {
        let mut lexer = Lexer::new(String::from("let x = 10;let y=11;"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::Let(String::from("x"), Expression::Int(10))),
            Box::new(Statement::Let(String::from("y"), Expression::Int(11)))
        ]);
    }

    #[test]
    fn test_ret() {
        let mut lexer = Lexer::new(String::from("return x; return 1;"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::Ret(Expression::Ident(String::from("x")))),
            Box::new(Statement::Ret(Expression::Int(1)))
        ]);
    }

    #[test]
    fn test_prefix_stmts() {
        let mut lexer = Lexer::new(String::from("x; 10 ; -1;"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::ExprStatement(
                Expression::Ident(String::from("x")))),
            Box::new(Statement::ExprStatement(
                Expression::Int(10))),
            Box::new(Statement::ExprStatement(
                Expression::Neg(Box::new(Expression::Int(1)))))
        ]);
    }

    #[test]
    fn test_infix_stmts() {
        let mut lexer = Lexer::new(String::from("x + 10;y < z; 1 + 2 * 3 / 4 - 5 == 0"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::ExprStatement(
                Expression::Plus(
                    Box::new(Expression::Ident(String::from("x"))),
                    Box::new(Expression::Int(10))))),
            Box::new(Statement::ExprStatement(
                Expression::Lt(
                    Box::new(Expression::Ident(String::from("y"))),
                    Box::new(Expression::Ident(String::from("z")))))),
            Box::new(Statement::ExprStatement(
                Expression::Eq(
                    Box::new(Expression::Minus(
                        Box::new(Expression::Plus(
                            Box::new(Expression::Int(1)),
                            Box::new(Expression::Div(
                                Box::new(Expression::Mul(
                                    Box::new(Expression::Int(2)),
                                    Box::new(Expression::Int(3))
                                )),
                                Box::new(Expression::Int(4)))),
                        )),
                        Box::new(Expression::Int(5))
                    )),
                    Box::new(Expression::Int(0)))))
        ]);
    }
}
