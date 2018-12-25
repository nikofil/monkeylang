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
    Eq,
    Not,
    Ne,
    Minus,
    Div,
    Mul,
    Lt,
    Gt,
    True,
    False,
    If,
    Else,
    Ret,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    read_pos: usize,
    ch: Option<char>,
    keywords: HashMap<&'static str, Token>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let mut keywords = HashMap::new();
        keywords.insert("let", Token::Let);
        keywords.insert("fn", Token::Function);
        keywords.insert("true", Token::True);
        keywords.insert("false", Token::False);
        keywords.insert("if", Token::If);
        keywords.insert("else", Token::Else);
        keywords.insert("return", Token::Ret);
        Lexer{ input: input.chars().collect::<Vec<char>>(), pos: 0, read_pos: 0, ch: None, keywords }
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

    pub fn peek_char(&self) -> Option<char> {
        self.input.get(self.read_pos).map(|c| *c)
    }

    pub fn next_token(&mut self) -> Token {
        while self.ch.map_or(false, |c| {
            c.is_whitespace()
        }) {
            self.read_char();
        }
        let mut read_next = true;
        let ret = self.ch.map_or(Token::Eof, |c| {
            match c {
                '=' => {
                    if let Some('=') = self.peek_char() {
                        self.read_char();
                        Token::Eq
                    } else {
                        Token::Assign
                    }
                },
                '+' => Token::Plus,
                ',' => Token::Comma,
                ';' => Token::Semicolon,
                '(' => Token::Lparen,
                ')' => Token::Rparen,
                '{' => Token::Lbrace,
                '}' => Token::Rbrace,
                '!' => {
                    if let Some('=') = self.peek_char() {
                        self.read_char();
                        Token::Ne
                    } else {
                        Token::Not
                    }
                },
                '-' => Token::Minus,
                '/' => Token::Div,
                '*' => Token::Mul,
                '<' => Token::Lt,
                '>' => Token::Gt,
                c => {
                    if c.is_alphabetic() || c == '_' {
                        let ident = self.read_ident();
                        read_next = false;
                        match self.keywords.get(ident.as_str()) {
                            None => Token::Ident(ident),
                            Some(c) => c.clone(),
                        }
                    } else if c.is_numeric() {
                        read_next = false;
                        Token::Int(self.read_num())
                    } else {
                        Token::Illegal
                    }
                },
            }
        });
        if read_next {
            self.read_char();
        }
        ret
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.ch {
            if !(c.is_alphabetic() || c == '_') {
                break;
            }
            ident.push(c);
            self.read_char();
        }
        ident
    }

    fn read_num(&mut self) -> i32 {
        let mut num_s = String::new();
        while let Some(c) = self.ch {
            if !(c.is_numeric()) {
                break;
            }
            num_s.push(c);
            self.read_char();
        }
        num_s.parse::<i32>().unwrap()
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let tok = self.next_token();
        match tok {
            Token::Eof | Token::Illegal => None,
            _ => Some(tok),
        }
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
        let mut lex = Lexer::new(String::from("let a+b = 32;"));
        let expected = vec![Token::Let, Token::Ident(String::from("a")),
            Token::Plus, Token::Ident(String::from("b")), Token::Assign,
            Token::Int(32), Token::Semicolon];
        let mut tokens = Vec::new();
        lex.read_char();
        let mut tok = lex.next_token();
        while tok != Token::Eof {
            tokens.push(tok.clone());
            tok = lex.next_token();
        }
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_lexer_2() {
        let input = "let five = 5;
                           let ten = 10;
                           let add = fn(x, y) {
                           x + y;
                           };
                           let result = add(five, ten);
                           !-/*5;
                           5 < 10 > 5;
                           if (5 < 10) {
                           return true;
                           } else {
                           return false;
                           }
                           10 == 10;
                           10 != 9;";
        let mut lex = Lexer::new(String::from(input));
        lex.read_char();
        let tokens = lex.collect::<Vec<Token>>();
        assert_eq!(tokens.len(), 73);
        assert!(!tokens.iter().any(|t| *t == Token::Illegal));
    }
}
