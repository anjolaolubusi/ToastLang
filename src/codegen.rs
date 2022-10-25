#![allow(non_snake_case)]
#![allow(unused_parens)]

use std::collections::HashMap;
use crate::{lexer::{Token}, parser::{ProtoAST, FuncAST}};
use inkwell::{context::Context, module::Module, values::{PointerValue, FunctionValue, FloatValue, BasicMetadataValueEnum, AnyValueEnum, IntValue}, types::BasicMetadataTypeEnum, passes::PassManager, execution_engine::{ExecutionEngine}};
use inkwell::builder::Builder;
use crate::parser::{ASTNode, ExprAST};
use uuid::Uuid;


pub struct Compiler<'a, 'ctx> {
    /// Stores core LLVM strcutres 
    pub context: &'ctx Context,
    /// Builds LLVM instructions
    pub builder: &'a Builder<'ctx>,
    /// Stores functions and global variables
    pub module: &'a Module<'ctx>,
    /// Current ASTNode
    pub expr: &'a ASTNode,
    pub firstPassManager: &'a PassManager<FunctionValue<'ctx>>,
    pub excutionEngine: &'a ExecutionEngine<'ctx>,
    /// Keeps track of variables
    variables: HashMap<String, PointerValue<'ctx>>,
    ///Keeps track of returned variable
    fn_value_opt: Option<FunctionValue<'ctx>>
}

