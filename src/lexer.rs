use std::collections::HashMap;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Token {
    Illegal,
    Eof,
    Ident(String),
    Int(i32),
    Assign,
    Plus,
    Comma,
    Semicolon,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Function,
    Let,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    read_pos: usize,
    ch: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        Lexer{ input: input.chars().collect::<Vec<char>>(), pos: 0, read_pos: 0, ch: None }
    }

    pub fn read_char(&mut self) {
        let nch = self.input.get(self.read_pos).map(|c| *c);
        self.ch = nch.map(|c| {
            self.pos = self.read_pos;
            self.read_pos += 1;
            c
        });
    }

    pub fn get_char(&self) -> Option<char> {
        self.ch
    }

    pub fn next_token(&mut self) -> Token {
        let mut keywords = HashMap::new();
        keywords.insert("let", Token::Let);
        keywords.insert("fn", Token::Function);
        while self.ch.map_or(false, |c| {
            c.is_whitespace()
        }) {
            self.read_char();
        }
        let ret = self.ch.map_or(Token::Eof, |c| {
            match c {
                '=' => Token::Assign,
                '+' => Token::Plus,
                ',' => Token::Comma,
                ';' => Token::Semicolon,
                '(' => Token::Lparen,
                ')' => Token::Rparen,
                '{' => Token::Lbrace,
                '}' => Token::Rbrace,
                c => {
                    if c.is_alphabetic() || c == '_' {
                        let ident = self.read_ident();
                        match keywords.get(ident.as_str()) {
                            None => Token::Ident(ident),
                            Some(c) => c.clone(),
                        }
                    } else {
                        Token::Illegal
                    }
                },
            }
        });
        self.read_char();
        ret
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.ch {
            if !(c.is_alphabetic() || c == '_') {
                break;
            }
            ident.push(c);
            self.read_char()
        }
        ident
    }
}

#[cfg(test)]
mod test {
    use super::{ Lexer, Token };

    #[test]
    fn test_read() {
        let input = String::from("hello");
        let mut lex = Lexer::new(input);
        lex.read_char();
        assert_eq!(lex.get_char().unwrap(), 'h');
        lex.read_char();
        assert_eq!(lex.get_char().unwrap(), 'e');
    }

    #[test]
    fn test_lexer() {
        let mut lex = Lexer::new(String::from("let a + b = ;"));
        let expected = vec![Token::Let, Token::Ident(String::from("a")),
            Token::Plus, Token::Ident(String::from("b")), Token::Assign, Token::Semicolon];
        let mut tokens = Vec::new();
        lex.read_char();
        let mut tok = lex.next_token();
        while tok != Token::Eof {
            tokens.push(tok.clone());
            tok = lex.next_token();
        }
        assert_eq!(tokens, expected);
    }
}
