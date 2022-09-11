#![allow(non_snake_case)]
use logos::{Lexer, Logos};
use crate::lexer::Token;
use std::collections::HashMap;



#[derive(PartialEq, Clone, Debug)]
pub enum ExprAST {
    VariableExpr(String),
    NumberExpr(f64),
    BinaryExpr {op: Token, lhs: Box<ExprAST>, rhs: Box<ExprAST>},
    CallExpr {func_name: String, parameters: Vec<ExprAST>},
}

#[derive(PartialEq, Clone, Debug)]
pub struct ProtoAST {
    pub Name: String,
    pub Args: Vec<String>
}

#[derive(PartialEq, Clone, Debug)]
pub struct FuncAST {
    pub Proto: ProtoAST,
    pub Body: ExprAST
}

#[derive(PartialEq, Clone, Debug)]
pub enum ASTNode {
    ExternNode(ProtoAST),
    FunctionNode(FuncAST),
    ExpressionNode(ExprAST)
}

#[derive(Clone, Debug)]
pub struct Parser<'a>{
    pub tokens: Vec<Token>,
    pub current_token: Option<Token>,
    pub lexer: Lexer<'a, Token>,
    pub BinOpPrecedence: HashMap<String, i64>
    // Add Operation Precedence
}

impl<'a> Parser <'a>{
    pub fn new(input: &'a str) -> Self{
        let mut BinOp = HashMap::new();
        BinOp.insert("<".to_string(), 10);
        BinOp.insert("+".to_string(), 20);
        BinOp.insert("-".to_string(), 20);
        BinOp.insert("*".to_string(), 40);
        BinOp.insert("/".to_string(), 30);

        Parser {
            tokens: Vec::<Token>::new()
            ,current_token: Some(Token::WhiteSpace)
            ,lexer: Token::lexer(input)
            ,BinOpPrecedence: BinOp.clone()
        }
    }

    pub fn LogErrorASTNode(&self, error: &str) -> Option<ASTNode>{
        println!("Error: {}", error);
        return None;
    }

    pub fn LogErrorExprAST(&self, error: &str) -> Option<ExprAST>{
        println!("Error: {}", error);
        return None;
    }

    pub fn LogErrorProtoAST(&self, error: &str) -> Option<ProtoAST>{
        println!("Error: {}", error);
        return None;
    }

    pub fn getNewToken(&mut self){
        loop{
        self.current_token = self.lexer.next();
        //println!("Current Token is: {:?} Current slice is: {}", self.current_token.clone().unwrap_or(Token::WhiteSpace), self.lexer.slice());
        if self.current_token.is_none() || self.current_token.unwrap() != Token::WhiteSpace {
            break;
        }
        }
    }

    pub fn parse(&mut self) -> Option<Vec<ASTNode>> {
        let mut program: Vec<ASTNode> = Vec::new();
        loop {
            self.getNewToken();
            //println!("{:?}", program);
            if self.current_token.is_none() {
                break;
            }
            let result = match self.current_token.unwrap() {
                Token::Extern => self.ParseExtern(),
                Token::Def => self.ParseDef(),
                _ => self.ParseTopLevel()
            };
            if result.is_none() {
                return  None;
            }
            program.push(result.clone().unwrap());
        }
        return Some(program);
    }

    pub fn ParseTopLevel(&mut self) -> Option<ASTNode> {
        let E = self.ParseExpr();
        if !E.is_none() {
            return Some(ASTNode::ExpressionNode(E.unwrap()))
        }
        self.LogErrorASTNode("Can not parse expression")
    }

    pub fn ParseExtern(&mut self) -> Option<ASTNode>{
        self.getNewToken(); //Consume Extern
        let prototype = self.ParsePrototype();
        //self.getNewToken(); //Consume ')'
        let astNode = ASTNode::ExternNode(prototype.unwrap());
        return Some(astNode);
    }

    pub fn ParseDef(&mut self) -> Option<ASTNode>{
        self.getNewToken(); //Consume Def
        let prototype = self.ParsePrototype();
        self.getNewToken(); //Consume ')'
        if prototype.is_none() {
            return None;
        }
        if self.current_token.unwrap() != Token::FuncBegin {
            return self.LogErrorASTNode("Expected a ':' here");
        }
        self.getNewToken(); //Consume ':'
        let body = self.ParseExpr();
        if body.is_none() {
            return None;
        }
        if self.current_token.is_none() || self.current_token.unwrap() != Token::FuncEnd {
            return self.LogErrorASTNode("Expected a 'end' here");
        }
        //self.getNewToken(); //Consume End
        let funcNode = FuncAST{
            Proto: prototype.unwrap(),
            Body: body.unwrap()
        };
        let astNode = ASTNode::FunctionNode(funcNode);
        return Some(astNode);
    }