impl <'a, 'ctx> Compiler<'a, 'ctx> {
   /// Gets a defined function given its name.
   #[inline]
   fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
       self.module.get_function(name)
   }

   /// Returns the `FunctionValue` representing the function being compiled.
   #[inline]
   fn fn_value(&self) -> FunctionValue<'ctx> {
       self.fn_value_opt.unwrap()
   }

   /// Creates a new stack allocation instruction in the entry block of the function.
   fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
       let builder = self.context.create_builder();

       let entry = self.fn_value().get_first_basic_block().unwrap();

       match entry.get_first_instruction() {
           Some(first_instr) => builder.position_before(&first_instr),
           None => builder.position_at_end(entry),
       }

       builder.build_alloca(self.context.f64_type(), name)
   }

   /// Compiles the specified `Expr` into an LLVM `FloatValue`.
   fn compile_expr(&mut self, expr: &ExprAST) -> Result<FloatValue<'ctx>, &'static str> {
       match *expr {
           ExprAST::NumberExpr(nb) => Ok(self.context.f64_type().const_float(nb)),
           ExprAST::VariableExpr(ref name) => match self.variables.get(name.as_str()) {
               Some(var) => Ok(self.builder.build_load(*var, name.as_str()).into_float_value()),
               None => Err("Could not find a matching variable."),
           },
           ExprAST::BinaryExpr {op, ref lhs, ref rhs, ref opChar} => {
                   let left = self.compile_expr(lhs)?;
                   let right = self.compile_expr(rhs)?;

                   match op {
                       Token::Plus => Ok(self.builder.build_float_add(left, right, "tmpadd")),
                       Token::Minus => Ok(self.builder.build_float_sub(left, right, "tmpsub")),
                       Token::Multiply => Ok(self.builder.build_float_mul(left, right, "tmpmul")),
                       Token::Divide => Ok(self.builder.build_float_div(left, right, "tmpdiv")),
                       Token::LessThan => {
                        let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::ULT, left, right, "cmptmp"); 
                        Ok(self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "tmpbool"))
                    },
                        Token::GreaterThan => {
                            let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::UGT, left, right, "cmptmp");
                            Ok(self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "tmpbool"))
                        },
                        Token::CustomBinOp => {
                            match self.get_function(&("binary".to_string() + opChar)){
                                Some(binaryFunc) => {
                                    let mut compiled_args = [left, right].to_vec();

                                    let argsv: Vec<BasicMetadataValueEnum> =
                                    compiled_args.iter().by_ref().map(|&val| val.into()).collect();
                                    
                                    match self
                                    .builder
                                    .build_call(binaryFunc, argsv.as_slice(), "tmp")
                                    .try_as_basic_value()
                                    .left()
                                {
                                    Some(value) => {
                                     Ok(value.into_float_value())},
                                    None => Err("Invalid call produced."),
                                }
                                },
                                _ => Err(&("Could not compile binary func"))
                            } 
                        },
                       _ => Err("Invalid Binary Operator")
                   }
                   //let F = self.get_function("binary")
               },
            ExprAST::UnaryExpr { ref Opcode, ref Operand } => {
                let compiledOper = BasicMetadataValueEnum::FloatValue(self.compile_expr(Operand).expect("Could not compile operand"));
                let unaryFunc = self.get_function(&("unary".to_string() + Opcode)).expect("Could not get unary function");
                match self
                .builder
                .build_call(unaryFunc, &[compiledOper], "unaryop")
                .try_as_basic_value()
                .left(){
                    Some(value) => {
                        Ok(value.into_float_value())
                    }
                    None => Err("Invalid call produced")
                }
            },
           ExprAST::CallExpr { ref func_name, ref parameters } => match self.get_function(func_name.as_str()) {
               Some(fun) => {
                   let mut compiled_args = Vec::with_capacity(parameters.len());

                   for arg in parameters {
                       compiled_args.push(self.compile_expr(arg)?);
                   }

                   let argsv: Vec<BasicMetadataValueEnum> =
                       compiled_args.iter().by_ref().map(|&val| val.into()).collect();

                   match self
                       .builder
                       .build_call(fun, argsv.as_slice(), "tmp")
                       .try_as_basic_value()
                       .left()
                   {
                       Some(value) => {
                        Ok(value.into_float_value())},
                       None => Err("Invalid call produced."),
                   }
               },
               None => Err("Unknown function."),
           },
           ExprAST::IfExpr { ref cond, ref Then, ref Else } => {
            let parentFunc = self.fn_value();
            let CondCodeGen = self.compile_expr(cond)?;
            let CondCodeGen = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, CondCodeGen, self.context.f64_type().const_float(0.0), "ifcond");
            
            
            let thenBB = self.context.append_basic_block(parentFunc, "then");
            let elseBB = self.context.append_basic_block(parentFunc, "else");
            let contBB = self.context.append_basic_block(parentFunc, "ifCont");
            self.builder.build_conditional_branch(CondCodeGen, thenBB, elseBB);
            
            self.builder.position_at_end(thenBB);
            let thenCodeGen = self.compile_expr(Then)?;
            self.builder.build_unconditional_branch(contBB);
            
            let thenBB = self.builder.get_insert_block().unwrap();

            
            self.builder.position_at_end(elseBB);
            let mut elseCodeGen = self.context.f64_type().const_float(0.0);
            if(Else.is_some()){
            elseCodeGen = self.compile_expr(&Else.clone().unwrap().to_owned()).expect("Could not compile else block");
            }
            self.builder.build_unconditional_branch(contBB);

            let elseBB = self.builder.get_insert_block().unwrap();


            self.builder.position_at_end(contBB);

            let phi = self.builder.build_phi(self.context.f64_type(), "if tmp");
            phi.add_incoming(&[(&thenCodeGen, thenBB), (&elseCodeGen, elseBB)]);
            Ok(phi.as_basic_value().into_float_value())
       },
            ExprAST::ForExpr { ref var, ref start, ref end, ref stepFunc, ref body } | ExprAST::InclusiveForExpr  { ref var, ref start, ref end, ref stepFunc, ref body } => {
                let parent = self.fn_value();
                let startBlockPointer = self.create_entry_block_alloca(var);
                let startVal = self.compile_expr(start)?;
                self.builder.build_store(startBlockPointer, startVal);

                let loopBB = self.context.append_basic_block(parent, "loopBlock");
                self.builder.build_unconditional_branch(loopBB);
                self.builder.position_at_end(loopBB);

                let old_val = self.variables.remove(var);
                self.variables.insert(var.to_owned(), startBlockPointer);
                
                self.compile_expr(body)?;
                
                let stepVal = self.compile_expr(stepFunc)?;

                let endVal = self.compile_expr(end)?;

                let currentVal = self.builder.build_load(startBlockPointer, var);
                let nextVal = self.builder.build_float_add(currentVal.into_float_value(),  stepVal, "nextVar");

                self.builder.build_store(startBlockPointer, nextVal);

                let mut end_cond : IntValue = self.context.i128_type().const_int(0, true);
                match expr{
                    ExprAST::ForExpr { var: _, start: _, end: _, stepFunc: _, body: _ } => {
                        end_cond = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, nextVal, endVal, "iterationCheck");
                    }
                    ExprAST::InclusiveForExpr { var: _, start: _, end: _, stepFunc: _, body: _ } => {
                        end_cond = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, currentVal.into_float_value(), endVal, "iterationCheck");
                    }
                    _=> {
                        Err::<ExprAST, &str>("Just how did you do this?");
                    }
                }
                //let end_cond = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, nextVal, endVal, "iterationCheck");
                let afterBB = self.context.append_basic_block(parent, "afterloop");

                self.builder.build_conditional_branch(end_cond, loopBB, afterBB);
                self.builder.position_at_end(afterBB);

                self.variables.remove(var);
                
                if let Some(val) = old_val {
                    self.variables.insert(var.to_string(), val);
                }

                Ok(self.context.f64_type().const_float(0.0))
            },
            ExprAST::CommentExpr(_) => {
                Ok(self.context.f64_type().const_float(0.0))
            }
       _=> Err("Unkown expression")
       }
   }

   /// Compiles the specified `Prototype` into an extern LLVM `FunctionValue`.
   fn compile_prototype(&self, proto: &ProtoAST) -> Result<FunctionValue<'ctx>, &'static str> {
       let ret_type = self.context.f64_type();
       let args_types = std::iter::repeat(ret_type)
           .take(proto.Args.len())
           .map(|f| f.into())
           .collect::<Vec<BasicMetadataTypeEnum>>();
       let args_types = args_types.as_slice();

       let fn_type = self.context.f64_type().fn_type(args_types, false);
       let fn_val = self.module.add_function(proto.Name.as_str(), fn_type, None);

       // set arguments names
       for (i, arg) in fn_val.get_param_iter().enumerate() {
           arg.into_float_value().set_name(proto.Args[i].as_str());
       }

       // finally return built prototype
       Ok(fn_val)
   }

    /// Compiles the specified `Function` into an LLVM `FunctionValue`.
    fn compile_fn(&mut self, cur_expr: &ASTNode) -> Result<AnyValueEnum<'ctx>, &'static str> {
        match cur_expr {
            ASTNode::FunctionNode(function_var) => {
                let proto = &function_var.Proto;
        let function = self.compile_prototype(proto)?;

        let entry = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(entry);

        // update fn field
        self.fn_value_opt = Some(function);

        // build variables map
        self.variables.reserve(proto.Args.len());

        for (i, arg) in function.get_param_iter().enumerate() {
            let arg_name = proto.Args[i].as_str();
            let alloca = self.create_entry_block_alloca(arg_name);

            self.builder.build_store(alloca, arg);

            self.variables.insert(proto.Args[i].clone(), alloca);
        }

        // compile body
        let body = self.compile_expr(&function_var.Body)?;

        self.builder.build_return(Some(&body));

        // return the whole thing after verification and optimization
        if function.verify(true) {
            self.firstPassManager.run_on(&function);

            Ok(AnyValueEnum::FunctionValue(function))
        } else {
            unsafe {
                function.delete();
            }

            Err("Invalid generated function.")
        }

            },
        ASTNode::ExternNode(proto) => {
            let function = self.compile_prototype(proto)?;

            if !Some(function).is_none() {
                return Ok(AnyValueEnum::FunctionValue(function))
            }
            Err("Could not compile external function")
        },
        ASTNode::ExpressionNode(expr) => {
            let mut funcName = "__anon_expr__".to_owned();
            let id = Uuid::new_v4().to_string().to_owned();
            funcName.push_str(id.to_string().as_str());

            let tempAST = ASTNode::FunctionNode(FuncAST{
                Proto: ProtoAST { Name: funcName.to_owned(), Args: Vec::new(), IsOperator: false, Precedence: 0 },
                Body: expr.to_owned()
            });
            let expr_out = self.compile_fn(&tempAST);
            if expr_out.is_ok() {
                self.excutionEngine.add_module(self.module);
                unsafe{
                    let test_fn = self.excutionEngine.get_function::<unsafe extern "C" fn() -> f64>(funcName.as_str()).unwrap();
                    test_fn.call();
                    self.excutionEngine.remove_module(self.module);
                }
                return expr_out;
            }
            Err("Could not compile expression")
        }
        // _ => Err("Invalid Function")
        }
    }

    /// Compiles the specified `Function` in the given `Context` and using the specified `Builder`, `PassManager`, and `Module`.
    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
        firstPassManager: &'a PassManager<FunctionValue<'ctx>>,
        excutionEngine: &'a ExecutionEngine<'ctx>,
        expr: &ASTNode
    ) -> Result<AnyValueEnum<'ctx>, &'static str> {
        let mut compiler = Compiler {
            context,
            builder,
            module,
            expr,
            firstPassManager,
            excutionEngine,
            fn_value_opt: None,
            variables: HashMap::new(),
        };

        compiler.compile_fn(expr)
    }

}
