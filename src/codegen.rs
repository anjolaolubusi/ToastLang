#![allow(non_snake_case)]
#![allow(unused_parens)]
use std::collections::HashMap;

use crate::parser::*;
use crate::lexer::Token;
use num;
use num_derive::{self, FromPrimitive};

pub struct ToastMemoryBlock {
    pub varIdTable : Vec<f64>
}

impl ToastMemoryBlock {
    pub fn new() -> Self{
        ToastMemoryBlock{
            varIdTable: Vec::<f64>::new()
        }
    }
}

pub struct ExprConverter {
    pub funcIdTable : HashMap<String, u16>,
    pub varLookUp: HashMap<String, (u16, u16)>,
    pub curMemoryId: u16,
    pub varCount: u16,
    pub curType: VarTypes,
    pub program : Vec<u16>, 
    pub func_id: u16,
    pub free_reg: u8
}

pub struct ToastVM{
    /// General Purpose Registers (8 64-bit registers)
    pub gp_reg: [u64; 9],
    /// Program Counter (Check where in the program we are)
    pub pc: usize,
    //pub mem: [u16; (1 << 16)],
    /// Condition register. 2 for false, 0 for netural and 1 for true.
    pub cond: u8,
    /// Sign register. 2 for false, 0 for netural and 1 for true.
    pub sign_reg: u8,
    pub program : Vec<u16>, 
    /// Counter to note the next free register    
    pub free_reg: u8,
    pub func_id: u16,
    /// Represent the current variable type
    pub curType: VarTypes,
    /// List of function bytecode
    pub funcByteList : HashMap<u16, usize>,
    pub memoryList: Vec<ToastMemoryBlock>,
    pub curMemoryId: usize
}

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum OpCodes{
    /// OpLoadReg - Operation Code for copy data from register to another
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 4 bits - Source 
    /// 
    /// Next 4 bits - Destination
    OpLoadReg = 0,
    /// OpLoadVal - Operation Code for loading data into a specified register
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - Register
    /// 
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// 
    /// Last 8 bit - Number value or full of ones for the next bytes
    OpLoadVal,
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
    /// Mext 3 bits - Second Reg
    OpAdd,
    /// OpSub- Operation Code for subtracting two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// 
    /// Mext 3 bits - Second Reg
    OpSub,
    /// OpMul- Operation Code for multiplying two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - First Reg
    /// 
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// 
    /// Mext 3 bits - Second Reg
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
    //// OpResult - Operation Code for printing results
    /// 
    /// First 4 bits - OpCode
    OpResult,
    //// OpType - Operation Code for setting the current type
    /// 
    /// First 4 bits - OpCode
    ///
    ///  Next 12 bits - Denotes type
    OpType,
    // //// OpSign - Operation Code for setting the sign register
    // /// 
    // /// First 4 bits - OpCode
    // /// Next 12 bits - Denotes sign
    // OpSign,

    /// OpFunc - Operation Code for the function start 
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 12 bits - Function Id
    OpFuncBegin,
    OpFuncEnd,
    OpFuncCall,
    /// OpNewMemBlock - Operation Code for creating new memory block
    ///
    /// First 4 bits - OpCode
    OpNewMemBlock,
    /// OpNewVar - Operation Code for creating new variable
    ///
    /// First 4 bits - OpCode
    OpNewVar,
    /// OpLoadVar - Operation Code for loading variable value
    ///
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - Register for value
    /// 
    /// Remaining 9 bits - Varible id
    OpLoadVar,
    /// OpLoadVarToReg - Operation Code for loading variable value to register
    /// 
    /// First 4 bits - OpCode
    /// 
    /// Next 3 bits - Register for value
    /// 
    /// Remaining 9 bits - Varible id
    OpLoadVarToReg,
}

#[derive(FromPrimitive, Debug, Clone, Copy)]
pub enum VarTypes{
    FloatType=0
}

impl ExprConverter {
    pub fn new() -> Self{
        ExprConverter{
            funcIdTable: HashMap::new(),
            varLookUp: HashMap::new(),
            curMemoryId: 0,
            varCount: 0,
            curType: VarTypes::FloatType,
            program: Vec::<u16>::new(),
            func_id: 0,
            free_reg: 0
        }
    }

