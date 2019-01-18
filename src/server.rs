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
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", "hello world!");
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
