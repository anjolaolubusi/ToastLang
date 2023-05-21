#![allow(non_snake_case)]
#![allow(unused_parens)]
use crate::parser::*;
use crate::lexer::Token;
pub struct ToastVM{
    /// General Purpose Registers (8 16-bit registers)
    pub gp_reg: [u16; 8],
    /// Program Counter (Check where in the program we are)
    pub pc: usize,
    //pub mem: [u16; (1 << 16)],
    pub cond: u8,
    pub program : Vec<u16>,
    pub free_reg: u8
}

pub enum OpCodes{
    OpReturn = 0,
    /// First 4 bits - OpCode
    /// Next 3 bits - Register
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Last 8 bit - Number value or full of ones for the next bytes
    OpLoad,
    /// First 4 bits - OpCode
    /// Next 3 bits - First Reg
    /// Mext 3 bits - Second Reg
    OpAdd
}

impl ToastVM{
    pub fn new() -> Self{
        ToastVM { gp_reg: [0; 8], pc: 0, cond: 0, program: Vec::<u16>::new(), free_reg: 0 }
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
                byteCode = byteCode | ((OpCodes::OpLoad as u16) << 12);
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
                let reg1 = self.ConvertExprToByteCode(*lhs);
                let reg2 = self.ConvertExprToByteCode(*rhs);

                return None;
            },
            _ => {println!("Could not convert expression to bytecode"); return None;}
        }
    }

    pub fn ConsumeByteCode(&mut self){
        let mut byteCode: u16 = 0;
        while self.pc < self.program.len(){
        byteCode = self.program[self.pc];
        let opCode = byteCode >> 12;
        match opCode {
            OpLoad  => {
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
            _ => println!("No implementation for opcode: {}", opCode)
        }
        self.pc = self.pc + 1;
    }
    }
}