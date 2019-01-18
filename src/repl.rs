use eval::State;
use std::io;
use std::io::Write;

pub fn start_repl() {
    let mut state = State::new();
    loop {
        let mut input = String::new();
        print!(">> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        state.eval(&input, &mut io::stdout()).map(|out| println!("-> {}\n", out));
    }
}
