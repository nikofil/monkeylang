use lexer::Lexer;
use parser::Parser;

pub fn start_repl() {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        println!("{:?}", program.statements());
    }
}
