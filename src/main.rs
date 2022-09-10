use inkwell::{context::Context, values::{AnyValue, AnyValueEnum}};

use crate::codegen::Compiler;

mod parser;
mod lexer;
mod codegen;

fn main() {
    let source = "def foo (a, b): \n a-b \n end \n extern boo(a, b) \n 4 + 5";
    let mut parser = parser::Parser::new(source);
    let test = parser.parse();
    
    if !test.is_none() {
        let parsedList = test.unwrap();
        let context = Context::create();
        let module = context.create_module("repl");
        let builder = context.create_builder();
        for expr in parsedList{
            let module = context.create_module("tmp");
            match Compiler::compile(&context, &builder, &module, &expr) {
                Ok(function) => {
                    println!("-> Expression compiled to IR:");
                    match function{
                        AnyValueEnum::FunctionValue(temp) => temp.print_to_stderr(),
                        AnyValueEnum::FloatValue(temp) => temp.print_to_stderr(),
                        _ => println!("No print function")
                    }
                },
                Err(err) => {
                    println!("!> Error compiling function: {}", err);
                    continue;
                }
            }
        }
    }
}
