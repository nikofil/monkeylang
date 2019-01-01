use eval::State;
use std::io::Write;

pub fn start_repl() {
    let mut state = State::new();
    loop {
        let mut input = String::new();
        print!(">> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        state.eval(&input, &mut std::io::stdout()).map(|out| println!("-> {}\n", out));
    }
}
