#![allow(non_snake_case)]
#![allow(unused_parens)]
use std::{array, collections::HashMap, u16};

use crate::parser::ExprAST;
use crate::lexer::Token;
use num::{self, range};
use num_derive::{self, FromPrimitive};

// Holds memory of function
#[derive(Debug)]
pub struct MemoryBlock {
    pub variableLookup: HashMap<u64, (VarTypes, u64)>,
    pub listLookup: Vec<(VarTypes, Vec<u64>)>
    
}

impl MemoryBlock{
    pub fn new() -> Self{
        MemoryBlock {
            variableLookup: HashMap::new(),
            listLookup: Vec::new()
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

    pub fn processProgram(&mut self, program: &Vec<u8>){
        let mut byteCode;
        while self.pc < program.len(){
            byteCode = program[self.pc];
            self.ConsumeByteCode(program, byteCode);
            self.pc = self.pc + 1;
        }
    }

    pub fn ConsumeByteCode(&mut self, program: &Vec<u8>, mut byteCode: u8){
        let opCode : OpCodes = num::FromPrimitive::from_u8(byteCode).unwrap();
        match opCode {
            OpCodes::OpLoadScalar => {
                // Add check for Scalar Type
                // Determine action based on scalar type
                self.pc += 1;
                byteCode = program[self.pc];
                // Only grabs the first 5 bits (31 is all first 5 bits as one) of the bytecode since that is where the current value type is
                let curType: VarTypes = num::FromPrimitive::from_u8((byteCode & 31)).unwrap();
                match curType {
                    VarTypes::FloatType => {
                        self.curType = VarTypes::FloatType;
                        // Shifts byte code by 5 bits to the right. Masks it by 7 (00000111).
                        let reg = (byteCode >> 5) & 7;
                        let mut num: u64 = 0;
                        //Floats seperated in to 8 8 bit chunks. Currenytly set to 64 bits. Might change if porting to a 32 bit system.
                        for i in range(0, 8){
                            self.pc += 1;
                            num = (num << 8 * (i > 0) as u8 ) | program[self.pc]as u64;
                        }
                        self.registers[reg as usize] = num;
                        println!("Float Value: {}", f64::from_bits(self.registers[reg as usize]))
                    },
                    VarTypes::CharType => {
                        self.curType = VarTypes::CharType;
                        // Only grabs the first 5 bits (31 is all first 5 bits as one) of the bytecode since that is where the current value type is
                        let reg = (byteCode >> 5) & 7;
                        self.pc = self.pc + 1;
                        let charBit = program[self.pc];
                        self.registers[reg as usize] = charBit as u64;
                        println!("Char Value: {:?}", (self.registers[reg as usize] as u8) as char);
                    }
                    _ => panic!("Unkown Type")
                }
            },
            OpCodes::OpAdd | OpCodes::OpSub | OpCodes::OpDiv | OpCodes::OpMul => {
                // Shifts byte code by 9 bits to the right. Masks it by 7 (00000111).
                self.pc += 1;
                byteCode = program[self.pc];
                let reg1 = (byteCode >> 5) & 7;
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
                    VarTypes::StringType => {
                        let reg2 = byteCode & 7;
                        match opCode {
                            OpCodes::OpAdd => {
                                let currMemoryList = self.memoryList.get(self.curMemoryId).unwrap();
                                let right_arr : Vec<u8> = currMemoryList.listLookup.get(self.registers[reg1 as usize] as usize).unwrap().1.clone().into_iter().map(|x| x as u8).collect();
                                let left_arr : Vec<u8> = currMemoryList.listLookup.get(self.registers[reg2 as usize] as usize).unwrap().1.clone().into_iter().map(|x| x as u8).collect();
                                let arr: Vec<u8> = [left_arr, right_arr].concat();
                                println!("{:?}", String::from_utf8(arr).unwrap());
                            }
                            _ => {print!("Unkown Operation")}
                        }
                    },
                    _ => {println!("Unkown number type")}
                }
            },
            OpCodes::OpLoadReg => {
                let sourceRegNum = (byteCode >> 5) & 0x0F;
                let destRegNum = (byteCode) & 7;
                self.registers[destRegNum as usize] = self.registers[sourceRegNum as usize];
            },
            OpCodes::OpNewVar => {
                self.pc += 1;
                let reg = (program[self.pc] >> 5) & 7;
                let curMemory = self.memoryList.get_mut(self.curMemoryId).unwrap();
                let variableType: VarTypes = num::FromPrimitive::from_u8((program[self.pc] & 0x0F)).unwrap();
                curMemory.variableLookup.insert(curMemory.variableLookup.len() as u64, ( variableType, self.registers[reg as usize]));
            },
            OpCodes::OpLoadVar => {
                self.pc += 1;
                let reg = (program[self.pc]  >> 5) & 7;
                let typeVal : VarTypes = num::FromPrimitive::from_u8(program[self.pc]  & 31).unwrap();
                let mut varId: u64 = 0;
                for i in range(0, 8){
                    self.pc += 1;
                    varId = (varId << 8 * (i > 0) as u8 ) | program[self.pc]as u64;
                }
                // varId = varId | program[self.pc] as u64 | (program[self.pc+1] as u64) << 16 | (program[self.pc+2] as u64) << 32 | (program[self.pc+3] as u64) << 48;
                // self.pc = self.pc+3;
                self.registers[reg as usize] = self.memoryList.get(self.curMemoryId).unwrap().variableLookup.get(&varId).unwrap().1;
                match typeVal {
                    VarTypes::FloatType => {
                        println!("Variable Value: {}", f64::from_bits(self.registers[reg as usize]))
                    },
                    VarTypes::CharType => {
                        println!("Char Value: {:?}", (self.registers[reg as usize] as u8) as char);
                    },
                    VarTypes::StringType => {
                        let arr_vec = self.memoryList.get(self.curMemoryId).unwrap().listLookup.get(self.registers[reg as usize] as usize).unwrap().1.clone();
                        // array_vec.clone().into_iter().map(|x| x as u64).collect()
                        let string_vec: Vec<u16> = arr_vec.into_iter().map(|x| x as u16).collect();
                        println!("String Value: {:?}", String::from_utf16(string_vec.as_slice()).unwrap())
                    }
                    _ => {println!("Unkown variable type")}
                }
            },
            OpCodes::OpStartFunc => {
                //Move from opCode
                self.pc += 1;
                //Grab parameter number (Max 255)
                let param_num = program[self.pc];
                println!("Para Number is {param_num}");
                let mut paramTypes = Vec::<VarTypes>::new();
                for _ in 0..param_num{
                    self.pc = self.pc + 1;
                    if program[self.pc] == OpCodes::OpAddFuncParameter as u8 {
                        self.pc = self.pc + 1;
                        let varType : VarTypes = num::FromPrimitive::from_u8(program[self.pc]).unwrap();
                        paramTypes.push(varType);
                    }
                }
                self.pc += 1;
                let startPCval = self.pc;
                self.funcList.insert(self.curFunctionId, (startPCval, paramTypes.clone()));
                while program[self.pc] != (OpCodes::OpEndFunc as u8) {
                    self.pc += 1;
                }
                self.curFunctionId += 1;

                // self.pc += 1;
                
            },
            OpCodes::OpCallFunc => {
                let mut function_id = 0;
                for i in range(0, 8){
                    self.pc += 1;
                    function_id = (function_id << 8 * (i > 0) as u8 ) | program[self.pc]as u64;
                }
                let func_data = self.funcList.get(&(function_id as usize)).unwrap_or_else(|| {panic!("Unkown function")}).clone();
                self.pc += 1;
                self.memoryList.push(MemoryBlock::new());
                self.curMemoryId += 1;
                while program[self.pc] != OpCodes::OpEndParamLoad as u8{
                    self.ConsumeByteCode(program, program[self.pc]);
                    self.pc += 1;
                }
                let oldPC = self.pc;
                self.pc = func_data.0;
                while program[self.pc]  != (OpCodes::OpEndFunc as u8) {
                    self.ConsumeByteCode(program, program[self.pc]);
                    self.pc += 1;
                }
                self.pc = oldPC;
                self.memoryList.pop();
                self.curMemoryId -= 1;

            },
            OpCodes::OpLoadArray => {
                self.curType = VarTypes::ArrayType;
                let reg = (byteCode >> 5) & 7;
                self.pc += 1;
                byteCode = program[self.pc];
                let elementType: VarTypes = num::FromPrimitive::from_u8(byteCode & 31).unwrap();
                let mut array_vec = Vec::<u8>::new();
                self.pc += 1;
                while (program[self.pc] != OpCodes::OpEndArray as u8) {
                    array_vec.push(program[self.pc]);
                    self.pc += 1;
                }
                self.memoryList.get_mut(self.curMemoryId).unwrap().listLookup.push((elementType, array_vec.clone().into_iter().map(|x| x as u64).collect()));
                self.registers[reg as usize] = (self.memoryList.get(self.curMemoryId).unwrap().listLookup.len()-1) as u64;
                match elementType {
                    VarTypes::CharType => {
                        self.curType = VarTypes::StringType; 
                        println!("String Value: {:?}", String::from_utf8(array_vec).unwrap())},
                    VarTypes::FloatType => {
                        self.curType = VarTypes::FloatType;
                        let mut num: u64 = 0;
                        for i in range(0, (array_vec.len())/8 ){
                            num = 0;
                            for j in range(0, 8){
                                num = (num << 8 * (j > 0) as u64) | array_vec[ (i * 8) + j] as u64;
                            }
                        }
                        println!("Float Value: {}", f64::from_bits(num))
                    },
                    _ => println!("Unkown Element Type")
                }
            },
            OpCodes::OpAccessElement => {
                self.pc += 1;
                let reg = program[self.pc];
                let element_id = f64::from_bits(self.registers[program[self.pc] as usize]);
                let mut array_var_id: u64 = 0;
                //Floats seperated in to 8 8-bit chunks. Currenytly set to 64 bits. Might change if porting to a 32 bit system.
                for i in range(0, 8){
                    self.pc += 1;
                    array_var_id = (array_var_id << 8 * (i > 0) as u8 ) | program[self.pc]as u64;
                }
                let array_expr = self.memoryList.get(self.curMemoryId).unwrap().listLookup.get(array_var_id as usize).unwrap();
                match array_expr.0 {
                    VarTypes::FloatType => {
                        let mut num = 0;
                        for j in range(0, 8){
                            num = (num << 8 * (j > 0) as u64) | *array_expr.1.get(((element_id * 8.0) + j as f64) as usize ).unwrap() as u64;
                        }
                        self.registers[reg as usize] = num;
                        println!("Float Value: {}", f64::from_bits(num))
                    },
                    VarTypes::CharType => {
                        let charVal = *array_expr.1.get(element_id as usize).unwrap();
                        self.registers[reg as usize] = charVal;
                        println!("Char Value: {:?}", (charVal as u8) as char );
                    }
                    _ => {println!("Unkown element type")}
                }
                
            },
            _ => println!("No implementation for opcode: {:#?}", opCode)
        }
        
    }
}

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum OpCodes {
    /// OpLoadReg - Operation Code for copy data from register to another
    /// 
    /// First 8 bits - OpCode
    /// 
    /// ------------------
    /// 
    /// Next 4 bits - Source 
    /// 
    /// Next 4 bits - Destination
    OpLoadReg = 0,
    //// OpLoadScalar - Operation Code for loading scalar values into a specified register
    /// 
    /// First 8 bits - OpCode
    /// 
    /// --------------
    /// 
    /// Next 3 bits - Register
    /// 
    /// Last x bits - VarType
    OpLoadScalar,
    /// OpAdd - Operation Code for adding two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 8 bits - OpCode
    ///
    /// -----------------------
    ///  
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - Null
    /// 
    /// Next 3 bits - Second Reg
    OpAdd,
    /// OpSub- Operation Code for subtracting two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 8 bits - OpCode
    /// 
    /// ---------------------
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - Null
    /// 
    /// Next 3 bits - Second Reg
    OpSub,
    /// OpMul- Operation Code for multiplying two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 8 bits - OpCode
    /// 
    /// ---------------------
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - Null
    /// 
    /// Next 3 bits - Second Reg
    OpMul,
    /// OpDiv- Operation Code for dividing two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 8 bits - OpCode
    /// 
    /// ---------------------
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - Null
    /// 
    /// Next 3 bits - Second Reg
    OpDiv,
    //// OpNewVar - Operation Code for adding a variable
    /// 
    /// ---------------------
    /// 
    /// First 8 bits - OpCode
    /// 
    /// ---------------------
    /// 
    /// Next 3 bits - Reg of variable value
    /// 
    /// Last 5 bits - Var Type
    OpNewVar,
    //// OpLoadVar - Operation Load Variable To register
    /// 
    /// ---------------------
    /// 
    /// First 8 bits - OpCode
    /// 
    /// Next 3 bits - Register Num
    /// 
    /// Last 5 bits - Var Type
    OpLoadVar,
    //// OpStartFunc - Operation Code to Start Function Definition
    /// 
    /// First 8 bits - OpCode
    /// 
    /// Next 8 bits - Number of parameters
    OpStartFunc,
    //// OpAddFuncParameter - Operation Code to Add Function Parameter
    /// 
    /// First 8 bits - OpCode
    /// 
    /// Next 8 bits - VarType
    OpAddFuncParameter,
    //// OpEndFunc - Operation Code to Start Function Definition
    /// 
    /// First 8 bits - OpCode
    OpEndFunc,
    //// OpCallFunc - Operation Code to call functiomn
    /// 
    /// First 8 bits - OpCode
    /// 
    /// Next 64 bits - Function Id
    OpCallFunc,
    //// OpEndFunc - Operation Code to Declare End of Param Load
    /// 
    /// First 8 bits - OpCode
    OpEndParamLoad,
    //// OpEndFunc - Operation Code to start array load
    /// 
    /// First 8 bits - OpCode
    /// 
    /// Next 3 bits - Register
    /// 
    /// Last 5 bits - Element Type
    OpLoadArray,
    //// OpEndFunc - Operation Code to Declare End of array load
    /// 
    /// First 8 bits - OpCode
    OpEndArray,
    OpPrint,
    /// OpAccessElement - 
    /// 
    /// First 16 bytes are Variable Ids
    /// Next 16 bytes are Element Index
    OpAccessElement
}

pub struct ASTConverter {
    pub funcIdTable: HashMap<String, u64>,
    // Key is variable name, Value is (Memory Block, VarType, Variable Id)
    pub varLookUp: HashMap<String, (u128, VarTypes, u64)>,
    // Key is variable name, Value is (Memory Block, ElementType, Variable Id)
    pub listLookUp: HashMap<String, (u128, VarTypes, u64)>,
    pub program: Vec<u8>,
    pub curType: VarTypes,
    pub curMemoryBlock: u128,
    pub curNumVarId: u64,
    pub curNumListId: u64,
    pub curFuncId: u64,
    pub free_reg: u8
}

#[derive(FromPrimitive, Debug, Clone, Copy)]
pub enum VarTypes{
    NullType=0,
    FloatType,
    CharType,
    StringType,
    ArrayType,
}


impl ASTConverter {
    pub fn new() -> Self{
        ASTConverter{
            funcIdTable: HashMap::new(),
            varLookUp: HashMap::new(),
            listLookUp: HashMap::new(),
            program: Vec::<u8>::new(),
            curType: VarTypes::FloatType,
            curMemoryBlock: 0,
            curNumVarId: 0,
            curNumListId: 0,
            curFuncId: 0,
            free_reg: 0
        }
    }

