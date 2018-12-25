use lexer::{ Lexer, Token };
use ast::*;
use std::mem;

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

    pub fn assert_ident(&mut self) -> String {
        match self.next_token().clone() {
            Token::Ident(s) => s,
            _ => panic!("Expected Ident, got {:?}", self.cur_tok),
        }
    }

    pub fn assert_int(&mut self) -> i32 {
        match self.next_token().clone() {
            Token::Int(i) => i,
            _ => panic!("Expected Int, got {:?}", self.cur_tok),
        }
    }

    pub fn next_token(&mut self) -> &Token {
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

    pub fn parse_statement(&mut self) -> Statement {
        match self.cur_tok {
            Token::Let => self.parse_let(),
            _ => panic!("{:?}", self.cur_tok),
        }
    }

    pub fn parse_let(&mut self) -> Statement {
        let ident = self.assert_ident();
        assert_eq!(self.next_token(), &Token::Assign);
        let rv = Statement::Let(ident.clone(), self.parse_expression());
        assert_eq!(self.next_token(), &Token::Semicolon);
        rv
    }

    pub fn parse_expression(&mut self) -> Expression {
        match self.next_token() {
            Token::Int(i) => Expression::Int(*i),
            other => panic!("Invalid expression: {:?}", other)
        }
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
