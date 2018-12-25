use lexer::Token;

#[derive(Debug)]
pub enum Expression {
    Int(i32),
}

#[derive(Debug)]
pub enum Statement {
    Let(String, Expression),
}

#[derive(Debug)]
pub struct Program {
    statements: Vec<Box<Statement>>,
}

impl Program {
    pub fn new() -> Program {
        Program{ statements: Vec::new() }
    }
    pub fn push(&mut self, statement: Statement) {
        self.statements.push(Box::new(statement));
    }
}