    pub fn ConvertExprToByteCode(&mut self, expr: ExprAST) -> Option<u8> {
        match expr {
            ExprAST::NumberExpr(num) => {
                let mut byteCode: u8 = 0;
                //Loads Op Code
                byteCode = byteCode | ((OpCodes::OpLoadScalar as u8) );
                self.program.push(byteCode);

                let mut byteCode: u8 = 0;
                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                //Load register into bytecode
                byteCode = byteCode | ((register as u8) << 5) | VarTypes::FloatType as u8;

                // Adds to the program list
                self.program.push(byteCode);

                let floatBits = f64::to_bits(num);
                for i in range(0, 8){
                    let shift: u8 = 56 - 8*i;
                    self.program.push( ((floatBits >> shift) & 0xFF) as u8);    
                }

                self.curType = VarTypes::FloatType;
                return Some(register); 
            },
            ExprAST::CharExpr(val) => {
                let mut byteCode: u8 = 0;
                //Loads Op Code
                byteCode = byteCode | ((OpCodes::OpLoadScalar as u8) );
                self.program.push(byteCode);

                let mut byteCode: u8 = 0;
                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                //Load register into bytecode
                byteCode = byteCode | ((register as u8) << 5) | VarTypes::CharType as u8;

                // Adds to the program list
                self.program.push(byteCode);

                let charBits = val.as_bytes()[0];
                self.program.push(charBits);
                self.curType = VarTypes::CharType;
                return Some(register); 
            },
            ExprAST::StringExpr(val) => {
                let mut bytecode: u8 = 0;

                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                // bytecode = bytecode | ((OpCodes::OpLoadArray as u16) << 12) | ((register as u16) << 9) | VarTypes::CharType as u16;
                bytecode = bytecode | (OpCodes::OpLoadArray as u8);
                self.program.push(bytecode);
                bytecode = 0;
                bytecode = bytecode | (register << 5) | VarTypes::CharType as u8;

                // Adds to the program list
                self.program.push(bytecode);
                
                let bytes_arr = val.as_bytes();
                for i in (0..bytes_arr.len()){
                    self.program.push(bytes_arr[i] as u8);
                }

                bytecode = 0 | (OpCodes::OpEndArray as u8);
                self.program.push(bytecode);
                self.curType = VarTypes::StringType;

                return Some(register)
            },
            ExprAST::ListExpr(listOfExpr) => {
                let mut bytecode: u8 = 0;

                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                // bytecode = bytecode | ((OpCodes::OpLoadArray as u16) << 12) | ((register as u16) << 9) | VarTypes::CharType as u16;
                bytecode = bytecode | (OpCodes::OpLoadArray as u8);
                self.program.push(bytecode);
                bytecode = 0;
                bytecode = bytecode | (register << 5); 
                
                let elementType: VarTypes;
                match listOfExpr.get(0).unwrap() {
                    ExprAST::NumberExpr(_) => {
                        elementType = VarTypes::FloatType;
                    },
                    ExprAST::CharExpr(_) => {
                        elementType = VarTypes::CharType;
                    },
                    ExprAST::StringExpr(_) => {
                        elementType = VarTypes::StringType;
                    },
                    _ => {elementType = VarTypes::NullType}
                }

                bytecode = bytecode | elementType as u8;
                

                // Adds to the program list
                self.program.push(bytecode);
                
                for i in (0..listOfExpr.len()){
                    match elementType {
                        VarTypes::FloatType => {
                            if let ExprAST::NumberExpr(val) = listOfExpr[i].clone() {
                                let floatBits = f64::to_bits(val);
                                for i in range(0, 8){
                                    let shift: u8 = 56 - 8*i;
                                    self.program.push( ((floatBits >> shift) & 0xFF) as u8);    
                                }
                            }
                        },
                        VarTypes::CharType => {
                            if let ExprAST::CharExpr(val) = listOfExpr[i].clone() {
                                let charBits = val.as_bytes()[0];
                                self.program.push(charBits);
                            }
                        },
                        _ => {panic!("Unimplemented element type")}
                    }
                }

                bytecode = 0 | (OpCodes::OpEndArray as u8);
                self.program.push(bytecode);
                self.curType = VarTypes::FloatType;

                return Some(register)

            },
            ExprAST::VariableExpr(name) => {
                let mut byteCode: u8 = 0;
                let varIdTuple = self.varLookUp.get(&name).unwrap().clone();
                let varId = varIdTuple.2;
                byteCode = byteCode | OpCodes::OpLoadVar as u8;
                self.program.push(byteCode);
                byteCode = 0;
                
                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                byteCode = byteCode | ((register as u8) << 5) | varIdTuple.1 as u8;
                self.program.push(byteCode);

                for i in range(0, 8){
                    let shift: u8 = 56 - 8*i;
                    self.program.push( ((varId >> shift) & 0xFF) as u8);    
                }

                return Some(register);
            }
            ExprAST::VariableAssignExpr { varObject, value } => {
                let mut byteCode: u8 = 0;
                let register_val: u8;
                if let ExprAST::VariableHeader { name, typeName } = *varObject.to_owned() {
                    register_val = self.ConvertExprToByteCode(*value).expect("Can not compile variable value");
                    let isArray = typeName.contains("[]");
                    let typeName_cleaned = typeName.replace("[]", "");
                    let mut valVarType = match typeName_cleaned.as_str() {
                        "number" => VarTypes::FloatType,
                        "char" => VarTypes::CharType,
                        "string" => VarTypes::StringType,
                        _ => panic!("Can not compile variable type")
                    };
                    if isArray {
                        valVarType = VarTypes::ArrayType;
                        self.listLookUp.insert(name, (self.curMemoryBlock, valVarType, self.curNumListId));
                        self.curNumListId += 1;
                    }else {
                        self.varLookUp.insert(name, (self.curMemoryBlock, valVarType, self.curNumVarId));
                        self.curNumVarId += 1;
                    }
                    byteCode = byteCode | OpCodes::OpNewVar as u8;
                    self.program.push(byteCode);
                    byteCode =  0;
                    byteCode = byteCode | ((register_val as u8) << 5)  | valVarType as u8;
                    self.program.push(byteCode);
                    return Some(register_val);
                }
                return None;
            }
            ExprAST::BinaryExpr { op, lhs, rhs, opChar: _ } => {
                // Gets register for the left hand side
                let reg1 = self.ConvertExprToByteCode(*lhs).unwrap();
                let mut byteCode : u8 = 0;
                // Gets right op code for operation
                let opCode : u8 = match op {
                    Token::Plus => OpCodes::OpAdd as u8,
                    Token::Minus => OpCodes::OpSub as u8,
                    Token::Multiply => OpCodes::OpMul as u8,
                    Token::Divide => OpCodes::OpDiv as u8,
                    _ => 0 as u8
                };

                match *rhs {
                    ExprAST::NumberExpr(_) | ExprAST::CharExpr(_) | ExprAST::StringExpr(_) | ExprAST::ElementAccess { array_name: _, element_index: _ } => {
                        let varTypeOpr1 = self.curType;
                        // Gets register for the right hand side
                        let reg2 = self.ConvertExprToByteCode(*rhs).unwrap();
                        let varTypeOpr2 = self.curType;
                        if varTypeOpr1 as u16 != varTypeOpr2 as u16 {
                            panic!("Operands must match type");
                        }
                        // Loads opCode and register into bytecode
                        byteCode = byteCode | opCode;
                        self.program.push(byteCode);
                        byteCode = 0;
                        byteCode = ( (reg1 as u8) << 5);
                        // Loads register to bytecode
                        byteCode = byteCode | (reg2 as u8);
                        // Pushed bytecode to program list
                        self.program.push(byteCode);
                        return Some(reg1);
                    },
                    ExprAST::BinaryExpr { op, lhs, rhs, opChar } => {
                        let binExpr = ExprAST::BinaryExpr { op: op, lhs: lhs, rhs: rhs, opChar: opChar };
                        let binExprReg = self.ConvertExprToByteCode(binExpr).unwrap();
                        
                        // Loads opCode and register into bytecode
                        byteCode = byteCode | opCode;
                        self.program.push(byteCode);
                        byteCode = 0;
                        byteCode = ( (reg1 as u8) << 5);
                        // Loads register to bytecode
                        byteCode = byteCode | (binExprReg as u8);
                        // Pushed bytecode to program list
                        self.program.push(byteCode);

                        // byteCode = byteCode | binExprReg;
                        // self.program.push(byteCode);
                        return Some(reg1);
                    },

                    // dfff
                    _ => {return None;}
                }
            },
            ExprAST::FuncExpr { name, args, body } => {
                let mut bytecode: u8 = 0;
                let oldMemoryBlockId = self.curMemoryBlock;
                let oldNumVarId = self.curNumVarId;
                let oldVarLookup = self.varLookUp.clone();
                self.curNumVarId = 0;
                self.varLookUp = HashMap::new();
                self.curMemoryBlock = self.curMemoryBlock + 1;
                // Insert Function Name with funcID
                self.funcIdTable.insert(name, self.curFuncId);
                self.curFuncId = self.curFuncId + 1;
                let param_count = (args.len() as u8);
                // bytecode = bytecode  | (OpCodes::OpStartFunc as u16) << 12 | param_count;
                bytecode = OpCodes::OpStartFunc as u8;
                self.program.push(bytecode);
                self.program.push(param_count);

                // Loop through arguments and load them in to the function def
                for param in args{
                    if let ExprAST::VariableHeader { name, typeName } = param {
                        let varVaribleType = match typeName.as_str() {
                            "number" => VarTypes::FloatType,
                            "string" => VarTypes::CharType,
                            _ => panic!("Can not compile type")
                            };
                        bytecode = 0; 
                        bytecode = bytecode | (OpCodes::OpAddFuncParameter as u8);
                        self.program.push(bytecode);
                        bytecode = 0 | (varVaribleType as u8);
                        self.program.push(bytecode);
                        self.varLookUp.insert(name, (self.curMemoryBlock, varVaribleType, self.curNumVarId));
                        self.curNumVarId += 1;
                    }
                }

                // Parse through body
                let mut lastReg: u8 = 0;
                for bodyExpr in body{
                    lastReg = self.ConvertExprToByteCode(bodyExpr).unwrap();
                }
                // Add FuncEnd part
                bytecode = 0 | (OpCodes::OpEndFunc as u8);
                self.program.push(bytecode);
                self.curMemoryBlock = oldMemoryBlockId;
                self.curNumVarId = oldNumVarId;
                self.varLookUp = oldVarLookup.clone();
                return Some(lastReg);
            },
            ExprAST::CallExpr { func_name, parameters } => {
                //Grabs function Id and loads it
                let funcIdOption = self.funcIdTable.get(&func_name);
                let mut bytecode: u8;
                if funcIdOption.is_none() {
                    println!("Function {:#?} Not found", func_name.as_str());
                }
                bytecode = 0 | (OpCodes::OpCallFunc as u8);
                self.program.push(bytecode);
                let funcId = (*funcIdOption.unwrap());
                
                for i in range(0, 8){
                    let shift: u8 = 56 - 8*i;
                    self.program.push( ((funcId >> shift) & 0xFF) as u8);    
                }

                //Loads function paramters
                let mut param_reg :Option<u8> = Some(self.free_reg);
                for param in parameters {
                    param_reg = self.ConvertExprToByteCode(param);
                    bytecode = 0 | (OpCodes::OpNewVar as u8);
                    self.program.push(bytecode);
                    bytecode = 0 | (param_reg.unwrap()) << 5 | (self.curType as u8);
                    self.program.push(bytecode);
                }
                bytecode = 0 |  (OpCodes::OpEndParamLoad as u8);
                self.program.push(bytecode);
                return param_reg;
            },
            ExprAST::ElementAccess { array_name, element_index } => {
                let array_obj = self.listLookUp.get(&array_name).unwrap();
                self.curType = array_obj.1;
                let mut array_id_index : [u8;8] = [0; 8];
                
                for i in range(0, 8){
                    let shift: u8 = 56 - 8*i;
                    array_id_index[i as usize] = ((array_obj.2 >> shift) & 0xFF) as u8;    
                }
                let param_reg :Option<u8> = self.ConvertExprToByteCode(*element_index);
                self.program.push( 0 | OpCodes::OpAccessElement as u8 );
                self.program.push(param_reg.unwrap());
                for i in range(0, 8){
                    self.program.push(array_id_index[i]);
                }
                return param_reg;
            }
            _ => {println!("Could not convert expression to bytecode"); return None;}
        }
    }

}

