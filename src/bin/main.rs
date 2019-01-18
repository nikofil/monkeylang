extern crate monkeylang;

use std::env;
use monkeylang::repl;
use monkeylang::server;

fn main() {
    let mut args = env::args();
    args.next();
    match args.next().as_ref() {
        Some(s) if s == "-serve" => {
            let interface = match args.next().as_ref() {
                Some(s) => s.clone(),
                _ => String::from("localhost"),
            };
            let port = args.next().as_ref().and_then(|i| i.parse().ok()).unwrap_or(80);
            server::serve(interface, port);
        },
        _ => repl::start_repl(),
    }
}
