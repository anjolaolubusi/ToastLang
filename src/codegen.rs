#![allow(non_snake_case)]
#![allow(unused_parens)]
use std::{collections::HashMap, u16};

use crate::parser::ExprAST;
use crate::lexer::Token;
use num;
use num_derive::{self, FromPrimitive};

// Holds memory of function
#[derive(Debug)]
pub struct MemoryBlock {
    pub numberStack: Vec<f64>
}

impl MemoryBlock{
    pub fn new() -> Self{
        MemoryBlock {
            numberStack: Vec::<f64>::new()
        }
    }
}

//Holds  cpu function of core
#[derive(Debug)]
pub struct VMCore {
    pub registers: [u64; 8],
    pub pc: usize,
    pub cond: u8,
    pub memoryList: Vec<MemoryBlock>,
    /// Key is function Id, Value is (Start pc value, list of param types)
    pub funcList: HashMap<usize, (usize, Vec<VarTypes>)>,
    pub curMemoryId: usize,
    pub curFunctionId: usize,
    pub curType: VarTypes,
}


impl VMCore {
    pub fn new() -> Self{
        let mut vm = VMCore {
            registers: [0; 8],
            pc: 0,
            cond: 0,
            memoryList: Vec::<MemoryBlock>::new(),
            funcList: HashMap::new(),
            curMemoryId: 0,
            curFunctionId: 0,
            curType: VarTypes::FloatType
        };
        vm.memoryList.push(MemoryBlock::new());
        return vm;
    }

    pub fn processProgram(&mut self, program: &Vec<u16>){
        let mut byteCode;
        while self.pc < program.len(){
            byteCode = program[self.pc];
            self.ConsumeByteCode(program, byteCode);
            self.pc = self.pc + 1;
        }
    }

