use std::collections::HashMap;
use crate::{lexer::{Token}, parser::{ProtoAST, FuncAST}};
use inkwell::{context::Context, module::Module, values::{PointerValue, FunctionValue, FloatValue, BasicMetadataValueEnum, AnyValueEnum, BasicValueEnum}, types::BasicMetadataTypeEnum, passes::PassManager, execution_engine::{ExecutionEngine, JitFunction}};
use inkwell::builder::Builder;
use crate::parser::{ASTNode, ExprAST};
use uuid::Uuid;


pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context, //Stores core LLVM strcutres 
    pub builder: &'a Builder<'ctx>, //Builds LLVM instructions
    pub module: &'a Module<'ctx>, //Stores functions and global variables
    pub expr: &'a ASTNode, // Current ASTNode
    pub firstPassManager: &'a PassManager<FunctionValue<'ctx>>,
    pub excutionEngine: &'a ExecutionEngine<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>, //Keeps track of variables
    fn_value_opt: Option<FunctionValue<'ctx>> //Keeps track of returned variable
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
           ExprAST::BinaryExpr {
               op,
               ref lhs,
               ref rhs,
           } => {
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
                       _ => Err("Invalid Binary Operator")
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
                        let value2 = value.clone().into_float_value();
                        Ok(value.into_float_value())},
                       None => Err("Invalid call produced."),
                   }
               },
               None => Err("Unknown function."),
           },
           ExprAST::IfExpr { ref cond, ref Then, ref Else } => {
            let parentFunc = self.fn_value();
            let mut CondCodeGen = self.compile_expr(cond);
            if(CondCodeGen.is_err()){
                Err::<FloatValue, &'static str>("Condition failed to compile");
            }
            let mut CondCodeGen = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, CondCodeGen.unwrap(), self.context.f64_type().const_float(0.0), "ifcond");
            
            
            let thenBB = self.context.append_basic_block(parentFunc, "then");
            let elseBB = self.context.append_basic_block(parentFunc, "else");
            let contBB = self.context.append_basic_block(parentFunc, "ifCont");
            self.builder.build_conditional_branch(CondCodeGen, thenBB, elseBB);
            
            self.builder.position_at_end(thenBB);
            let thenCodeGen = self.compile_expr(Then);
            if(thenCodeGen.is_err()){
                Err::<FloatValue, &'static str>("Could not compile then block");
            }
            let thenCodeGen = thenCodeGen.unwrap();
            self.builder.build_unconditional_branch(contBB);
            
            let thenBB = self.builder.get_insert_block().unwrap();

            self.builder.position_at_end(elseBB);
            let elseCodeGen = self.compile_expr(Else)?;
            self.builder.build_unconditional_branch(contBB);

            let elseBB = self.builder.get_insert_block().unwrap();

            self.builder.position_at_end(contBB);

            let phi = self.builder.build_phi(self.context.f64_type(), "if tmp");
            phi.add_incoming(&[(&thenCodeGen, thenBB), (&elseCodeGen, elseBB)]);
            Ok(phi.as_basic_value().into_float_value())
       },
            ExprAST::ForExpr { ref var, ref start, ref end, ref stepFunc, ref body } => {
                let parent = self.fn_value();
                let startBlockPointer = self.create_entry_block_alloca(var);
                let startVal = self.compile_expr(start)?;
                self.builder.build_store(startBlockPointer, startVal);

                let loopBB = self.context.append_basic_block(parent, "loopBlock");
                self.builder.build_unconditional_branch(loopBB);
                self.builder.position_at_end(loopBB);

                let old_val = self.variables.remove(var);
                self.variables.insert(var.to_owned(), startBlockPointer);
                
                let bodyVal = self.compile_expr(body)?;
                
                let stepVal = self.compile_expr(stepFunc)?;

                let endVal = self.compile_expr(end)?;

                let currentVal = self.builder.build_load(startBlockPointer, var);
                let nextVal = self.builder.build_float_add(currentVal.into_float_value(),  stepVal, "nextVar");

                self.builder.build_store(startBlockPointer, nextVal);

                let end_cond = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, nextVal, endVal, "iterationCheck");
                let afterBB = self.context.append_basic_block(parent, "afterloop");

                self.builder.build_conditional_branch(end_cond, loopBB, afterBB);
                self.builder.position_at_end(afterBB);

                self.variables.remove(var);
                
                if let Some(val) = old_val {
                    self.variables.insert(var.to_string(), val);
                }

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
                Proto: ProtoAST { Name: funcName.to_owned(), Args: Vec::new() },
                Body: expr.to_owned()
            });
            let expr_out = self.compile_fn(&tempAST);
            if expr_out.is_ok() {
                self.excutionEngine.add_module(self.module);
                unsafe{
                    let test_fn = self.excutionEngine.get_function::<unsafe extern "C" fn() -> f64>(funcName.as_str()).unwrap();
                    let return_value = test_fn.call();
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