    /// Converts AST Nodes to bytecode
    /// 
    /// node - AST Node
    pub fn ConvertNodeToByteCode(&mut self, node: ASTNode){
        match node {
            ASTNode::ExpressionNode(x) => {
                let final_reg = self.ConvertExprToByteCode(x);
                if final_reg.is_some(){
                    // Loads opCode and register into bytecode
                    let mut byteCode = 0 | (OpCodes::OpLoadReg as u16) << 12 | (final_reg.unwrap() as u16) << 8 | (8) << 4;
                    self.program.push(byteCode);
                    self.UpdateCurType();
                    byteCode = 0 | (OpCodes::OpResult as u16) << 12;
                    self.program.push(byteCode);
                }
            },
            ASTNode::FunctionNode(x) => {
                let funcId = self.func_id;
                self.funcIdTable.insert(x.Proto.Name, self.func_id);
                self.curMemoryId += 1;
                let mut byteCode : u16 = 0 | ((OpCodes::OpFuncBegin as u16) << 12);
                byteCode = byteCode | funcId;
                self.program.push(byteCode);
                let final_reg = self.ConvertExprToByteCode(x.Body);
                self.UpdateCurType();
                byteCode = 0 | (OpCodes::OpLoadReg as u16) << 12 | (final_reg.unwrap() as u16) << 8 | (8) << 4;
                self.program.push(byteCode);
                byteCode = 0 | (OpCodes::OpResult as u16) << 12;
                self.program.push(byteCode);
                byteCode = 0 | (OpCodes::OpFuncEnd as u16) << 12;
                self.program.push(byteCode);
                self.func_id = self.func_id + 1;

            }
            _ => println!("Could not convert node to bytecode")
        };
    }

    /// Converts experission to bytecode
    /// 
    /// expr - expression
    pub fn ConvertExprToByteCode(&mut self, expr: ExprAST) -> Option<u8>{
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
                self.curType = VarTypes::FloatType;
                self.UpdateCurType();
                return Some(register); 
            },
            ExprAST::VariableExpr(val) => {
                let mut bytecode : u16 = 0;
                if !self.varLookUp.contains_key(&val) {
                    self.varLookUp.insert(val, (self.varCount, self.curMemoryId as u16));
                    bytecode = 0 | (OpCodes::OpNewVar as u16) << 12;
                    self.program.push(bytecode);
                    return None;
                }else{
                    let varTuple = self.varLookUp.get(&val).unwrap();
                    let currentReg = self.free_reg;
                    bytecode =  0 | (OpCodes::OpLoadVarToReg as u16) << 12 | (currentReg as u16) << 9 | (varTuple.0 as u16);
                    self.program.push(bytecode);
                    self.free_reg = (self.free_reg + 1) % 8;
                    return Some(currentReg);
                }
            },
            ExprAST::VariableAssignExpr { varName, value } => {
                let mut bytecode = 0;
                self.ConvertExprToByteCode(*varName);
                let varId = self.varCount;
                self.varCount += 1;
                let valueReg = self.ConvertExprToByteCode(*value).unwrap();
                bytecode =  0 | (OpCodes::OpLoadVar as u16) << 12 | (valueReg as u16) << 9 | (varId as u16);
                self.program.push(bytecode);
                return  Some(0);                
            },
            ExprAST::BinaryExpr { op, lhs, rhs, opChar } => {
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
                    ExprAST::NumberExpr(trueNum) => {
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
            ExprAST::CallExpr { func_name, parameters } => {
                let mut varId: usize = 0;
                let funcId  =  *self.funcIdTable.get(&func_name).unwrap();
                let mut bytecode : u16 = 0;
                bytecode = 0 | (OpCodes::OpNewMemBlock as u16) << 12;
                self.program.push(bytecode);
                if !parameters.is_empty() {
                    loop {
                        let reg : u8 = self.ConvertExprToByteCode(parameters.get(varId).unwrap().clone()).unwrap();
                        bytecode = 0 | (OpCodes::OpNewVar as u16) << 12;
                        self.program.push(bytecode);
                        bytecode =  0 | (OpCodes::OpLoadVar as u16) << 12 | (reg as u16) << 9 | (varId as u16);
                        self.program.push(bytecode);
                        varId += 1;
                        if(varId >= parameters.len()){
                            break;
                        }
                    }
                }
                bytecode = 0 | (OpCodes::OpFuncCall as u16) << 12 | funcId;
                self.program.push(bytecode);
                return None;
            }
            _ => {println!("Could not convert expression to bytecode"); return None;}
        }
    }