    pub fn ConsumeByteCode(&mut self, program: &Vec<u16>, byteCode: u16){
        let opCode : OpCodes = num::FromPrimitive::from_u16(byteCode >> 12).unwrap();
        match opCode {
            OpCodes::OpLoadFloat => {
                self.curType = VarTypes::FloatType;
                // Shifts byte code by 9 bits to the right. Masks it by 7 (00000111).
                let reg = (byteCode >> 9) & 7;
                self.pc = self.pc + 1;
                let mut num: u64 = 0;
                //Float is seperated in 4 16 bit chunks
                num = num | program[self.pc] as u64 | (program[self.pc+1] as u64) << 16 | (program[self.pc+2] as u64) << 32 | (program[self.pc+3] as u64) << 48;
                self.pc = self.pc+3;
                self.registers[reg as usize] = num;
                println!("Float Value: {}", f64::from_bits(self.registers[reg as usize]))
            },
            OpCodes::OpAdd | OpCodes::OpSub | OpCodes::OpDiv | OpCodes::OpMul => {
                // Shifts byte code by 9 bits to the right. Masks it by 7 (00000111).
                let reg1 = (byteCode >> 9) & 7;
                match self.curType {
                    VarTypes::FloatType => {
                        // Mask bytecode by 7 (00000111)
                        let reg2 = byteCode & 7;
                        match opCode {
                            OpCodes::OpAdd => {self.registers[reg1 as usize] = f64::to_bits(f64::from_bits(self.registers[reg1 as usize]) + f64::from_bits(self.registers[reg2 as usize]));},
                            OpCodes::OpSub => {self.registers[reg1 as usize] = f64::to_bits(f64::from_bits(self.registers[reg1 as usize]) - f64::from_bits(self.registers[reg2 as usize]));},
                            OpCodes::OpMul => {self.registers[reg1 as usize] = f64::to_bits(f64::from_bits(self.registers[reg1 as usize]) * f64::from_bits(self.registers[reg2 as usize]));},
                            OpCodes::OpDiv => {self.registers[reg1 as usize] = f64::to_bits(f64::from_bits(self.registers[reg1 as usize]) / f64::from_bits(self.registers[reg2 as usize]));},
                            _ => {print!("Unkown Operation")}
                        }
                        println!("Answer: {}", f64::from_bits(self.registers[reg1 as usize]))
                    },
                    _ => {println!("Unkown number type")}
                }
            },
            OpCodes::OpLoadReg => {
                let sourceRegNum = (byteCode >> 8) & 15;
                let destRegNum = (byteCode >> 4) & 15;
                self.registers[destRegNum as usize] = self.registers[sourceRegNum as usize];
            },
            OpCodes::OpType => {
                let varType: VarTypes = num::FromPrimitive::from_u16(byteCode & 0x0FFF).unwrap();
                self.curType = varType;
            },
            OpCodes::OpNewVar => {
                match self.curType {
                    VarTypes::FloatType => {
                        let reg = byteCode & 7;
                        self.memoryList.get_mut(self.curMemoryId).unwrap().numberStack.push(f64::from_bits(self.registers[reg as usize]));
                    }
                    _ => {println!("Unkown variable type")}
                }
            },
            OpCodes::OpLoadVar => {
                match self.curType {
                    VarTypes::FloatType => {
                        let reg = (byteCode >> 9) & 7;
                        self.pc = self.pc + 1;
                        let mut varId: u64 = 0;
                        varId = varId | program[self.pc] as u64 | (program[self.pc+1] as u64) << 16 | (program[self.pc+2] as u64) << 32 | (program[self.pc+3] as u64) << 48;
                        self.pc = self.pc+3;
                        self.registers[reg as usize] = f64::to_bits( *self.memoryList.get(self.curMemoryId).unwrap().numberStack.get(varId as usize).unwrap() );
                        println!("Variable Value: {}", f64::from_bits(self.registers[reg as usize]))

                    }
                    _ => {println!("Unkown variable type")}
                }
            },
            OpCodes::OpStartFunc => {
                let param_num = byteCode & 0x0FFF;
                println!("Para Number is {param_num}");
                let mut paramTypes = Vec::<VarTypes>::new();
                for _ in 0..param_num{
                    self.pc = self.pc + 1;
                    let varType : VarTypes = num::FromPrimitive::from_u16(program[self.pc] & 0x0FFF).unwrap();
                    paramTypes.push(varType);
                }
                self.pc += 1;
                let startPCval = self.pc;
                self.funcList.insert(self.curFunctionId, (startPCval, paramTypes.clone()));
                while program[self.pc] >> 12 != (OpCodes::OpEndFunc as u16) {
                    self.pc += 1;
                }
                self.curFunctionId += 1;

                // self.pc += 1;
                
            },
            OpCodes::OpCallFunc => {
                let function_id = byteCode & 0x0FFF;
                let func_data = self.funcList.get(&(function_id as usize)).unwrap_or_else(|| {panic!("Unkown function")}).clone();
                self.pc += 1;
                self.memoryList.push(MemoryBlock::new());
                self.curMemoryId += 1;
                while (program[self.pc] >> 12) != OpCodes::OpEndParamLoad as u16 {
                    self.ConsumeByteCode(program, program[self.pc]);
                    self.pc += 1;
                }
                let oldPC = self.pc;
                self.pc = func_data.0;
                while (program[self.pc] >> 12)  != (OpCodes::OpEndFunc as u16) {
                    self.ConsumeByteCode(program, program[self.pc]);
                    self.pc += 1;
                }
                self.pc = oldPC;
                self.memoryList.pop();
                self.curMemoryId -= 1;

            }
            _ => println!("No implementation for opcode: {:#?}", opCode)
        }
        
    }
}

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum OpCodes {
    /// OpLoadReg - Operation Code for copy data from register to another
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 4 bits - Source 
    /// 
    /// Next 4 bits - Destination
    OpLoadReg = 0,
    //// OpLoadFloat - Operation Code for loading flaoats into a specified register
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - Register
    OpLoadFloat,
    /// OpAdd - Operation Code for adding two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// 
    /// Next 3 bits - Second Reg
    OpAdd,
    /// OpSub- Operation Code for subtracting two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// 
    /// Next 3 bits - Second Reg
    OpSub,
    /// OpMul- Operation Code for multiplying two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// 
    /// Next 3 bits - Second Reg
    OpMul,
    /// OpDiv- Operation Code for dividing two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// 
    /// Next 3 bits - Second Reg
    OpDiv,
    //// OpType - Operation Code for setting the current type
    /// 
    /// First 4 bits - OpCode
    ///
    ///  Next 12 bits - Denotes type
    OpType,
    //// OpNewVar - Operation Code for adding a variable
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Last 3 bits - Reg of variable value
    OpNewVar,
    //// OpLoadVar - Operation Load Variable To register
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - Register Num
    OpLoadVar,
    //// OpStartFunc - Operation Code to Start Function Definition
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Last 12 bits - Number of parameters
    OpStartFunc,
    //// OpAddFuncParameter - Operation Code to Add Function Parameter
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Last 12 bits - VarType
    OpAddFuncParameter,
    //// OpEndFunc - Operation Code to Start Function Definition
    /// 
    /// First 4 bits - OpCode
    OpEndFunc,
    //// OpCallFunc - Operation Code to call functiomn
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Last 12 bits - Function Id
    OpCallFunc,
    OpEndParamLoad
}

