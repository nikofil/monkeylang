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
        println!("prog {:?}", prog);
        prog
    }

    pub fn parse_statement(&mut self) -> Statement {
        match self.cur_tok {
            Token::Let => self.parse_let(),
            _ => panic!("{:?}", self.cur_tok),
        }
    }

    pub fn parse_let(&mut self) -> Statement {
        let next_tok = self.next_token().clone();
        if let Token::Ident(ident) = next_tok {
            let rv = match self.next_token() {
                Token::Assign => {
                    Statement::Let(ident.clone(), self.parse_expression())
                },
                _ => panic!("Assign not found"),
            };
            assert_eq!(self.next_token(), &Token::Semicolon);
            rv
        } else {
            panic!("Identifier not found");
        }
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
        let mut lexer = Lexer::new(String::from("let x = 10;"));
        let mut parser = Parser::new(&mut lexer);
        parser.parse_program();
    }
}