    pub fn UpdateCurType(&mut self){
        let byteCode = 0 | (OpCodes::OpType as u16) << 12 | (self.curType as u16);
        self.program.push(byteCode);
    }

}

impl ToastVM{
    pub fn new() -> Self{
        let mut vm = ToastVM { gp_reg: [0; 9], pc: 0, cond: 0, program: Vec::<u16>::new(), free_reg: 0, sign_reg: 0, curType: VarTypes::FloatType, func_id: 0, funcByteList: HashMap::new(), memoryList: Vec::<ToastMemoryBlock>::new(), curMemoryId: 0};
        vm.memoryList.push(ToastMemoryBlock::new());
        return vm;
    }

    pub fn processProgram(&mut self){
        let mut byteCode;
        while self.pc < self.program.len(){
            byteCode = self.program[self.pc];
            self.ConsumeByteCode(byteCode);
            self.pc = self.pc + 1;
        }
    }

    /// Executes Byte code
    pub fn ConsumeByteCode(&mut self, mut byteCode: u16){
        // let mut byteCode;
        // while self.pc < self.program.len(){
        // byteCode = self.program[self.pc];
        // let temp = byteCode >> 12;
        let opCode : OpCodes = num::FromPrimitive::from_u16(byteCode >> 12).unwrap();
        match opCode {
            OpCodes::OpLoadVal  => {
                let reg = (byteCode >> 9) & 7;
                let immediateMode = (byteCode >> 8) & (0x0001);
                if(immediateMode == 1){
                    self.gp_reg[reg as usize] = ((byteCode) & (0x00FF)) as u64;
                }else{
                self.pc = self.pc + 1;
                byteCode = self.program[self.pc];
                self.gp_reg[reg as usize] = byteCode as u64;
                }

            },
            OpCodes::OpLoadFloat => {
                let reg = (byteCode >> 9) & 7;
                self.pc = self.pc + 1;
                let mut num: u64 = 0;
                num = num | self.program[self.pc] as u64 | (self.program[self.pc+1] as u64) << 16 | (self.program[self.pc+2] as u64) << 32 | (self.program[self.pc+3] as u64) << 48;
                self.pc = self.pc+3;
                self.gp_reg[reg as usize] = num;
            },
            OpCodes::OpAdd | OpCodes::OpSub | OpCodes::OpDiv | OpCodes::OpMul => {
                let reg1 = (byteCode >> 9) & 7;
                match self.curType {
                    VarTypes::FloatType => {
                        let reg2 = byteCode & 7;
                        match opCode {
                            OpCodes::OpAdd => {self.gp_reg[reg1 as usize] = f64::to_bits(f64::from_bits(self.gp_reg[reg1 as usize]) + f64::from_bits(self.gp_reg[reg2 as usize]));},
                            OpCodes::OpSub => {self.gp_reg[reg1 as usize] = f64::to_bits(f64::from_bits(self.gp_reg[reg1 as usize]) - f64::from_bits(self.gp_reg[reg2 as usize]));},
                            OpCodes::OpMul => {self.gp_reg[reg1 as usize] = f64::to_bits(f64::from_bits(self.gp_reg[reg1 as usize]) * f64::from_bits(self.gp_reg[reg2 as usize]));},
                            OpCodes::OpDiv => {self.gp_reg[reg1 as usize] = f64::to_bits(f64::from_bits(self.gp_reg[reg1 as usize]) / f64::from_bits(self.gp_reg[reg2 as usize]));},
                            _ => {print!("Unkown Operation")}
                        }
                    },
                    _ => {println!("Unkown number type")}
                }
            },
            OpCodes::OpLoadReg => {
                let sourceRegNum = (byteCode >> 8) & 15;
                let destRegNum = (byteCode >> 4) & 15;
                self.gp_reg[destRegNum as usize] = self.gp_reg[sourceRegNum as usize];
            },
            OpCodes::OpResult => {
                match self.curType {
                    VarTypes::FloatType => {println!("{:?}", f64::from_bits(self.gp_reg[8]))},
                    _ => {println!("{:?}", self.gp_reg[8])}
                }
            },
            OpCodes::OpType => {
                let varType: VarTypes = num::FromPrimitive::from_u16(byteCode & 0x0FFF).unwrap();
                self.curType = varType;
            },
            OpCodes::OpFuncBegin => {
                let funcId = byteCode & 4095;
                let funcStart = self.pc;
                self.pc += 1;
                let mut opCode : OpCodes = num::FromPrimitive::from_u16(self.program[self.pc] >> 12).unwrap();
                while (opCode != OpCodes::OpFuncEnd) {
                    self.pc += 1;
                    opCode = num::FromPrimitive::from_u16(self.program[self.pc] >> 12).unwrap();
                }
                self.funcByteList.insert(funcId, funcStart);
            },
            OpCodes::OpFuncCall => {
                let funcId = byteCode & 4095;
                let funcStart = self.funcByteList.get(&funcId).unwrap().to_owned();
                let returnPC = self.pc + 1;
                self.pc = funcStart;
                self.pc += 1;
                let mut opCode : OpCodes = num::FromPrimitive::from_u16(self.program[self.pc] >> 12).unwrap();
                while(opCode != OpCodes::OpFuncEnd){
                    self.ConsumeByteCode(self.program[self.pc]);
                    self.pc += 1;
                    opCode  = num::FromPrimitive::from_u16(self.program[self.pc] >> 12).unwrap();
                }
                self.memoryList.pop();
                self.curMemoryId -= 1;
                self.pc = returnPC;
            },
            OpCodes::OpNewMemBlock => {
                self.memoryList.push(ToastMemoryBlock::new());
                self.curMemoryId += 1;
            },
            OpCodes::OpNewVar => {
                let curMem = self.memoryList.get_mut(self.curMemoryId).unwrap();
                curMem.varIdTable.push(0 as f64);
            },
            OpCodes::OpLoadVar => {
                let varReg = (byteCode & 0x0700) >> 9;
                let varId = (byteCode & 0x00FF);
                let curMem = self.memoryList.get_mut(self.curMemoryId).unwrap();
                curMem.varIdTable[varId as usize] = f64::from_bits(self.gp_reg[varReg as usize]);
            },
            OpCodes::OpLoadVarToReg => {
                let varReg = (byteCode & 0x0700) >> 9;
                let varId = (byteCode & 0x00FF);
                let curMem = self.memoryList.get_mut(self.curMemoryId).unwrap();
                let varVal = *curMem.varIdTable.get(varId as usize).unwrap();
                let num: u64 = f64::to_bits(varVal);
                self.gp_reg[varReg as usize] = num;
            }
            _ => println!("No implementation for opcode: {:#?}", opCode)
        }
//        self.pc = self.pc + 1;
    
    }

