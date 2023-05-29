#![allow(non_snake_case)]
#![allow(unused_parens)]
use crate::parser::*;
use crate::lexer::Token;
use num;
use num_derive::{self, FromPrimitive};

pub struct ToastVM{
    /// General Purpose Registers (8 16-bit registers)
    pub gp_reg: [u16; 9],
    /// Program Counter (Check where in the program we are)
    pub pc: usize,
    //pub mem: [u16; (1 << 16)],
    /// Condition register. -1 for false, 0 for netural and 1 for true.
    pub cond: u8,
    /// Sign register. -1 for false, 0 for netural and 1 for true.
    pub sign_reg: u8,
    pub program : Vec<u16>,
    /// Counter to note the next free register    
    pub free_reg: u8,
}

#[derive(FromPrimitive, Debug)]
pub enum OpCodes{
    /// First 4 bits - OpCode
    /// Next 4 bits - Source 
    /// Next 4 bits - Destination
    OpLoadReg = 0,
    /// First 4 bits - OpCode
    /// Next 3 bits - Register
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Last 8 bit - Number value or full of ones for the next bytes
    OpLoadVal,
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpAdd,
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpSub,
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpMul,
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Mext 3 bits - Second Reg
    OpDiv,
}

impl ToastVM{
    pub fn new() -> Self{
        ToastVM { gp_reg: [0; 9], pc: 0, cond: 0, program: Vec::<u16>::new(), free_reg: 0, sign_reg: 0}
    }

    pub fn ConvertNodeToByteCode(&mut self, node: ASTNode){
        match node {
            ASTNode::ExpressionNode(x) => {
                self.ConvertExprToByteCode(x);
            },
            _ => println!("Could not convert node to bytecode")
        };
    }

    pub fn ConvertExprToByteCode(&mut self, expr: ExprAST) -> Option<u8>{
        match expr {
            ExprAST::NumberExpr(num) => {
                let mut byteCode: u16 = 0;
                byteCode = byteCode | ((OpCodes::OpLoadVal as u16) << 12);
                let register : u8  = self.free_reg;
                self.free_reg = (self.free_reg + 1) % 8;
                byteCode = byteCode | ((register as u16) << 9);
                let immediateMode: u16 = (num < 256.0) as u16;
                byteCode = byteCode | (immediateMode << 8);
                if(immediateMode == 1 as u16){
                    byteCode = byteCode | (num as u16);
                    self.program.push(byteCode);
                    return Some(register);
                }
                self.program.push(byteCode);
                byteCode = (num as u16);
                self.program.push(byteCode);
                return Some(register);
            },
            ExprAST::BinaryExpr { op, lhs, rhs, opChar } => {
                let reg1 = self.ConvertExprToByteCode(*lhs).unwrap();
                let mut byteCode : u16 = 0;
                let opCode : u8 = match op {
                    Token::Plus => OpCodes::OpAdd as u8,
                    Token::Minus => OpCodes::OpSub as u8,
                    Token::Multiply => OpCodes::OpMul as u8,
                    Token::Divide => OpCodes::OpDiv as u8,
                    _ => 0 as u8
                };
                byteCode = byteCode | (opCode as u16) << 12 | ( (reg1 as u16) << 9);
                if let ExprAST::NumberExpr(trueNum) = *rhs {
                    let immediateMode: u16 = (trueNum < 256.0) as u16;
                    byteCode = byteCode | (immediateMode << 8);
                    if(immediateMode == 1 as u16){
                        byteCode = byteCode | (trueNum as u16);
                        self.program.push(byteCode);
                        byteCode = 0;
                        byteCode = byteCode | (OpCodes::OpLoadReg as u16) << 12 | (reg1 as u16) << 8 | (8) << 4;
                        self.program.push(byteCode);
                        return None;
                    }
                    let reg2 = self.ConvertExprToByteCode(*rhs).unwrap();
                    byteCode = byteCode | (reg2 as u16);
                    self.program.push(byteCode);
                    byteCode = 0;
                    byteCode = byteCode | (OpCodes::OpLoadReg as u16) << 12 | (reg1 as u16) << 8 | (8) << 4;
                    self.program.push(byteCode);
                    return None;
                }


                return None;
            },
            _ => {println!("Could not convert expression to bytecode"); return None;}
        }
    }

    pub fn ConsumeByteCode(&mut self){
        let mut byteCode;
        while self.pc < self.program.len(){
        byteCode = self.program[self.pc];
        let opCode = num::FromPrimitive::from_u16(byteCode >> 12).unwrap();
        match opCode {
            OpCodes::OpLoadVal  => {
                let reg = (byteCode >> 9) & 7;
                let immediateMode = (byteCode >> 8) & (0x0001);
                if(immediateMode == 1){
                    self.gp_reg[reg as usize] = ((byteCode) & (0x00FF)) as u16;
                }else{
                self.pc = self.pc + 1;
                byteCode = self.program[self.pc];
                self.gp_reg[reg as usize] = byteCode;
                }

            },
            OpCodes::OpAdd | OpCodes::OpSub | OpCodes::OpDiv | OpCodes::OpMul => {
                let reg1 = (byteCode >> 9) & 7;
                let immediateMode = (byteCode >> 8) & (0x0001);
                if(immediateMode == 1){
                    match opCode {
                        OpCodes::OpAdd => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] + (byteCode & 0x00FF);},
                        OpCodes::OpSub => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] - (byteCode & 0x00FF);},
                        OpCodes::OpMul => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] * (byteCode & 0x00FF);},
                        OpCodes::OpDiv => {self.gp_reg[reg1 as usize] = self.gp_reg[reg1 as usize] / (byteCode & 0x00FF);},
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
            OpCodes::OpLoadReg => {
                let sourceRegNum = (byteCode >> 8) & 15;
                let destRegNum = (byteCode >> 4) & 15;
                self.gp_reg[destRegNum as usize] = self.gp_reg[sourceRegNum as usize];
            },
            _ => println!("No implementation for opcode: {:#?}", opCode)
        }
        self.pc = self.pc + 1;
    }
    }


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