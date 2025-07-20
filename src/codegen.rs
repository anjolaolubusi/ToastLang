#![allow(non_snake_case)]
#![allow(unused_parens)]
use std::{array, collections::HashMap, hash::Hash, u16};

use crate::parser::ExprAST;
use crate::lexer::Token;
use num::{self, range};
use num_derive::{self, FromPrimitive};
use multimap::MultiMap;

// Holds memory of function
#[derive(Debug, Clone)]
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
    pub registers: [u64; 9],
    pub pc: usize,
    pub cond: u8,
    pub memoryList: Vec<MemoryBlock>,
    /// Key is function Id, Value is (Start pc value, list of param types)
    pub funcList: MultiMap<usize, (usize, Vec<VarTypes>)>,
    pub curMemoryId: usize,
    pub curFunctionId: usize,
    pub curType: VarTypes,
}

impl VMCore {
    pub fn getSystemFunctions() -> MultiMap<usize, (usize, Vec<VarTypes>)> {
        let mut systemFunctions : MultiMap<usize, (usize, Vec<VarTypes>)> = MultiMap::new();

        systemFunctions.insert(SystemFunctions::printFunction as usize, (0, [VarTypes::FloatType].to_vec() ));
        systemFunctions.insert(SystemFunctions::printFunction as usize, (0, [VarTypes::CharType].to_vec() ));
        systemFunctions.insert(SystemFunctions::printFunction as usize, (0, [VarTypes::ArrayType].to_vec() ));


        return systemFunctions.clone();
    }

    pub fn get64BitVal(&mut self,  program: &Vec<u8>) -> u64 {
        let mut num: u64 = 0;
        //Floats seperated in to 8 8 bit chunks. Currenytly set to 64 bits. Might change if porting to a 32 bit system.
        for i in range(0, 8){
            self.pc += 1;
            num = (num << 8 * (i > 0) as u8 ) | program[self.pc]as u64;
        }
        return num;
    }