    pub fn ParsePrototype(&mut self) -> Option<ProtoAST>{
        if self.current_token.is_none() || self.current_token.unwrap() != Token::Ident {
            return self.LogErrorProtoAST("Expected function name here");
        }
        let prototypeName = self.lexer.slice().to_owned();
        self.getNewToken(); //Consume Identifer
        if self.current_token.unwrap() != Token::OpeningParenthesis {
            return self.LogErrorProtoAST("Expected a '(' here");
        }
        self.getNewToken(); //Consume '('
        let mut newArgs: Vec<String> = Vec::new();
        loop{
            match self.current_token.unwrap() {
                Token::Ident => {
                    newArgs.push(self.lexer.slice().to_owned());
                    self.getNewToken();
                },
                Token::Comma => self.getNewToken(),
                _ => break
            }
        }
        if self.current_token.unwrap() != Token::ClosingParenthesis {
            return self.LogErrorProtoAST("Expected a ')' here");
        }
        let proto: ProtoAST = ProtoAST { Name: prototypeName, Args: newArgs.clone() };
        return Some(proto)
        
    }
    
    pub fn ParsePrimaryExpr(&mut self) -> Option<ExprAST>{
        match self.current_token.unwrap() {
            Token::Ident => {
                return self.ParseIdentExpr();
            },
            Token::Number => {
                let result = ExprAST::NumberExpr(self.lexer.slice().parse::<f64>().unwrap());
                self.getNewToken();
                return Some(result);
            },
            Token::OpeningParenthesis => {
                self.getNewToken(); //Consumes '('
                let expr = self.ParseExpr();
                if self.current_token.unwrap() != Token::ClosingParenthesis {
                    return self.LogErrorExprAST("Expected a ')' here");
                }
                self.getNewToken(); //Consumes ')'
                return Some(expr.unwrap());
            }
            _ => {return self.LogErrorExprAST("Unkown Token");}
        }
    }

    pub fn ParseIdentExpr(&mut self) -> Option<ExprAST>{
        let IdName = self.lexer.slice().to_owned();
        self.getNewToken(); //Consume Ident
        if self.current_token.is_none() || self.current_token.unwrap() != Token::OpeningParenthesis {
            return Some(ExprAST::VariableExpr(IdName));
        }
        self.getNewToken(); //Consume '('
        let mut newArgs: Vec<ExprAST> = Vec::new();
        loop{
            let parameter = self.ParseExpr();
            if parameter.is_none() {
                return None;
            }
            newArgs.push(parameter.unwrap());
            if self.current_token.unwrap() != Token::Comma {
                break;
            }
            self.getNewToken(); //Consume Comma
        }
        if self.current_token.unwrap() != Token::ClosingParenthesis {
            //Error
        }
        self.getNewToken(); //Consume ')'
        return Some(ExprAST::CallExpr { func_name: IdName, parameters: newArgs.clone() })
    }

    pub fn GetTokPrecedence(&mut self)-> i64{
        if !self.lexer.slice().is_ascii() {
            return -1;
        }

        let TokPrec = self.BinOpPrecedence.get(&self.lexer.slice().to_string()).unwrap_or(&-1).to_owned();
        if TokPrec <= 0 {
            return -1;
        }
        return TokPrec;
    }

    pub fn ParseExpr(&mut self) -> Option<ExprAST>{
        let LHS_EXPR = self.ParsePrimaryExpr();
        if LHS_EXPR.is_none() {
            return None;
        }
        //self.getNewToken(); //Eat LHS
        return self.ParseBinOpRHS(0, LHS_EXPR);
    }

    pub fn ParseBinOpRHS(&mut self, ExprPrec: i64, mut LHS: Option<ExprAST>) -> Option<ExprAST>{
        loop{
        let TokPrec = self.GetTokPrecedence();

        if TokPrec < ExprPrec {
            return LHS;
        }

        let BinOp = self.current_token;
        self.getNewToken();
        
        let mut RHS = self.ParsePrimaryExpr();

        if RHS.is_none() {
            return None;
        }

        let NextPrec = self.GetTokPrecedence();
        if TokPrec < NextPrec {
            RHS = self.ParseBinOpRHS(TokPrec + 1, LHS.clone());
            if RHS.is_none() {
                return None;
            }
        }
        //Merge
        //Fix
        let LHS_BOX: Box<ExprAST> = Box::new(LHS.unwrap());
        let RHS_BOX: Box<ExprAST> = Box::new(RHS.unwrap());
        LHS = Some(ExprAST::BinaryExpr { op: BinOp.unwrap(), lhs: LHS_BOX, rhs: RHS_BOX })
        }
    }

    pub fn UpdateSourceString(&mut self, newSource: &'a String){
        self.lexer = Token::lexer(&newSource);
    }
}

mod tests {
    use crate::parser::Parser;

    
    
    #[test]
    fn checkBasicParse(){
        let source = "def foo (a, b): a-b end";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 1)
    }

    #[test]
    fn checkTwoFuncs(){
        let source = "def foo (a, b): \n a-b \n end \n def boo (a, b): a+b end";
        println!("{}", source);
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 2);
    }

    #[test]
    fn checkExprParsing(){
        let source = "4 + 5";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 1);

    }
}