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
        opp.insert(Token::Lparen, OpPrecedence::Call);
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

    fn next_token(&mut self) -> &Token {
        self.cur_tok = mem::replace(&mut self.next_tok, self.lexer.next_token());
        &self.cur_tok
    }

    pub fn parse_program(&mut self) -> Program {
        let mut prog = Program::new();
        loop {
            match self.cur_tok {
                Token::Eof => break,
                Token::Illegal => panic!("Illegal token {:?}", self.cur_tok),
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
            Token::Lbrace => self.parse_block(),
            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_let(&mut self) -> Statement {
        let ident = self.assert_ident();
        assert_eq!(self.next_token(), &Token::Assign);
        self.next_token();
        let rv = Statement::Let(ident.clone(), self.parse_expression(OpPrecedence::Lowest));
        if self.next_tok == Token::Semicolon {
            self.next_token();
        }
        rv
    }

    fn parse_ret(&mut self) -> Statement {
        self.next_token();
        let rv = Statement::Ret(self.parse_expression(OpPrecedence::Lowest));
        if self.next_tok == Token::Semicolon {
            self.next_token();
        }
        rv
    }

    fn parse_cond(&mut self) -> Expression {
        assert_eq!(self.next_token(), &Token::Lparen);
        self.next_token();
        let cond = self.parse_expression(OpPrecedence::Lowest);
        assert_eq!(self.next_token(), &Token::Rparen);
        self.next_token();
        let if_st = self.parse_statement();
        let else_st = {
            if self.next_tok == Token::Else {
                self.next_token();
                self.next_token();
                self.parse_statement()
            } else {
                Statement::BlockStatement(Vec::new())
            }
        };
        Expression::If(Box::new(cond), Box::new(if_st), Box::new(else_st))
    }

    fn parse_fn(&mut self) -> Expression {
        let mut params = Vec::new();
        assert_eq!(self.next_token(), &Token::Lparen);
        while self.next_tok != Token::Rparen {
            params.push(self.assert_ident());
            if self.next_tok == Token::Comma {
                self.next_token();
            }
        }
        self.next_token();
        self.next_token();
        Expression::FnDecl(params, Box::new(self.parse_statement()))
    }

    fn parse_array(&mut self) -> Expression {
        let mut elems = Vec::new();
        while self.next_tok != Token::Rbracket {
            self.next_token();
            elems.push(self.parse_expression(OpPrecedence::Lowest));
            if self.next_tok == Token::Comma {
                self.next_token();
            }
        }
        self.next_token();
        Expression::Array(elems)
    }

    fn parse_block(&mut self) -> Statement {
        let mut v = Vec::new();
        while self.next_token() != &Token::Rbrace {
            v.push(self.parse_statement());
        }
        Statement::BlockStatement(v)
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
            Token::True => Expression::True,
            Token::False => Expression::False,
            Token::If => self.parse_cond(),
            Token::Function => self.parse_fn(),
            Token::Lbracket => self.parse_array(),
            Token::String(s) => Expression::String(s),
            Token::Lparen => {
                self.next_token();
                let exp = self.parse_expression(OpPrecedence::Lowest);
                assert_eq!(self.next_token(), &Token::Rparen);
                exp
            },
            other => exprs::prefix_parser(&other).map(|prefix_fn| {
                self.next_token();
                prefix_fn(self.parse_expression(OpPrecedence::Prefix))
            }).unwrap_or_else(|| panic!("Prefix operator not found: {:?}", &other))
        };

        while self.next_tok != Token::Semicolon && op_prec < self.peek_precedence() {
            if self.next_tok != Token::Lparen {
                let infix = match exprs::infix_parser(&self.next_tok) {
                    None => break,
                    Some(i) => i,
                };
                self.next_token();
                let prec = self.cur_precedence();
                self.next_token();
                left = infix(left, self.parse_expression(prec));
            } else {
                left = self.parse_call(left);
            }
        }
        left
    }

    fn parse_call(&mut self, fn_exp: Expression) -> Expression {
        let mut params = Vec::new();
        self.next_token();
        while self.next_tok != Token::Rparen {
            self.next_token();
            params.push(self.parse_expression(OpPrecedence::Lowest));
            if self.next_tok == Token::Comma {
                self.next_token();
            }
        }
        self.next_token();
        Expression::Call(Box::new(fn_exp), params)
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
        let mut lexer = Lexer::new(String::from("x + 10;y < z; 1 + 2 * 3 / 4 - 5 == 0; -1-2-3"));
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
                    Box::new(Expression::Int(0))))),
            Box::new(Statement::ExprStatement(
                Expression::Minus(
                    Box::new(Expression::Minus(
                        Box::new(Expression::Neg(Box::new(Expression::Int(1)))),
                        Box::new(Expression::Int(2))
                    )),
                    Box::new(Expression::Int(3))
                )
            ))
        ]);
    }

    #[test]
    fn test_paren() {
        let mut lexer = Lexer::new(String::from("(x * (y + z)) == true"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::ExprStatement(Expression::Eq(Box::new(
                Expression::Mul(
                    Box::new(Expression::Ident(String::from("x"))),
                    Box::new(Expression::Plus(
                        Box::new(Expression::Ident(String::from("y"))),
                        Box::new(Expression::Ident(String::from("z"))),
                    ))
                )
            ), Box::new(Expression::True))))
        ]);
    }

    #[test]
    fn test_cond() {
        let mut lexer = Lexer::new(String::from("if (x > 0) {let x = 1; x + 1} else (1+2)*3"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::ExprStatement(Expression::If(
                Box::new(Expression::Gt(
                    Box::new(Expression::Ident(String::from("x"))),
                    Box::new(Expression::Int(0)))),
                Box::new(Statement::BlockStatement(vec![
                    Statement::Let(String::from("x"), Expression::Int(1)),
                    Statement::ExprStatement(Expression::Plus(
                        Box::new(Expression::Ident(String::from("x"))),
                        Box::new(Expression::Int(1)),
                    )),
                ])),
                Box::new(Statement::ExprStatement(Expression::Mul(
                    Box::new(Expression::Plus(
                        Box::new(Expression::Int(1)),
                        Box::new(Expression::Int(2)),
                    )),
                    Box::new(Expression::Int(3))))),
            )))
        ]);
    }

    #[test]
    fn test_only_if() {
        let mut lexer = Lexer::new(String::from("if (((0))) let x = (1);"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::ExprStatement(Expression::If(
                Box::new(Expression::Int(0)),
                Box::new(Statement::Let(String::from("x"), Expression::Int(1))),
                Box::new(Statement::BlockStatement(Vec::new()))
            )))
        ]);
    }

    #[test]
    fn test_if_precedence() {
        let mut lexer = Lexer::new(String::from("1 == -(if (0) 1)*2"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::ExprStatement(Expression::Eq(
                Box::new(Expression::Int(1)),
                Box::new(Expression::Mul(
                    Box::new(Expression::Neg(
                        Box::new(Expression::If(
                            Box::new(Expression::Int(0)),
                            Box::new(Statement::ExprStatement(Expression::Int(1))),
                            Box::new(Statement::BlockStatement(Vec::new()))
                        ))
                    )),
                    Box::new(Expression::Int(2)),
                ))
            )))
        ]);
    }

    #[test]
    fn test_fn_decl() {
        let mut lexer = Lexer::new(String::from("let x = fn() 1; let y = fn(a,b) { let x = 1; a+b }"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::Let(
                String::from("x"),
                Expression::FnDecl(Vec::new(), Box::new(Statement::ExprStatement(Expression::Int(1)))))),
            Box::new(Statement::Let(
                String::from("y"),
                Expression::FnDecl(
                    vec![String::from("a"), String::from("b")],
                    Box::new(Statement::BlockStatement(vec![
                        Statement::Let(String::from("x"), Expression::Int(1)),
                        Statement::ExprStatement(Expression::Plus(
                            Box::new(Expression::Ident(String::from("a"))),
                            Box::new(Expression::Ident(String::from("b"))),
                    ))]))))),
        ]);
    }

    #[test]
    fn test_fn_call() {
        let mut lexer = Lexer::new(String::from("func(); func1(1); func2(1,2);"));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::ExprStatement(Expression::Call(
                Box::new(Expression::Ident(String::from("func"))), vec![]))),
            Box::new(Statement::ExprStatement(Expression::Call(
                Box::new(Expression::Ident(String::from("func1"))),
                vec![Expression::Int(1)]))),
            Box::new(Statement::ExprStatement(Expression::Call(
                Box::new(Expression::Ident(String::from("func2"))),
                vec![Expression::Int(1), Expression::Int(2)]))),
        ]);
    }

    #[test]
    fn test_str() {
        let mut lexer = Lexer::new(String::from("let x = \"a b \" + \" c d \""));
        let mut parser = Parser::new(&mut lexer);
        assert_eq!(parser.parse_program().statements(), &vec![
            Box::new(Statement::Let(
                String::from("x"),
                Expression::Plus(
                    Box::new(Expression::String(String::from("a b "))),
                    Box::new(Expression::String(String::from(" c d ")))))),
        ]);
    }
}