    pub fn new() -> Self{
        let mut vm = VMCore {
            registers: [0; 9],
            pc: 0,
            cond: 0,
            memoryList: Vec::<MemoryBlock>::new(),
            funcList: VMCore::getSystemFunctions().clone(),
            curMemoryId: 0,
            curFunctionId: 1,
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

    pub fn printScalar(&self, scalarVal: u64, scalarType: VarTypes) {
        match scalarType {
            VarTypes::FloatType => {
                print!("{:?}", f64::from_bits(scalarVal));
            },
            VarTypes::CharType => {
                print!("{:?}", (scalarVal as u8) as char);
            },
            _ => println!("Unimplemented type")
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
                        let num: u64 = self.get64BitVal(program);
                        self.registers[reg as usize] = num;
                        // println!("Float Value: {}", f64::from_bits(self.registers[reg as usize]))
                    },
                    VarTypes::CharType => {
                        self.curType = VarTypes::CharType;
                        // Only grabs the first 5 bits (31 is all first 5 bits as one) of the bytecode since that is where the current value type is
                        let reg = (byteCode >> 5) & 7;
                        self.pc = self.pc + 1;
                        let charBit = program[self.pc];
                        self.registers[reg as usize] = charBit as u64;
                        // println!("Char Value: {:?}", (self.registers[reg as usize] as u8) as char);
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
                        println!("Answer: {}", f64::from_bits(self.registers[reg1 as usize]));
                        self.registers[8 as usize] = self.registers[reg1 as usize];
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
                let varId: u64 = self.get64BitVal(program);
                self.registers[reg as usize] = self.memoryList.get(self.curMemoryId).unwrap().variableLookup.get(&varId).unwrap().1;
                self.registers[8 as usize] = self.registers[reg as usize];
                match typeVal {
                    VarTypes::FloatType => {
                        println!("Variable Value: {}", f64::from_bits(self.registers[reg as usize]))
                    },
                    VarTypes::CharType => {
                        println!("Char Value: {:?}", (self.registers[reg as usize] as u8) as char);
                    },
                    VarTypes::ArrayType => {
                        let arr_data = self.memoryList.get(self.curMemoryId).unwrap().listLookup.get(self.registers[reg as usize] as usize).unwrap().clone();
                        match arr_data.0 {
                            VarTypes::CharType => {
                                let string_vec: Vec<u16> = arr_data.1.into_iter().map(|x| x as u16).collect();
                                println!("String Value: {:?}", String::from_utf16(string_vec.as_slice()).unwrap())
                            }
                            _ => {println!("Unimplemented type")}
                        }
                    }
                    _ => {println!("Unkown variable type")}
                }
            },
            OpCodes::OpStartFunc => {
                //Move from opCode
                self.pc += 1;
                //Grab parameter number (Max 255)
                let param_num = program[self.pc];
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
                //TODO: Replace 1 with function that counts system functions
                let function_id = self.get64BitVal(program);
                let func_data = self.funcList.get_vec(&(function_id as usize)).unwrap_or_else(|| {panic!("Unkown function")}).clone();
                self.pc += 1;
                self.memoryList.push(MemoryBlock::new());
                self.curMemoryId += 1;
                while program[self.pc] != OpCodes::OpEndParamLoad as u8{
                    self.ConsumeByteCode(program, program[self.pc]);
                    self.pc += 1;
                }
                let oldPC = self.pc;
                self.pc = func_data[0].0;
                if func_data[0].0 != 0 {
                    while program[self.pc]  != (OpCodes::OpEndFunc as u8) {
                        self.ConsumeByteCode(program, program[self.pc]);
                        self.pc += 1;
                    }
                }else{
                    let systemFunction : SystemFunctions = num::FromPrimitive::from_usize(func_data[0].0).unwrap();
                    match systemFunction {
                        SystemFunctions::printFunction => {
                            let firstParam = self.memoryList.get(self.curMemoryId).unwrap().variableLookup.get(&0).unwrap();
                            self.registers[8 as usize] = firstParam.1;
                            if func_data.contains( &(0 as usize, [firstParam.0].to_vec()) ){
                                if firstParam.0 == VarTypes::ArrayType {
                                    let arr = self.memoryList.get(self.curMemoryId).unwrap().listLookup.get(firstParam.1 as usize).unwrap().clone();
                                    match arr.0 {
                                        VarTypes::CharType => {
                                            let string_vec: Vec<u16> = arr.1.into_iter().map(|x| x as u16).collect();
                                            println!("{:?}", String::from_utf16(string_vec.as_slice()).unwrap())
                                        },
                                        VarTypes::FloatType => {
                                            print!("[");
                                            for ele in arr.1 {
                                                self.printScalar(ele, arr.0);
                                                print!(",")
                                            }
                                            print!("]\n");
                                        }
                                        _ => {println!("Unimplemented variable type");}
                                    }
                                }else{
                                    self.printScalar(firstParam.1, firstParam.0);
                                    print!("\n");
                                }
                            }
                        }
                    }
                }
                self.pc = oldPC;
                self.memoryList.pop();
                self.curMemoryId -= 1;

            },
            OpCodes::OpLoadArray => {
                self.pc += 1;
                byteCode = program[self.pc];
                self.curType = VarTypes::ArrayType;
                let reg = (byteCode >> 5) & 7;
                let elementType: VarTypes = num::FromPrimitive::from_u8(byteCode & 31).unwrap();
                let mut array_vec = Vec::<u8>::new();
                self.pc += 1;
                let oldPC = self.pc;
                while (program[self.pc] != OpCodes::OpEndArray as u8) {
                    array_vec.push(program[self.pc]);
                    self.pc += 1;
                }
                
                match elementType {
                    VarTypes::CharType => {
                        self.memoryList.get_mut(self.curMemoryId).unwrap().listLookup.push((elementType, array_vec.clone().into_iter().map(|x| x as u64).collect()));
                        self.curType = VarTypes::CharType;
                    },
                    VarTypes::FloatType => {
                        self.curType = VarTypes::FloatType;
                        let mut num: u64 = 0;
                        let mut new_arr : Vec<u64> = Vec::new();
                        if (array_vec.len() > 0){
                            for i in range(0, (array_vec.len())/8 ){
                                num = 0;
                                for j in range(0, 8){
                                    num = (num << 8 * (j > 0) as u64) | array_vec[ (i * 8) + j] as u64;
                                }
                                new_arr.push(num);
                            }
                        }
                        self.memoryList.get_mut(self.curMemoryId).unwrap().listLookup.push((elementType, new_arr));
                    },
                    VarTypes::ArrayType => {
                        self.pc = oldPC;
                        let mut new_arr : Vec<u64> = Vec::new();
                        while (program[self.pc] != OpCodes::OpEndArray as u8) { 
                            match num::FromPrimitive::from_u8(program[self.pc]).unwrap() {
                                OpCodes::OpLoadMultiDimensionalArrayElement => {
                                    self.pc += 1;
                                    self.ConsumeByteCode(program, program[self.pc]);
                                },
                                OpCodes::OpEndMultiDimensionalArrayElement => {
                                    new_arr.push( (self.memoryList.get_mut(self.curMemoryId).unwrap().listLookup.len()-1) as u64 );
                                    if (self.pc+1 > program.len()){
                                        break;
                                    }
                                    if (program[self.pc+1] != OpCodes::OpLoadMultiDimensionalArrayElement as u8){
                                        self.pc += 1;
                                        break;
                                    }
                                }
                                _ => {panic!("Unimplemented arm for Array match")}
                            }
                            self.pc += 1;
                        }
                        self.memoryList.get_mut(self.curMemoryId).unwrap().listLookup.push((VarTypes::ArrayRef, new_arr));

                    }
                    _ => println!("Unkown Element Type")
                }
                self.registers[reg as usize] = (self.memoryList.get(self.curMemoryId).unwrap().listLookup.len()-1) as u64;
            },
            OpCodes::OpAccessArray => {
                    println!("{:?}", program);
                    let array_id: u64 = self.get64BitVal(program);
                    let mut elements_indexes = Vec::<u64>::new();
                    self.pc += 1;
                    loop {
                        let bb : OpCodes = num::FromPrimitive::from_u8(program[self.pc]).unwrap(); 
                        if [OpCodes::OpAccessElementBegin as u8, OpCodes::OpAccessElementEnd as u8].contains(&program[self.pc]) == false {
                            panic!("Expected OpAccessElementBegin or OpAccessElementEnd, found: {}", program[self.pc]);
                        }
                        if program[self.pc] == OpCodes::OpAccessElementBegin as u8 {
                            self.pc += 1;
                            self.ConsumeByteCode(program, program[self.pc]);
                            self.pc += 1;
                            elements_indexes.push(self.registers[program[self.pc] as usize]);
                            self.pc += 1;
                        } else if program[self.pc] == OpCodes::OpAccessElementEnd as u8 {
                            if program.len() <= self.pc + 1 {
                                break;
                            }
                            
                            if program[self.pc + 1] != OpCodes::OpAccessElementBegin as u8 {
                                break;
                            }
                            self.pc += 1;
                        }
                    }
                    let arr = self.memoryList.get(self.curMemoryId).unwrap().listLookup.get(array_id as usize).unwrap();

                    if elements_indexes.len() > 1 {
                        let element_index: u64 = *elements_indexes.last().unwrap();
                        elements_indexes.pop();
                        let mut cur_arr = arr.1.clone();
                        for i in 0 .. elements_indexes.len() {
                            let arr_ref_index = f64::from_bits(*elements_indexes.get(i).unwrap());
                            let list_index =    *cur_arr.get(arr_ref_index as usize).unwrap() as usize;
                            let cur_memory = self.memoryList.get(self.curMemoryId).unwrap();
                            let cur_list: Option<&(VarTypes, Vec<u64>)> = cur_memory.listLookup.get(list_index);
                            cur_arr = cur_list.unwrap().1.clone();
                        }
                        let num = cur_arr.get(f64::from_bits(element_index) as usize).unwrap();
                        println!("Float Value: {}", f64::from_bits(*num));
                        self.registers[8 as usize] = *num;
                    } else {
                        match arr.0 {
                            VarTypes::FloatType => {
                                let index = f64::from_bits(*elements_indexes.first().unwrap());
                                let num = arr.1.get(index as usize).unwrap();
                                println!("Float Value: {}", f64::from_bits(*num));
                                self.registers[8 as usize] = *num;
                            },
                            _ => {panic!("Unimplemented array type")}
                        }
                    }
            },
            OpCodes::OpCopyVarToNewMemoryBlock => {
                self.pc += 1;
                let var_id: u64 = self.get64BitVal(program);

                let memoryList = &mut self.memoryList;

                let varTuple = *memoryList.get(self.curMemoryId - 1).unwrap().variableLookup.get(&var_id).unwrap();
                let mut newVarId = memoryList.get(self.curMemoryId).unwrap().variableLookup.len();
                if newVarId > 0 {
                    newVarId -= 1;
                }

                match varTuple.0 {
                    VarTypes::ArrayType => {
                        let var_list: (VarTypes, Vec<u64>) = memoryList.get(self.curMemoryId - 1).unwrap().listLookup.get(varTuple.1 as usize).unwrap().clone();
                        let mut newListId: usize = memoryList.get(self.curMemoryId).unwrap().listLookup.len();
                        if newListId > 0 {
                            newListId -= 1;
                        }
                        memoryList.get_mut(self.curMemoryId).unwrap().listLookup.insert(newListId, var_list.clone());
                        memoryList.get_mut(self.curMemoryId).unwrap().variableLookup.insert(newVarId as u64, (varTuple.0, newListId as u64));
                    }
                    _ => {
                        memoryList.get_mut(self.curMemoryId).unwrap().variableLookup.insert(newVarId as u64, varTuple);
                    }
                }
            }
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
    OpAccessArray,
    OpAccessElementBegin,
    OpAccessElementEnd,
    OpCopyVarToNewMemoryBlock,
    OpLoadMultiDimensionalArrayElement,
    OpEndMultiDimensionalArrayElement,
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

#[derive(FromPrimitive, Debug, Clone, Copy, PartialEq)]
pub enum VarTypes{
    NullType=0,
    FloatType,
    CharType,
    StringType,
    ArrayType,
    ArrayRef
}

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum SystemFunctions{
    printFunction=0,
}

impl ASTConverter {
    pub fn new() -> Self{
        let mut systemFuncTable: HashMap<String, u64> = HashMap::new();
        systemFuncTable.insert("print".to_string(), SystemFunctions::printFunction as u64);

        ASTConverter{
            funcIdTable: systemFuncTable.clone(),
            varLookUp: HashMap::new(),
            listLookUp: HashMap::new(),
            program: Vec::<u8>::new(),
            curType: VarTypes::NullType,
            curMemoryBlock: 0,
            curNumVarId: 0,
            curNumListId: 0,
            curFuncId: 1,
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
                self.curType = VarTypes::ArrayType;

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
                if (listOfExpr.len() < 1) {
                    elementType = self.curType;
                } else {
                    match listOfExpr.get(0).unwrap() {
                        ExprAST::NumberExpr(_) => {
                            elementType = VarTypes::FloatType;
                        },
                        ExprAST::CharExpr(_) => {
                            elementType = VarTypes::CharType;
                        },
                        ExprAST::StringExpr(_) => {
                            elementType = VarTypes::CharType;
                        },
                        ExprAST::ListExpr(_) => {
                            elementType = VarTypes::ArrayType;
                        },
                        _ => {elementType = self.curType}
                    }
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
                        VarTypes::ArrayType => {
                            self.program.push(OpCodes::OpLoadMultiDimensionalArrayElement as u8);
                            self.ConvertExprToByteCode(listOfExpr[i].clone());
                            self.program.push(OpCodes::OpEndMultiDimensionalArrayElement as u8);
                            self.curNumListId += 1;
                        }
                        _ => {panic!("Unimplemented element type")}
                    }
                }

                bytecode = 0 | (OpCodes::OpEndArray as u8);
                self.program.push(bytecode);
                self.curType = VarTypes::ArrayType;

                return Some(register)

            },
            ExprAST::VariableExpr(name) => {
                let mut byteCode: u8 = 0;
                let varIdTuple = self.varLookUp.get(&name).unwrap().clone();

                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                let mut varId: u64 = 0;

                if varIdTuple.0 != self.curMemoryBlock {
                    panic!("Variable does not exist in memory block");
                }
                varId = varIdTuple.2;
                byteCode = byteCode | OpCodes::OpLoadVar as u8;
                self.program.push(byteCode);
                byteCode = 0;

                byteCode = byteCode | ((register as u8) << 5) | varIdTuple.1 as u8;
                self.program.push(byteCode);

                for i in range(0, 8){
                    let shift: u8 = 56 - 8*i;
                    self.program.push( ((varId >> shift) & 0xFF) as u8);    
                }

                self.curType = varIdTuple.1;

                return Some(register);
            }
            ExprAST::VariableAssignExpr { varObject, value } => {
                let mut byteCode: u8 = 0;
                let register_val: u8;
                if let ExprAST::VariableHeader { name, typeName } = *varObject.to_owned() {
                    let array_dim_count = typeName.split("[]").count() - 1;
                    let typeName_cleaned = typeName.replace("[]", "");
                    let mut valVarType = match typeName_cleaned.as_str() {
                        "number" => VarTypes::FloatType,
                        "char" => VarTypes::CharType,
                        "string" => VarTypes::CharType,
                        _ => panic!("Can not compile variable type")
                    };
                    let isArray = (array_dim_count > 0);
                    self.curType = valVarType;
                    register_val = self.ConvertExprToByteCode(*value).expect("Can not compile variable value");
                    if isArray || typeName_cleaned.as_str() == "string" {
                        self.listLookUp.insert(name.clone(), (self.curMemoryBlock, valVarType, self.curNumListId));
                        self.curNumListId += 1;
                        valVarType = VarTypes::ArrayType;
                    }
                        self.varLookUp.insert(name, (self.curMemoryBlock, valVarType, self.curNumVarId));
                        self.curNumVarId += 1;

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
                    ExprAST::NumberExpr(_) | ExprAST::CharExpr(_) | ExprAST::StringExpr(_) | ExprAST::ElementAccess { array_name: _, element_indexes: _ } => {
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
                    // Add check for variable
                    // If variable, copy variable and append to memory block
                    match param {
                        ExprAST::VariableExpr(var) => {
                            bytecode = 0 | (OpCodes::OpCopyVarToNewMemoryBlock as u8);
                            self.program.push(bytecode);
                            let varIdTuple = self.varLookUp.get(&var).unwrap().clone();
                            let varId = varIdTuple.2;
                            bytecode = 0;
                            bytecode = bytecode | varIdTuple.1 as u8;
                            self.program.push(bytecode);
            
                            for i in range(0, 8){
                                let shift: u8 = 56 - 8*i;
                                self.program.push( ((varId >> shift) & 0xFF) as u8);    
                            }

                        },
                        _ => {
                            param_reg = self.ConvertExprToByteCode(param);
                            bytecode = 0 | (OpCodes::OpNewVar as u8);
                            self.program.push(bytecode);
                            bytecode = 0 | (param_reg.unwrap()) << 5 | (self.curType as u8);
                            self.program.push(bytecode);
                        }
                    }
                }
                bytecode = 0 |  (OpCodes::OpEndParamLoad as u8);
                self.curMemoryBlock += 1;
                self.program.push(bytecode);
                self.curMemoryBlock -= 1;
                return param_reg;
            },
            ExprAST::ElementAccess { array_name, element_indexes: element_index } => {
                let array_obj = self.listLookUp.get(&array_name).unwrap();
                self.curType = array_obj.1;
                let array_id = array_obj.2;
                let mut param_reg :Option<u8> = None;

                self.program.push(0 | OpCodes::OpAccessArray as u8);

                for i in range(0, 8){
                    let shift: u8 = 56 - 8*i;
                    self.program.push((( array_id >> shift) & 0xFF) as u8);
                }


                for ele_index in element_index {
                    self.program.push( 0 | OpCodes::OpAccessElementBegin as u8 );
                    param_reg = self.ConvertExprToByteCode(*ele_index);
                    self.program.push(param_reg.unwrap());
                    self.program.push( 0 | OpCodes::OpAccessElementEnd as u8 );
                }
                
                return param_reg;
            }
            _ => {println!("Could not convert expression to bytecode"); return None;}
        }
    }

}

mod tests {
    use crate::parser::{ExprAST, Parser};
    use crate::codegen::{ASTConverter, VMCore, VarTypes};

    #[test]
    fn compileBasicEquation(){
        let source = "1 + 2";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [1, 1, 63, 240, 0, 0, 0, 0, 0, 0, 1, 33, 64, 0, 0, 0, 0, 0, 0, 0, 2, 1].to_vec();
        assert_eq!(ast_converter.program, true_val);
    }

    #[test]
    fn compileAndRunBasicEquation(){
        let source = "1 + 2";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val_program: Vec<u8> = [1, 1, 63, 240, 0, 0, 0, 0, 0, 0, 1, 33, 64, 0, 0, 0, 0, 0, 0, 0, 2, 1].to_vec();
        assert_eq!(&ast_converter.program, &true_val_program);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        assert_eq!(f64::from_bits(toast_vm.registers[8 as usize]), (3 as f64));

    }

    #[test]
    fn compileString(){
        let source = "\"Hello World\"";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 2, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 14].to_vec();
        assert_eq!(ast_converter.program, true_val);
        //assert_eq!(f64::from_bits(toast_vm.registers[7 as usize]), (3 as f64));
    }

    #[test]
    fn compileAndRunString(){
        let source = "\"Hello World\"";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 2, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 14].to_vec();
        assert_eq!(ast_converter.program, true_val);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        let curMemoryBlock = toast_vm.memoryList.first();
        assert_eq!(curMemoryBlock.is_some(), true);
        let string_u8_vec: Vec<u8> = curMemoryBlock.unwrap().listLookup.first().unwrap().1.clone().into_iter().map(|x| x as u8).collect();
        assert_eq!(String::from_utf8(string_u8_vec).unwrap(), "Hello World".to_string());
    }

    #[test]
    fn compileArray(){
        let source = "let arr: number[] = [1,2,3,4]";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 1, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 64, 16, 0, 0, 0, 0, 0, 0, 14, 6, 4].to_vec();
        assert_eq!(ast_converter.program, true_val);
    }

