
use std::{io::{self, Read, Write}, env, fs};

// use crate::codegen::{ToastVM, ExprConverter};

mod parser;
mod lexer;
mod codegen;

// macro used to print & flush without printing a new line
macro_rules! print_flush {
    ( $( $x:expr ),* ) => {
        print!( $($x, )* );

        std::io::stdout().flush().expect("Could not flush to standard output.");
    };
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let mut buffer = "".to_string();
    let args: Vec<String> = env::args().collect();
    let mut toast_vm = codegen::VMCore::new();
    let mut ast_converter = codegen::ASTConverter::new();
    // let mut cpu: ToastVM = ToastVM::new();
    // let mut converter: ExprConverter = ExprConverter::new();
    println!("{:?}", args);
    match args.len() {
        1 => {

            loop{
                print_flush!("> ");
            //io::stdout().write_all(b"> ");
            io::stdin().read_to_string(&mut buffer);
            if buffer.trim() == "exit" {
                break;
            }
            let mut parser = parser::Parser::new(&buffer);
            let ast_nodes = parser.parse();

            println!("Parser: {:?}", &ast_nodes);
            for ast in &ast_nodes.unwrap() {
                ast_converter.ConvertExprToByteCode(ast.to_owned());
                toast_vm.processProgram(&ast_converter.program);
                println!("ToastVM: {:?}", toast_vm);
            }

            buffer = "".to_string();
            }
        },
        2 => {
            let contents = fs::read_to_string(args[1].clone()).expect("Expected file here");
            let mut parser = parser::Parser::new(&contents);
            let test = parser.parse();
            if !test.is_none() {
                let parsed_list = test.unwrap();
                // println!("-> Parsed: {:?}", parsed_list);
                for ast in &parsed_list {
                    ast_converter.ConvertExprToByteCode(ast.to_owned());
                    toast_vm.processProgram(&ast_converter.program);
                    println!("ToastVM: {:?}", toast_vm);
                }
            }
        },
        _ => {println!("Too many arguments")}
    }

    //let source = "def foo (a, b): \n a-b \n end \n extern boo(a, b) \n 4 + 5";

}
