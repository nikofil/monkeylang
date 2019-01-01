use lexer::Lexer;
use parser::Parser;
use ast::*;
use ast::Statement::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Write;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Str(String),
    FnDecl(Vec<String>, Box<Statement>),
    FnBuiltin(String, Box<fn(Vec<Option<Value>>) -> (Box<Value>, Option<String>)>),
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
            FnBuiltin(ident, _) => f.write_str(&format!("builtin {}", ident)),
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
        let mut state = HashMap::new();
        state.insert(String::from("len"), Value::FnBuiltin(String::from("len"), Box::new(|v| {
            (Box::new(match v.get(0) {
                Some(Some(Str(s))) => Int(s.len() as i32),
                _ => Null,
            }), None)
        })));
        state.insert(String::from("print"), Value::FnBuiltin(String::from("print"), Box::new(|v| {
            let mut out = String::new();
            v.iter().for_each(|v| {
                if let Some(val) = v {
                    out.push_str(&format!("{}", val));
                }
            });
            (Box::new(Null), Some(out))
        })));
        state.insert(String::from("println"), Value::FnBuiltin(String::from("print"), Box::new(|v| {
            let mut out = String::new();
            v.iter().for_each(|v| {
                if let Some(val) = v {
                    out.push_str(&format!("{}\n", val));
                }
            });
            (Box::new(Null), Some(out))
        })));
        State{ state }
    }

    fn set(&mut self, name: &String, value: Value) {
        self.state.insert(name.clone(), value);
    }

    fn get(&self, name: &String) -> Option<&Value> {
        self.state.get(name)
    }

    pub fn eval(&mut self, input: &str, writer: &mut Write) -> Option<Value> {
        let mut lexer = Lexer::new(String::from(input));
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        program.eval(self, writer)
    }
}

trait Eval {
    fn eval(&self, state: &mut State, writer: &mut Write) -> Option<Value>;
}

impl Eval for Program {
    fn eval(&self, state: &mut State, writer: &mut Write) -> Option<Value> {
        let mut rv = None;
        for st in self.statements() {
            rv = st.eval(state, writer);
        }
        rv
    }
}

impl Eval for Statement {
    fn eval(&self, state: &mut State, writer: &mut Write) -> Option<Value> {
        match self {
            Let(s, exp) => {
                let val = exp.eval(state, writer).unwrap().unret();
                state.set(&s, val);
                None
            },
            Ret(exp) => exp.eval(state, writer).map(|v| RetVal(Box::new(v.unret()))),
            BlockStatement(stmts) => {
                let mut val = None;
                for st in stmts {
                    val = st.eval(state, writer);
                    if let Some(RetVal(_)) = val {
                        break;
                    }
                }
                val
            },
            ExprStatement(exp) => exp.eval(state, writer).map(|v| v),
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
    fn eval(&self, state: &mut State, writer: &mut Write) -> Option<Value> {
        Some(match self {
            Expression::Int(i) => Int(*i),
            Expression::True => Bool(true),
            Expression::False => Bool(false),
            Expression::Plus(l, r) => {
                let lev = l.eval(state, writer);
                let rev = r.eval(state, writer);
                match (lev, rev) {
                    (Some(Str(s)), rev) => Str(format!("{}{}", s, rev.unwrap_or(Null))),
                    (lev, Some(Str(s))) => Str(format!("{}{}", lev.unwrap_or(Null), s)),
                    (lev, rev) => math_op(lev, rev, &|l, r| l + r),
                }
            },
            Expression::Minus(l, r) => math_op(l.eval(state, writer), r.eval(state, writer), &|l, r| l - r),
            Expression::Div(l, r) => math_op(l.eval(state, writer), r.eval(state, writer), &|l, r| l / r),
            Expression::Mul(l, r) => math_op(l.eval(state, writer), r.eval(state, writer), &|l, r| l * r),
            Expression::Eq(l, r) => bool_op(l.eval(state, writer), r.eval(state, writer), &|l, r| l == r),
            Expression::Ne(l, r) => bool_op(l.eval(state, writer), r.eval(state, writer), &|l, r| l != r),
            Expression::Lt(l, r) => bool_op(l.eval(state, writer), r.eval(state, writer), &|l, r| l < r),
            Expression::Gt(l, r) => bool_op(l.eval(state, writer), r.eval(state, writer), &|l, r| l > r),
            Expression::Ident(id) => state.get(&id).unwrap_or(&Null).clone(),
            Expression::String(s) => Value::Str(s.clone()),
            Expression::Neg(n) => if let Some(Int(i)) = n.eval(state, writer) { Int(-i) } else { Null },
            Expression::Not(n) => if let Some(Bool(b)) = n.eval(state, writer) { Bool(!b) } else { Null },
            Expression::If(cond, ifb, elb) => {
                let cond_val = cond.eval(state, writer);
                match cond_val {
                    Some(Bool(true)) => ifb.eval(state, writer).unwrap_or(Null),
                    Some(Bool(false)) => elb.eval(state, writer).unwrap_or(Null),
                    _ => Null,
                }
            },
            Expression::FnDecl(pars, stmt) => FnDecl(pars.clone(), stmt.clone()),
            Expression::Call(func, actual) => {
                match func.eval(state, writer) {
                    Some(FnDecl(formal, stmt)) => {
                        let mut fn_state = state.clone();
                        for i in 0..std::cmp::min(actual.len(), formal.len()) {
                            fn_state.set(&formal[i], actual[i].eval(state, writer).unwrap_or(Null));
                        }
                        stmt.eval(&mut fn_state, writer).unwrap_or(Null).unret()
                    },
                    Some(FnBuiltin(_, builtin)) => {
                        let (val, out) = builtin(actual.iter().map(|a| a.eval(state, writer)).collect::<Vec<Option<Value>>>());
                        out.map(|out| writer.write(out.as_bytes()));
                        *val
                    },
                    _ => Null,
                }
            },
        })
    }
}

pub fn eval(input: &str) -> Option<Value> {
    let mut out = std::io::stdout();
    State::new().eval(input, &mut out).map(|v| v.clone())
}

#[cfg(test)]
mod test {
    use super::eval;
    use super::Value::*;
    use super::State;

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

    #[test]
    fn test_len() {
        assert_eq!(eval("let x = \"a\"; let x = x + \" \"; len(x + 1)").unwrap(), Int(3));
    }

    #[test]
    fn test_print() {
        let mut out: Vec<u8> = Vec::new();
        State::new().eval("print(1, \"a\")", &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "1a");
    }
}