    pub fn UpdateCurType(&mut self){
        let byteCode = 0 | (OpCodes::OpType as u16) << 12 | (self.curType as u16);
        self.program.push(byteCode);
    }

    /// Goes through program and logs each bytecode
    pub fn LogByteCodeProgram(&mut self){
        let mut byteCode;
        let mut i = 0;
        println!("");
        println!(
            "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
            "ID", "OpCode", "Data", "HexCode"
        );
        while i < self.program.len(){
            byteCode = self.program[i];
            let opCode: OpCodes = num::FromPrimitive::from_u16(byteCode >> 12).unwrap();
            match opCode {
                OpCodes::OpLoadVal => {
                    let indexNum = i;
                    let originalByteCode = byteCode;
                    let regNum = (byteCode >> 9) & 7;
                    let immediateMode = (byteCode >> 8) & (0x0001);
                    let trueNum;
                    if(immediateMode == 1){
                        trueNum = ((byteCode) & (0x00FF));
                    }else{
                        i = i + 1;
                        byteCode = self.program[i];
                        trueNum = byteCode;
                    }
                    println!(
                        "{0: <10} | {opCode:?} | Registor: {reg:?} Immediate Mode: {iMode} Number: {trueNum} | {hexCode:X}",
                        indexNum, opCode=opCode ,reg=regNum, iMode=immediateMode,trueNum=trueNum, hexCode=originalByteCode)                    
                },
                OpCodes::OpAdd | OpCodes::OpSub | OpCodes::OpMul | OpCodes::OpDiv => {
                    let indexNum = i;
                    let originalByteCode = byteCode;
                    let reg1 = (byteCode >> 9) & 7;
                    let immediateMode = (byteCode >> 8) & (0x0001);
                    let operandNum;
                    if(immediateMode == 1){
                        operandNum = ((byteCode) & (0x00FF));
                        println!(
                            "{0: <10} | {opCode:?} | Registor: {reg:?} Immediate Mode: {iMode} Operand Number: {trueNum} | {hexCode:X}",
                            indexNum, opCode=opCode ,reg=reg1, iMode=immediateMode,trueNum=operandNum, hexCode=originalByteCode);
                    }else{
                        operandNum = byteCode & 7;
                        println!(
                            "{0: <10} | {opCode:?} | Registor 1: {reg:?} Immediate Mode: {iMode} Registor 2: {reg2} | {hexCode:X}",
                            indexNum, opCode=opCode ,reg=reg1, iMode=immediateMode,reg2=operandNum, hexCode=originalByteCode)
                    }
                },
                OpCodes::OpLoadReg => {
                    let indexNum = i;
                    let originalByteCode = byteCode;
                    let sourceRegNum = (byteCode >> 8) & 15;
                    let destRegNum = (byteCode >> 4) & 15;
                    println!(
                        "{0: <10} | {opCode:?} | Source Registor: {reg:?} Destination Registor: {destReg} | {hexCode:X}",
                        indexNum, opCode=opCode ,reg=sourceRegNum, destReg=destRegNum, hexCode=originalByteCode)                    
                },
                _ => {
                    println!(
                        "{0: <10} | {1: <10} | {2: <10} | {hexCode:X}",
                       i,"Unknown","", hexCode=byteCode)
                }
            };
            i = i + 1;
        }
    }
}