pub struct ASTConverter {
    pub funcIdTable: HashMap<String, u64>,
    // Key is variable name, Value is (Memory Block, VarType, Variable Id)
    pub varLookUp: HashMap<String, (u128, VarTypes, u64)>,
    pub program: Vec<u16>,
    pub curType: VarTypes,
    pub curMemoryBlock: u128,
    pub curNumVarId: u64,
    pub curFuncId: u64,
    pub free_reg: u8
}

#[derive(FromPrimitive, Debug, Clone, Copy)]
pub enum VarTypes{
    NullType=0,
    FloatType,
    CharType
}


impl ASTConverter {
    pub fn new() -> Self{
        ASTConverter{
            funcIdTable: HashMap::new(),
            varLookUp: HashMap::new(),
            program: Vec::<u16>::new(),
            curType: VarTypes::FloatType,
            curMemoryBlock: 0,
            curNumVarId: 0,
            curFuncId: 0,
            free_reg: 0
        }
    }

    pub fn ConvertExprToByteCode(&mut self, expr: ExprAST) -> Option<u8> {
        match expr {
            ExprAST::NumberExpr(num) => {
                let mut byteCode: u16 = 0;
                //Loads Op Code
                byteCode = byteCode | ((OpCodes::OpLoadFloat as u16) << 12);

                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                //Load register into bytecode
                byteCode = byteCode | ((register as u16) << 9);

                // Adds to the program list
                self.program.push(byteCode);

                let floatBits = f64::to_bits(num);
                self.program.push( (floatBits & 0xFFFF) as u16); //0-15
                self.program.push( (floatBits >> 16 & 0xFFFF) as u16);//16-31
                self.program.push( ((floatBits >> 32 & 0xFFFF)) as u16); //31-47
                self.program.push( ((floatBits >> 48 & 0xFFFF)) as u16); //48-63
                // self.curType = VarTypes::FloatType;
                self.UpdateCurType(VarTypes::FloatType);
                return Some(register); 
            },
            ExprAST::CharExpr(val) => {
                let mut byteCode: u16 = 0;
                return  None;
            }
            ExprAST::VariableExpr(name) => {
                let mut byteCode: u16 = 0;
                let varIdTuple = self.varLookUp.get(&name).unwrap().clone();
                self.UpdateCurType(varIdTuple.1);
                let varId = varIdTuple.2;
                byteCode = byteCode | ((OpCodes::OpLoadVar as u16) << 12);
                
                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                byteCode = byteCode | ((register as u16) << 9);
                self.program.push(byteCode);

                self.program.push( (varId & 0xFFFF) as u16); //0-15
                self.program.push( (varId >> 16 & 0xFFFF) as u16);//16-31
                self.program.push( ((varId >> 32 & 0xFFFF)) as u16); //31-47
                self.program.push( ((varId >> 48 & 0xFFFF)) as u16); //48-63

                return Some(register);
            }
            ExprAST::VariableAssignExpr { varObject, value } => {
                let mut byteCode: u16 = 0;
                let num_register_val: u8;
                if let ExprAST::VariableHeader { name, typeName } = *varObject.to_owned() {
                    match typeName.as_str() {
                        "number" => {
                            // self.curType = VarTypes::FloatType;
                            self.UpdateCurType(VarTypes::FloatType);
                            num_register_val = self.ConvertExprToByteCode(*value).expect("Can not compile variable value");
                            self.varLookUp.insert(name, (self.curMemoryBlock, self.curType, self.curNumVarId));
                            byteCode = byteCode | ((OpCodes::OpNewVar as u16) << 12) | num_register_val as u16;
                            self.program.push(byteCode);
                            self.curNumVarId += 1;
                            return Some(num_register_val);
                        },
                        _ => panic!("Can not compile variable type")
                    }

                    // match *value {
                    //     ExprAST::NumberExpr(num) =>  {
                    //         num_register_val = self.ConvertExprToByteCode(*value);
                    //     },
                    //     _ => panic!("Can not compile variable value")
                    // }
                }
                return None;
            }
            ExprAST::BinaryExpr { op, lhs, rhs, opChar: _ } => {
                // Gets register for the left hand side
                let reg1 = self.ConvertExprToByteCode(*lhs).unwrap();
                let mut byteCode : u16 = 0;
                // Gets right op code for operation
                let opCode : u8 = match op {
                    Token::Plus => OpCodes::OpAdd as u8,
                    Token::Minus => OpCodes::OpSub as u8,
                    Token::Multiply => OpCodes::OpMul as u8,
                    Token::Divide => OpCodes::OpDiv as u8,
                    _ => 0 as u8
                };
                // Loads opCode and register into bytecode
                byteCode = byteCode | (opCode as u16) << 12 | ( (reg1 as u16) << 9);
                match *rhs {
                    ExprAST::NumberExpr(_) => {
                        // Gets register for the right hand side
                        let reg2 = self.ConvertExprToByteCode(*rhs).unwrap();
                        // Loads register to bytecode
                        byteCode = byteCode | (reg2 as u16);
                        // Pushed bytecode to program list
                        self.program.push(byteCode);
                        return Some(reg1);
                    },
                    ExprAST::BinaryExpr { op, lhs, rhs, opChar } => {
                        let binExpr = ExprAST::BinaryExpr { op: op, lhs: lhs, rhs: rhs, opChar: opChar };
                        let binExprReg = self.ConvertExprToByteCode(binExpr).unwrap();
                        byteCode = byteCode | (0 << 8) | (binExprReg as u16);
                        self.program.push(byteCode);
                        return Some(reg1);
                    }
                    _ => {return None;}
                }
            },
            ExprAST::FuncExpr { name, args, body } => {
                let mut bytecode: u16 = 0;
                let oldMemoryBlockId = self.curMemoryBlock;
                let oldNumVarId = self.curNumVarId;
                let oldVarLookup = self.varLookUp.clone();
                self.curNumVarId = 0;
                self.varLookUp = HashMap::new();
                self.curMemoryBlock = self.curMemoryBlock + 1;
                // Insert Function Name with funcID
                self.funcIdTable.insert(name, self.curFuncId);
                self.curFuncId = self.curFuncId + 1;
                let param_count = (args.len() as u16);
                bytecode = bytecode  | (OpCodes::OpStartFunc as u16) << 12 | param_count;
                self.program.push(bytecode);

                // Loop through arguments and load them in to the function def
                for param in args{
                    if let ExprAST::VariableHeader { name, typeName } = param {
                        match typeName.as_str() {
                            "number" => {
                                bytecode = 0 | (OpCodes::OpAddFuncParameter as u16) << 12 | (VarTypes::FloatType as u16);
                                self.program.push(bytecode);
                                self.varLookUp.insert(name, (self.curMemoryBlock, VarTypes::FloatType, self.curNumVarId));
                                self.curNumVarId += 1;
                            }
                            _ => panic!("Can not compile data types")
                        }
                    }
                }

                // Parse through body
                let mut lastReg: u8 = 0;
                for bodyExpr in body{
                    lastReg = self.ConvertExprToByteCode(bodyExpr).unwrap();
                }
                // Add FuncEnd part
                bytecode = 0 | (OpCodes::OpEndFunc as u16) << 12;
                self.program.push(bytecode);
                self.curMemoryBlock = oldMemoryBlockId;
                self.curNumVarId = oldNumVarId;
                self.varLookUp = oldVarLookup.clone();
                return Some(lastReg);
            },
            ExprAST::CallExpr { func_name, parameters } => {
                let funcIdOption = self.funcIdTable.get(&func_name);
                let mut bytecode: u16;
                if funcIdOption.is_none() {
                    println!("Function {:#?} Not found", func_name.as_str());
                }
                bytecode = 0 | (OpCodes::OpCallFunc as u16) << 12 | (*funcIdOption.unwrap()) as u16;
                self.program.push(bytecode);
                let mut param_reg :Option<u8> = Some(self.free_reg);
                for param in parameters {
                    param_reg = self.ConvertExprToByteCode(param);
                    bytecode = 0 | (OpCodes::OpNewVar as u16) << 12 | (param_reg.unwrap() as u16);
                    self.program.push(bytecode);
                }
                bytecode = 0 |  (OpCodes::OpEndParamLoad as u16) << 12;
                self.program.push(bytecode);
                return param_reg;
            }
            _ => {println!("Could not convert expression to bytecode"); return None;}
        }
    }

    pub fn UpdateCurType(&mut self, curType: VarTypes){
        self.curType = curType;
        let byteCode = 0 | (OpCodes::OpType as u16) << 12 | (curType as u16);
        self.program.push(byteCode);
    }

}