    #[test]
    fn compileAndRunArray(){
        let source = "let arr: number[] = [1,2,3,4]";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 1, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 64, 16, 0, 0, 0, 0, 0, 0, 14, 6, 4].to_vec();
        assert_eq!(ast_converter.program, true_val);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        let curMemoryBlock = toast_vm.memoryList.first();
        assert_eq!(curMemoryBlock.is_some(), true);
        assert_eq!(curMemoryBlock.unwrap().variableLookup.get(&(0 as u64)).unwrap().0, VarTypes::ArrayType);
        assert_eq!(curMemoryBlock.unwrap().listLookup.first().unwrap().0, VarTypes::FloatType);
        let float_vec: Vec<f64> = curMemoryBlock.unwrap().listLookup.first().unwrap().1.clone().into_iter().map(|x| f64::from_bits(x)).collect();
        assert_eq!(float_vec, [1.0,2.0,3.0,4.0].to_vec());
    }

    #[test]
    fn compileFunction(){
        let source = "def foo(a: number):\na*100\nend";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [8, 1, 9, 1, 7, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 33, 64, 89, 0, 0, 0, 0, 0, 0, 4, 1, 10].to_vec();
        assert_eq!(ast_converter.program, true_val);
    }

    #[test]
    fn compileAndRunFunction(){
        let source = "def foo(a: number):\na*100\nend";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [8, 1, 9, 1, 7, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 33, 64, 89, 0, 0, 0, 0, 0, 0, 4, 1, 10].to_vec();
        assert_eq!(ast_converter.program, true_val);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        println!("{:?}", toast_vm);
        let func_len = toast_vm.funcList.keys().len();
        assert_eq!(toast_vm.funcList.get(&(func_len-1)).unwrap().1, [VarTypes::FloatType].to_vec());
    }


    #[test]
    fn compileFunctionCall(){
        let source = "def foo(a: number):\na*100\nend\nfoo(32)";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [8, 1, 9, 1, 7, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 33, 64, 89, 0, 0, 0, 0, 0, 0, 4, 1, 10, 11, 0, 0, 0, 0, 0, 0, 0, 1, 1, 65, 64, 64, 0, 0, 0, 0, 0, 0, 6, 65, 12].to_vec();
        assert_eq!(ast_converter.program, true_val);
    }

    #[test]
    fn compileAndRunFunctionCall(){
        let source = "def foo(a: number):\na*100\nend\nfoo(32)";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [8, 1, 9, 1, 7, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 33, 64, 89, 0, 0, 0, 0, 0, 0, 4, 1, 10, 11, 0, 0, 0, 0, 0, 0, 0, 1, 1, 65, 64, 64, 0, 0, 0, 0, 0, 0, 6, 65, 12].to_vec();
        assert_eq!(ast_converter.program, true_val);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        let func_len = toast_vm.funcList.keys().len();
        assert_eq!(toast_vm.funcList.get(&(func_len-1)).unwrap().1, [VarTypes::FloatType].to_vec());
        assert_eq!(f64::from_bits(toast_vm.registers[8]), 3200 as f64);
    }


    #[test]
    fn compileDeclareVarible(){
        let source = "let x: number = 67";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        println!("{:?}", ast_converter.program);
        let true_val: Vec<u8> = [1, 1, 64, 80, 192, 0, 0, 0, 0, 0, 6, 1].to_vec();
        assert_eq!(ast_converter.program, true_val);
    }

    #[test]
    fn compileAndDeclareVarible(){
        let source = "let x: number = 67";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [1, 1, 64, 80, 192, 0, 0, 0, 0, 0, 6, 1].to_vec();
        assert_eq!(ast_converter.program, true_val);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        let toast_vm_var = toast_vm.memoryList.first().unwrap().variableLookup.get(&(0 as u64)).unwrap();
        println!("{:?}", toast_vm);
        assert_eq!(toast_vm_var.0, VarTypes::FloatType);
        assert_eq!(f64::from_bits(toast_vm_var.1), 67 as f64);
    }

    #[test]
    fn compileElementAccessSingleDimensionalArray(){
        let source = "let arr: number[] = [1,2,3] arr[0]";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 1, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 14, 6, 4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 17, 1, 33, 0, 0, 0, 0, 0, 0, 0, 0, 1, 18].to_vec();
        assert_eq!(ast_converter.program, true_val);
    }

    #[test]
    fn compileAndRunElementAccessSingleDimensionalArray(){
        let source = "let arr: number[] = [1,2,3] arr[0]";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 1, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 14, 6, 4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 17, 1, 33, 0, 0, 0, 0, 0, 0, 0, 0, 1, 18].to_vec();
        assert_eq!(ast_converter.program, true_val);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        println!("{:?}", toast_vm);
        let curMemoryBlock = toast_vm.memoryList.first();
        assert_eq!(curMemoryBlock.is_some(), true);
        let listLookup = curMemoryBlock.unwrap().listLookup.first();
        assert_eq!(listLookup.unwrap().0, VarTypes::FloatType);
        assert_eq!(f64::from_bits(toast_vm.registers[8 as usize]), 1 as f64 ); 
    }

    #[test]
    fn compileElementAccessMultiDimensionalArray(){
        let source = "let arr: number[] = [[1,2,3], [4,5,6]] arr[1][1]";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 4, 20, 13, 33, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 14, 21, 20, 13, 65, 64, 16, 0, 0, 0, 0, 0, 0, 64, 20, 0, 0, 0, 0, 0, 0, 64, 24, 0, 0, 0, 0, 0, 0, 14, 21, 14, 6, 4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 17, 1, 97, 63, 240, 0, 0, 0, 0, 0, 0, 3, 18, 17, 1, 129, 63, 240, 0, 0, 0, 0, 0, 0, 4, 18].to_vec();
        assert_eq!(ast_converter.program, true_val);
    }

    #[test]
    fn compileAndRunElementAccessMultiDimensionalArray(){
        let source = "let arr: number[] = [[1,2,3], [4,5,6]] arr[1][1]";
        let mut parser = Parser::new(source);
        let ast_nodes = parser.parse();
        let mut ast_converter = ASTConverter::new();
        for ast in &ast_nodes.unwrap() {
            ast_converter.ConvertExprToByteCode(ast.to_owned());
        }
        let true_val: Vec<u8> = [13, 4, 20, 13, 33, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 14, 21, 20, 13, 65, 64, 16, 0, 0, 0, 0, 0, 0, 64, 20, 0, 0, 0, 0, 0, 0, 64, 24, 0, 0, 0, 0, 0, 0, 14, 21, 14, 6, 4, 16, 0, 0, 0, 0, 0, 0, 0, 2, 17, 1, 97, 63, 240, 0, 0, 0, 0, 0, 0, 3, 18, 17, 1, 129, 63, 240, 0, 0, 0, 0, 0, 0, 4, 18].to_vec();
        assert_eq!(ast_converter.program, true_val);
        let mut toast_vm = VMCore::new();
        toast_vm.processProgram(&ast_converter.program);
        let curMemoryBlock = toast_vm.memoryList.first();
        assert_eq!(curMemoryBlock.is_some(), true);
        let listLookup = curMemoryBlock.unwrap().listLookup.first();
        assert_eq!(listLookup.unwrap().0, VarTypes::FloatType);
        assert_eq!(f64::from_bits(toast_vm.registers[8 as usize]), 5 as f64 ); 
    }
}
