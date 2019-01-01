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
    Str(String),
    FnDecl(Vec<String>, Box<Statement>),
    RetVal(Box<Value>),
    Null,
}

use eval::Value::*;

impl Value {
    fn unret(self) -> Value {
        match self {
            RetVal(v) => *v,
            other => other,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Int(i) => f.write_str(&format!("{}", i)),
            Bool(b) => f.write_str(&format!("{}", b)),
            Str(s) => f.write_str(&format!("{}", s)),
            FnDecl(pars, _stmt) => f.write_str(&format!("fn({})", pars.join(", "))),
            RetVal(v) => v.fmt(f),
            Null => f.write_str("null"),
        }
    }
}

#[derive(Clone)]
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
                let val = exp.eval(state).unwrap().unret();
                state.set(&s, val);
                None
            },
            Ret(exp) => exp.eval(state).map(|v| RetVal(Box::new(v.unret()))),
            BlockStatement(stmts) => {
                let mut val = None;
                for st in stmts {
                    val = st.eval(state);
                    if let Some(RetVal(_)) = val {
                        break;
                    }
                }
                val
            },
            ExprStatement(exp) => exp.eval(state).map(|v| v),
        }
    }
}

fn math_op(l: Option<Value>, r: Option<Value>, op: &Fn(i32, i32) -> i32) -> Value {
    match (l, r) {
        (Some(Int(lv)), Some(Int(rv))) => Int(op(lv, rv)),
        _ => Null,
    }
}

fn bool_op(l: Option<Value>, r: Option<Value>, op: &Fn(i32, i32) -> bool) -> Value {
    match (l, r) {
        (Some(Int(lv)), Some(Int(rv))) => Bool(op(lv, rv)),
        _ => Null,
    }
}

impl Eval for Expression {
    fn eval(&self, state: &mut State) -> Option<Value> {
        Some(match self {
            Expression::Int(i) => Int(*i),
            Expression::True => Bool(true),
            Expression::False => Bool(false),
            Expression::Plus(l, r) => {
                let lev = l.eval(state);
                let rev = r.eval(state);
                match (lev, rev) {
                    (Some(Str(s)), rev) => Str(format!("{}{}", s, rev.unwrap_or(Null))),
                    (lev, Some(Str(s))) => Str(format!("{}{}", lev.unwrap_or(Null), s)),
                    (lev, rev) => math_op(lev, rev, &|l, r| l + r),
                }
            },
            Expression::Minus(l, r) => math_op(l.eval(state), r.eval(state), &|l, r| l - r),
            Expression::Div(l, r) => math_op(l.eval(state), r.eval(state), &|l, r| l / r),
            Expression::Mul(l, r) => math_op(l.eval(state), r.eval(state), &|l, r| l * r),
            Expression::Eq(l, r) => bool_op(l.eval(state), r.eval(state), &|l, r| l == r),
            Expression::Ne(l, r) => bool_op(l.eval(state), r.eval(state), &|l, r| l != r),
            Expression::Lt(l, r) => bool_op(l.eval(state), r.eval(state), &|l, r| l < r),
            Expression::Gt(l, r) => bool_op(l.eval(state), r.eval(state), &|l, r| l > r),
            Expression::Ident(id) => state.get(&id).unwrap_or(&Null).clone(),
            Expression::String(s) => Value::Str(s.clone()),
            Expression::Neg(n) => if let Some(Int(i)) = n.eval(state) { Int(-i) } else { Null },
            Expression::Not(n) => if let Some(Bool(b)) = n.eval(state) { Bool(!b) } else { Null },
            Expression::If(cond, ifb, elb) => {
                let cond_val = cond.eval(state);
                match cond_val {
                    Some(Bool(true)) => ifb.eval(state).unwrap_or(Null),
                    Some(Bool(false)) => elb.eval(state).unwrap_or(Null),
                    _ => Null,
                }
            },
            Expression::FnDecl(pars, stmt) => FnDecl(pars.clone(), stmt.clone()),
            Expression::Call(func, actual) => {
                if let Some(FnDecl(formal, stmt)) = func.eval(state) {
                    let mut fn_state = state.clone();
                    for i in 0..std::cmp::min(actual.len(), formal.len()) {
                        fn_state.set(&formal[i], actual[i].eval(state).unwrap_or(Null));
                    }
                    stmt.eval(&mut fn_state).unwrap_or(Null).unret()
                } else {
                    Null
                }
            },
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
        assert_eq!(eval("true").unwrap(), Bool(true));
    }

    #[test]
    fn test_call() {
        assert_eq!(eval("let abs = fn (x) { if (x > 0) { return x; } return -x; }; abs(1) + abs(-1);").unwrap(), Int(2));
    }

    #[test]
    fn test_recursive() {
        assert_eq!(eval("let fib = fn (x) { if (x < 2) { return x; } else { fib(x-1) + fib(x-2); } }; fib(10);").unwrap(), Int(55));
    }

    #[test]
    fn test_stack() {
        assert_eq!(eval("let x = 10; let f = fn (x) { let x = 2*x+1; x }; let y = f(x); y + x;").unwrap(), Int(31));
    }

    #[test]
    fn test_str() {
        assert_eq!(eval("let a = \" hello \"; let b = \"world \"; a + b + 1").unwrap(), Str(String::from(" hello world 1")));
    }

    #[test]
    fn test_higher_order() {
        assert_eq!(eval("let twice = fn (f, x) f(f(x)); twice(fn(x) x*2, 10)").unwrap(), Int(40));
    }
}
