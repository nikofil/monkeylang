use std::fs::File;
use std::env;
use std::io::{ Read, Write };
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::collections::VecDeque;
use lexer::{ Token, Lexer, TokenLexer };
use parser::Parser;
use eval::{ State, Eval, Value };
use std::collections::HashMap;

pub fn serve(interface: String, port: u16) {
    let listener = TcpListener::bind(format!("{}:{}", interface, port)).unwrap();

    println!("Serving to {} at port {}", interface, port);
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => {s},
            Err(_) => {continue},
        };
        println!("New connection from {}", stream.peer_addr().unwrap());

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 40960];

    let read_size = stream.read(&mut buffer).unwrap();
    let buffer = &buffer[0..read_size];
    let buf_utf = String::from_utf8_lossy(&buffer[..]);
    let mut lines = buf_utf.lines();
    match lines.next() {
        None => {},
        Some(req) => {
            println!("Request: {}", req);
            let mut parts = req.split(' ');
            let contents = match parts.next() {
                Some(verb) if verb == "GET" || verb == "POST" => {
                    parts.next().and_then(|part| {
                        let (path_str, get_args) = parse_get_args(part);
                        let post_args = if verb == "POST" {
                            while let Some(l) = lines.next() {
                                if l.trim().len() == 0 {
                                    break;
                                }
                            }
                            lines.next().map(parse_form_args).unwrap_or(Vec::new())
                        } else {
                            Vec::new()
                        };
                        let cwd = env::current_dir().ok()?;
                        let public_path = cwd.join("public");
                        let file_path = public_path.join(path_str);
                        let path = file_path.canonicalize().ok()?;
                        if path.starts_with(public_path) {
                            parse_file(path, get_args, post_args)
                        } else {
                            None
                        }
                    })
                },
                _ => None,
            };
            let response = format!("HTTP/1.1 {}\r\n\r\n{}", if contents.is_some() {
                "200 OK"
            } else {
                "404 NOT FOUND"
            }, contents.unwrap_or(String::from("Not found\r\n")));
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        },
    }
}

fn parse_get_args(req: &str) -> (&str, Vec<(&str, &str)>) {
    let mut split = req.split('?');
    let path = split.next().unwrap();
    let args = split.next().map(parse_form_args);
    (if path == "/" { "index.ml" } else { &path[1..] }, args.unwrap_or(Vec::new()))
}

fn parse_form_args(args: &str) -> Vec<(&str, &str)> {
    args.trim().split('&').map(|arg| {
        let mut arg_split = arg.split('=');
        let name = arg_split.next().unwrap();
        let val = arg_split.next().unwrap_or("");
        (name, val)
    }).collect::<Vec<(&str, &str)>>()
}

struct ScriptLexer(VecDeque<Token>);

impl TokenLexer for ScriptLexer {
    fn next_token(&mut self) -> Token {
        self.0.pop_front().unwrap_or(Token::Eof)
    }
}

fn parse_file(path: PathBuf, get_args: Vec<(&str, &str)>, post_args: Vec<(&str, &str)>) -> Option<String> {
    let mut contents = String::new();
    File::open(path.clone()).ok()?.read_to_string(&mut contents).ok()?;
    match path.extension() {
        Some(ext) if ext == "ml" => {
            let mut line_buf = VecDeque::new();
            let mut is_ml = false;
            contents.lines().for_each(|line| {
                if is_ml {
                    if line == "%>" {
                        is_ml = false;
                    } else {
                        for tok in Lexer::lex_str(line) {
                            line_buf.push_back(tok);
                        }
                    }
                } else if line == "<%" {
                    is_ml = true;
                } else {
                    line_buf.push_back(Token::Ident(String::from("println")));
                    line_buf.push_back(Token::Lparen);
                    line_buf.push_back(Token::String(String::from(line)));
                    line_buf.push_back(Token::Rparen);
                }
            });
            let program = Parser::new(&mut ScriptLexer(line_buf)).parse_program();
            let mut state = State::new();
            let mut get_map = HashMap::new();
            let mut post_map = HashMap::new();
            for (k, v) in get_args {
                get_map.insert(String::from(k), parse_value(v));
            }
            for (k, v) in post_args {
                post_map.insert(String::from(k), parse_value(v));
            }
            state.set(&String::from("get"), Value::Hash(get_map));
            state.set(&String::from("post"), Value::Hash(post_map));

            let mut output: Vec<u8> = Vec::new();
            program.eval(&mut state, &mut output).map(|_| {
                String::from_utf8_lossy(&output).to_string()
            })
        },
        _ => Some(contents),
    }
}

fn parse_value(val: &str) -> Value {
    if val == "true" {
        Value::Bool(true)
    } else if val == "false" {
        Value::Bool(false)
    } else if let Ok(i) = val.trim().parse::<i32>() {
        Value::Int(i)
    } else {
        Value::Str(String::from(val))
    }
}
