#![allow(non_snake_case)]
#![allow(unused_parens)]
use crate::parser::*;
use crate::lexer::Token;
use num;
use num_derive::{self, FromPrimitive};

pub struct ToastVM{
    /// General Purpose Registers (8 16-bit registers)
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
    /// Represent the current variable type
    pub curType: VarTypes
}

#[derive(FromPrimitive, Debug)]
pub enum OpCodes{
    /// OpLoadReg - Operation Code for copy data from register to another
    /// 
    /// First 4 bits - OpCode
    /// Next 4 bits - Source 
    /// Next 4 bits - Destination
    OpLoadReg = 0,
    /// OpLoadVal - Operation Code for loading data into a specified register
    /// 
    /// First 4 bits - OpCode
    /// Next 3 bits - Register
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Last 8 bit - Number value or full of ones for the next bytes
    OpLoadVal,
    //// OpLoadFloat - Operation Code for loading flaoats into a specified register
    /// 
    /// First 4 bits - OpCode
    /// Next 3 bits - Register
    /// /
    OpLoadFloat,
    /// OpAdd - Operation Code for adding two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpAdd,
    /// OpSub- Operation Code for subtracting two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpSub,
    /// OpMul- Operation Code for multiplying two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpMul,
    /// OpDiv- Operation Code for dividing two numbers that are either in two registers or in the op-code bytecode
    /// 
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpDiv,
    //// OpLoadFloat - Operation Code for printing results
    /// 
    /// First 4 bits - OpCode
    OpResult,
    //// OpLoadFloat - Operation Code for setting the current type
    /// 
    /// First 4 bits - OpCode
    /// Next 12 bits - Denotes type
    OpType,
    //// OpSign - Operation Code for setting the sign register
    /// 
    /// First 4 bits - OpCode
    /// Next 12 bits - Denotes sign
    OpSign
}

#[derive(FromPrimitive, Debug, Clone, Copy)]
pub enum VarTypes{
    IntType=0,
    FloatType
}

impl ToastVM{
    pub fn new() -> Self{
        ToastVM { gp_reg: [0; 9], pc: 0, cond: 0, program: Vec::<u16>::new(), free_reg: 0, sign_reg: 0, curType: VarTypes::IntType}
    }

    /// Converts AST Nodes to bytecode
    /// 
    /// node - AST Node
    pub fn ConvertNodeToByteCode(&mut self, node: ASTNode){
        match node {
            ASTNode::ExpressionNode(x) => {
                let final_reg = self.ConvertExprToByteCode(x);
                // Loads opCode and register into bytecode
                let mut byteCode = 0 | (OpCodes::OpLoadReg as u16) << 12 | (final_reg.unwrap() as u16) << 8 | (8) << 4;
                self.program.push(byteCode);
                self.UpdateCurType();
                byteCode = 0 | (OpCodes::OpResult as u16) << 12;
                self.program.push(byteCode);
            },
            _ => println!("Could not convert node to bytecode")
        };
    }

