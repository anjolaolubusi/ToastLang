use inkwell::{context::Context, values::{AnyValueEnum, FunctionValue}, builder::Builder, passes::PassManager, OptimizationLevel, execution_engine::ExecutionEngine};
use crate::codegen::Compiler;
use std::{io::{self, Read, Write}};

mod parser;
mod lexer;
mod codegen;

fn main() {
    let mut buffer = "".to_string();
    let context = Context::create();
    let module = context.create_module("repl");
    let builder = context.create_builder();
    let ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    module.set_data_layout(&ee.get_target_data().get_data_layout());
    let fpm = PassManager::create(&module);
    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.initialize();
    let mut tempId = 0;
    loop{
    io::stdout().write_all(b"> ");
    io::stdin().read_to_string(&mut buffer);
    if buffer.trim() == "exit" {
        break;
    }
    let mut parser = parser::Parser::new(&buffer);
    let test = parser.parse();
    if !test.is_none() {
        let parsed_list = test.unwrap();
        for expr in parsed_list{
            //let module2 = context.create_module("tmp");
            match Compiler::compile(&context, &builder, &module, &fpm, &ee, &expr) {
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


    buffer = "".to_string();
    }
    //let source = "def foo (a, b): \n a-b \n end \n extern boo(a, b) \n 4 + 5";

}
