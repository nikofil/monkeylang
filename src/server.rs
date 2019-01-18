use std::fs::File;
use std::env;
use std::path::Path;
use std::io::{ Read, Write };
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::collections::VecDeque;
use lexer::{ Token, Lexer, TokenLexer };
use parser::Parser;
use eval::{ State, Eval };

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

    stream.read(&mut buffer).unwrap();
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
                        let (path_str, args) = parse_get_args(part);
                        let cwd = env::current_dir().ok()?;
                        let public_path = cwd.join("public");
                        let file_path = public_path.join(path_str);
                        let path = file_path.canonicalize().ok()?;
                        println!("{} {}", path.to_str()?, public_path.to_str()?);
                        if path.starts_with(public_path) {
                            parse_file(path)
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
    args.split('&').map(|arg| {
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

fn parse_file(path: PathBuf) -> Option<String> {
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
            let mut output: Vec<u8> = Vec::new();
            program.eval(&mut State::new(), &mut output).map(|_| {
                String::from_utf8_lossy(&output).to_string()
            })
        },
        _ => Some(contents),
    }
}
