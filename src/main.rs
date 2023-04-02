#![allow(non_snake_case)]
#![allow(unused_parens)]

use inkwell::{context::Context, values::{AnyValueEnum}, passes::PassManager, OptimizationLevel, builder::Builder, execution_engine::ExecutionEngine};
use crate::{codegen::Compiler, parser::ExprAST};
use std::{io::{self, Read, Write}, any::Any, env, fs};

mod parser;
mod lexer;
mod codegen;



pub struct ExternFN {
    pub Name: String,
    pub Function: dyn Fn() -> dyn Any
}

// macro used to print & flush without printing a new line
macro_rules! print_flush {
    ( $( $x:expr ),* ) => {
        print!( $($x, )* );

        std::io::stdout().flush().expect("Could not flush to standard output.");
    };
}

#[no_mangle]
pub extern "C" fn putchard(x: f64) -> f64 {
    print_flush!("{}", x as u8 as char);
    x
}

#[no_mangle]
pub extern "C" fn printd(x: f64) -> f64 {
    println!("{}", x);
    x
}

// Adding the functions above to a global array,
// so Rust compiler won't remove them.
#[used]
static EXTERNAL_FNS: [extern "C" fn(f64) -> f64; 2] = [putchard, printd];


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
    
    //TODO: Abstract this out to a class/method
    let ft = context.f64_type();
    let extf = module.add_function("printd", ft.fn_type(&[ft.into()], false), None);
    ee.add_global_mapping(&extf, printd as usize);
    let extf = module.add_function("putchard", ft.fn_type(&[ft.into()], false), None);
    ee.add_global_mapping(&extf, putchard as usize);

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let mut testComp = Compiler::new(&context, &builder, &module, &fpm, &ee);
    match args.len() {
        1 => {
            loop{
                print_flush!("?> ");
            io::stdout().write_all(b"> ");
            io::stdin().read_to_string(&mut buffer);
            if buffer.trim() == "exit" {
                break;
            }
            let mut parser = parser::Parser::new(&buffer);
            let test = parser.parse();
            if !test.is_none() {
                let parsed_list = test.unwrap();
                println!("-> Parsed: {:?}", parsed_list);
                for expr in parsed_list{
                    //let module2 = context.create_module("tmp");
                    match testComp.testComp(Some(&expr)){ //Compiler::compile(&context, &builder, &module, &fpm, &ee, Some(&expr)) {
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
        },
        2 => {
            let contents = fs::read_to_string(args[1].clone()).expect("Expected file here");
            let mut parser = parser::Parser::new(&contents);
            let test = parser.parse();
            if !test.is_none() {
                let parsed_list = test.unwrap();
                println!("-> Parsed: {:?}", parsed_list);
                for expr in parsed_list{
                    //let module2 = context.create_module("tmp");
                    match Compiler::compile(&context, &builder, &module, &fpm, &ee, Some(&expr)) {
                        Ok(function) => {
                            // println!("-> Expression compiled to IR:");
                            // match function{
                            //     AnyValueEnum::FunctionValue(temp) => temp.print_to_stderr(),
                            //     AnyValueEnum::FloatValue(temp) => temp.print_to_stderr(),
                            //     _ => println!("No print function")
                            // }
                        },
                        Err(err) => {
                            println!("!> Error compiling function: {}", err);
                            continue;
                        }
                    }
                }
            }
        },
        _ => {println!("Too many arguments")}
    }

    //let source = "def foo (a, b): \n a-b \n end \n extern boo(a, b) \n 4 + 5";

}