    /// Converts experission to bytecode
    /// 
    /// expr - expression
    pub fn ConvertExprToByteCode(&mut self, expr: ExprAST) -> Option<u8>{
        match expr {
            ExprAST::IntExpr(num) => {
                let mut byteCode: u16 = 0;
                //Loads Op Code
                byteCode = byteCode | ((OpCodes::OpLoadVal as u16) << 12);

                //Set the register to load into
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;

                //Load register into bytecode
                byteCode = byteCode | ((register as u16) << 9);

                //Determine if we need to be in immediateMode
                let immediateMode: u16 = (num.abs() < 256) as u16;
                //Loads value into bytecode
                byteCode = byteCode | (immediateMode << 8);
                
                if(immediateMode == 1 as u16){
                    //Load number into bytecode
                    byteCode = byteCode | (num as u16);
                    // Adds to the program list
                    self.program.push(byteCode);
                    //Returns register number
                    return Some(register);
                }
                // Adds to the program list
                self.program.push(byteCode);
                //Adds the number to the new bytecode chunk
                byteCode = (num as u16);
                // Adds to the program list
                self.program.push(byteCode);
                self.curType = VarTypes::IntType;
                self.UpdateCurType();
                return Some(register);
            },
            ExprAST::FloatExpr(num) => {
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
            ExprAST::BinaryExpr { op, lhs, rhs, opChar } => {
                // Gets register for the left hand side
                let reg1 = self.ConvertExprToByteCode(*lhs).unwrap();
                let mut byteCode : u16 = 0;
                /// Gets right op code for operation
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
                    ExprAST::IntExpr(trueNum)=> {
                        //Determines if immediate is needed
                        let immediateMode: u16 = (trueNum < 256) as u16;
                        byteCode = byteCode | (immediateMode << 8);
                        if(immediateMode == 1 as u16){
                            // Loads number imto bytecode
                            byteCode = byteCode | (trueNum as u16);
                            // Pushed bytecode to program list
                            self.program.push(byteCode);
                            byteCode = 0;
                            return Some(reg1);
                        }
                        // Gets register for the right hand side
                        let reg2 = self.ConvertExprToByteCode(*rhs).unwrap();
                        // Loads register to bytecode
                        byteCode = byteCode | (reg2 as u16);
                        // Pushed bytecode to program list
                        self.program.push(byteCode);
                        return Some(reg1);
                    },
                    ExprAST::FloatExpr(trueNum) => {
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
            _ => {println!("Could not convert expression to bytecode"); return None;}
        }
    }

    /// Executes Byte code
    pub fn ConsumeByteCode(&mut self){
        let mut byteCode;
        while self.pc < self.program.len(){
        byteCode = self.program[self.pc];
        let temp = byteCode >> 12;
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
                    VarTypes::IntType => {
                        let immediateMode = (byteCode >> 8) & (0x0001);
                        if(immediateMode == 1){
                            match opCode {
                                OpCodes::OpAdd => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] + (byteCode & 0x00FF) as u64;},
                                OpCodes::OpSub => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] - (byteCode & 0x00FF) as u64;},
                                OpCodes::OpMul => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] * (byteCode & 0x00FF) as u64;},
                                OpCodes::OpDiv => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] / (byteCode & 0x00FF) as u64;},
                                _ => {print!("Unkown Operation")}
                            }
                        }else{
                            let reg2 = byteCode & 7;
                            match opCode {
                                OpCodes::OpAdd => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] + self.gp_reg[reg2 as usize];},
                                OpCodes::OpSub => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] - self.gp_reg[reg2 as usize];},
                                OpCodes::OpMul => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] * self.gp_reg[reg2 as usize];},
                                OpCodes::OpDiv => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] / self.gp_reg[reg2 as usize];},
                                _ => {print!("Unkown Operation")}
                            }
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
                    VarTypes::FloatType => {println!("{:?}", f64::from_bits(self.gp_reg[8]))}
                    _ => {println!("{:?}", self.gp_reg[8])}
                }
            },
            OpCodes::OpType => {
                let varType: VarTypes = num::FromPrimitive::from_u16(byteCode & 0x0FFF).unwrap();
                self.curType = varType;
            }
            _ => println!("No implementation for opcode: {:#?}", opCode)
        }
        self.pc = self.pc + 1;
    }
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
    use crate::parser::Parser;

    use super::ToastVM;
    
    #[test]
    fn compileAndRunSimpleAdditionProgram(){
        let code = "10 + 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let mut cpu: ToastVM = ToastVM::new();
        for astNode in astNodes.unwrap(){
            cpu.ConvertNodeToByteCode(astNode);
        }
        cpu.ConsumeByteCode();
        assert_eq!(cpu.gp_reg[8], 15);
    }

    #[test]
    fn compileAndRunSimpleMultiplicationProgram(){
        let code = "10 * 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let mut cpu: ToastVM = ToastVM::new();
        for astNode in astNodes.unwrap(){
            cpu.ConvertNodeToByteCode(astNode);
        }
        cpu.ConsumeByteCode();
        assert_eq!(cpu.gp_reg[8], 50);
    }


    #[test]
    fn compileAndRunSimpleSubtractionProgram(){
        let code = "10 - 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let mut cpu: ToastVM = ToastVM::new();
        for astNode in astNodes.unwrap(){
            cpu.ConvertNodeToByteCode(astNode);
        }
        cpu.ConsumeByteCode();
        assert_eq!(cpu.gp_reg[8], 5);
    }

    #[test]
    fn compileAndRunSimpleDivisionProgram(){
        let code = "10 / 5";
        let mut parser = Parser::new(code);
        let astNodes = parser.parse();
        let mut cpu: ToastVM = ToastVM::new();
        for astNode in astNodes.unwrap(){
            cpu.ConvertNodeToByteCode(astNode);
        }
        cpu.ConsumeByteCode();
        assert_eq!(cpu.gp_reg[8], 2);
    }
}