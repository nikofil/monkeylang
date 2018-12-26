use lexer::{ Lexer, Token };
use ast::*;
use std::mem;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
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
        let rv = Statement::Let(ident.clone(), self.parse_expression(OpPrecedence::Lowest));
        assert_eq!(self.next_token(), &Token::Semicolon);
        rv
    }

    fn parse_ret(&mut self) -> Statement {
        let rv = Statement::Ret(self.parse_expression(OpPrecedence::Lowest));
        assert_eq!(self.next_token(), &Token::Semicolon);
        rv
    }

    fn parse_expression(&mut self, op_prec: OpPrecedence) -> Expression {
        match self.next_token() {
            Token::Int(i) => Expression::Int(*i),
            other => panic!("Invalid expression: {:?}", other)
        }
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
}
