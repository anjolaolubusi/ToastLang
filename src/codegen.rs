use crate::parser::*;
use crate::lexer::Token;
pub struct ToastVM{
    /// General Purpose Registers (8 16-bit registers)
    pub gp_reg: [u16; 8],
    /// Program Counter (Check where in the program we are)
    pub pc: usize,
    //pub mem: [u16; (1 << 16)],
    pub cond: u8,
    pub program : Vec<u8>
}

pub enum OpCodes{
    OP_RETURN = 0,
    /// First 4 bits - OpCod
    /// Next 3 bits - Register
    /// Next bit - 1 if Immediate mode else Multiple byte mode
    /// Last 8 bit - Number value or full of ones for the next bytes
    OP_LOAD,
    OP_ADD
}

impl ToastVM{
    pub fn new() -> Self{
        ToastVM { gp_reg: [0; 8], pc: 0, cond: 0, program: Vec::<u8>::new() }
    }

    pub fn ConvertNodeToByteCode(&mut self, node: ASTNode){
        match node {
            ASTNode::ExpressionNode(x) => {
                self.ConvertExprToByteCode(x);
            },
            _ => println!("Could not convert node to bytecode")
        };
    }

    pub fn ConvertExprToByteCode(&mut self, expr: ExprAST){
        match expr {
            ExprAST::NumberExpr(num) => {
                let mut byteCode: u8 = 0;
                byteCode = byteCode | ((OpCodes::OP_LOAD as u8) << 4);
                let register : u8  = 4;
                byteCode = byteCode | (register << 1);
                let immediateMode: u8 = (num < 256.0) as u8;
                byteCode = byteCode | (immediateMode);
                self.program.push(byteCode);
                byteCode = 0;
                if(immediateMode == 1 as u8){
                    byteCode = (num as u8);
                    self.program.push(byteCode);
                    return;
                }
                byteCode = ((num as u16) & (0x00FF)) as u8;
                self.program.push(byteCode);
                byteCode = ((num as u16) >> 8 & (0x00FF)) as u8;
                self.program.push(byteCode);
            },
            _ => println!("Could not convert expression to bytecode")
        }
    }

    pub fn ConsumeByteCode(&mut self){
        let mut byteCode: u8 = self.program[self.pc];
        while self.pc < self.program.len(){
        let opCode = byteCode >> 4;
        match opCode {
            OP_LOAD  => {
                let temp = (byteCode >> 1);
                let reg = (byteCode >> 1) & 7;
                let immediateMode = (byteCode) & (0x0001);
                if(immediateMode == 1){
                    self.pc = self.pc + 1;
                    byteCode = self.program[self.pc];
                    self.gp_reg[reg as usize] = ((byteCode) & (0x00FF)) as u16;
                }
                self.pc = self.pc + 1;
                byteCode = self.program[self.pc];
                let mut bigNum: u16 = 0;
                bigNum = bigNum | (byteCode as u16);
                self.pc = self.pc + 1;
                byteCode = self.program[self.pc];
                bigNum = bigNum | ((byteCode as u16) << 8);
                self.gp_reg[reg as usize] = bigNum;

            },
            _ => println!("No implementation for opcode: {}", opCode)
        }
        self.pc = self.pc + 1;
    }
    }
}