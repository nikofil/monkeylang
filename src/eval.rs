use lexer::Lexer;
use parser::Parser;
use ast::*;
use ast::Statement::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Null,
}

use eval::Value::*;

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Int(i) => f.write_str(&format!("{}", i)),
            Bool(b) => f.write_str(&format!("{}", b)),
            Null => f.write_str("null"),
        }
    }
}

pub struct State {
    state: HashMap<String, Value>,
}

impl State {
    pub fn new() -> State {
        State{ state: HashMap::new() }
    }

    fn set(&mut self, name: &String, value: Value) {
        self.state.insert(name.clone(), value);
    }

    fn get(&self, name: &String) -> Option<&Value> {
        self.state.get(name)
    }

    pub fn eval(&mut self, input: &str) -> Option<Value> {
        let mut lexer = Lexer::new(String::from(input));
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        program.eval(self)
    }
}

trait Eval {
    fn eval(&self, state: &mut State) -> Option<Value>;
}

impl Eval for Program {
    fn eval(&self, state: &mut State) -> Option<Value> {
        let mut rv = None;
        for st in self.statements() {
            rv = st.eval(state);
        }
        rv
    }
}

impl Eval for Statement {
    fn eval(&self, state: &mut State) -> Option<Value> {
        match self {
            Let(s, exp) => {
                let val = exp.eval(state).unwrap().clone();
                state.set(&s, val);
                None
            },
            Ret(exp) => exp.eval(state),
            BlockStatement(stmts) => {
                let mut rv = None;
                for st in stmts {
                    rv = st.eval(state);
                }
                rv
            },
            ExprStatement(exp) => exp.eval(state),
        }
    }
}

impl Eval for Expression {
    fn eval(&self, state: &mut State) -> Option<Value> {
        Some(match self {
            Expression::Int(i) => Int(*i),
            Expression::True => Bool(true),
            Expression::False => Bool(false),
            _ => unimplemented!()
        })
    }
}

pub fn eval(input: &str) -> Option<Value> {
    State::new().eval(input).map(|v| v.clone())
}

#[cfg(test)]
mod test {
    use super::eval;
    use super::Value::*;

    #[test]
    fn test_prims() {
        assert_eq!(eval("1").unwrap(), Int(1));
    }
}
