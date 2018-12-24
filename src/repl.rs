use lexer::{ Lexer, Token };

pub fn start_repl() {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let mut lexer = Lexer::new(input);
        lexer.read_char();
        let tokens = lexer.collect::<Vec<Token>>();
        println!("{:?}", tokens);
    }
}