mod tests {
    use crate::{codegen::ExprConverter, parser::Parser};

    use super::ToastVM;
    
    #[test]
    fn compileAndRunSimpleAdditionProgram(){
        let code = "10 + 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let mut cpu: ToastVM = ToastVM::new();
        let mut converter: ExprConverter = ExprConverter::new();
        for astNode in astNodes.unwrap(){
            converter.ConvertNodeToByteCode(astNode);
            cpu.program = converter.program.clone();
        }
        cpu.processProgram();
        assert_eq!(cpu.gp_reg[8], 15);
    }

    #[test]
    fn compileAndRunSimpleMultiplicationProgram(){
        let code = "10 * 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let mut cpu: ToastVM = ToastVM::new();
        let mut converter: ExprConverter = ExprConverter::new();
        for astNode in astNodes.unwrap(){
            converter.ConvertNodeToByteCode(astNode);
            cpu.program = converter.program.clone();
        }
        cpu.processProgram();
        assert_eq!(cpu.gp_reg[8], 50);
    }


    #[test]
    fn compileAndRunSimpleSubtractionProgram(){
        let code = "10 - 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let mut cpu: ToastVM = ToastVM::new();
        let mut converter: ExprConverter = ExprConverter::new();
        for astNode in astNodes.unwrap(){
            converter.ConvertNodeToByteCode(astNode);
            cpu.program = converter.program.clone();
        }
        cpu.processProgram();
        assert_eq!(cpu.gp_reg[8], 5);
    }

    #[test]
    fn compileAndRunSimpleDivisionProgram(){
        let code = "10 / 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let converter: ExprConverter = ExprConverter::new();
        let mut cpu: ToastVM = ToastVM::new();
        let mut converter: ExprConverter = ExprConverter::new();
        for astNode in astNodes.unwrap(){
            converter.ConvertNodeToByteCode(astNode);
            cpu.program = converter.program.clone();
        }
        cpu.processProgram();
        assert_eq!(cpu.gp_reg[8], 2);
    }
}