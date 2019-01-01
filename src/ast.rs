#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Int(i32),
    Plus(Box<Expression>, Box<Expression>),
    Minus(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Eq(Box<Expression>, Box<Expression>),
    Ne(Box<Expression>, Box<Expression>),
    Lt(Box<Expression>, Box<Expression>),
    Gt(Box<Expression>, Box<Expression>),
    Ident(String),
    Neg(Box<Expression>),
    Not(Box<Expression>),
    True,
    False,
    If(Box<Expression>, Box<Statement>, Box<Statement>),
    FnDecl(Vec<String>, Box<Statement>),
    Call(Box<Expression>, Vec<Expression>),
    String(String),
    Array(Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let(String, Expression),
    Ret(Expression),
    BlockStatement(Vec<Statement>),
    ExprStatement(Expression),
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

    pub fn statements(&self) -> &Vec<Box<Statement>> {
        &self.statements
    }
}
