use std::fs::File;
use std::env;
use std::path::Path;
use std::io::prelude::{ Read, Write };
use std::net::TcpListener;
use std::net::TcpStream;

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
                            let mut contents = String::new();
                            File::open(path).ok()?.read_to_string(&mut contents).ok()?;
                            Some(contents)
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
