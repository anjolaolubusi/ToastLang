mod parser;
mod lexer;

fn main() {
    println!("Hello, world!");
    let source = " def foo (a, b): (a+b)-(a*b) end";
    let mut parser = parser::Parser::new(source);
    let test = parser.parse();
    if !test.is_none() {
        println!("{:?}", test.unwrap().len());
    }
}
