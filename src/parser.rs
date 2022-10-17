#![allow(non_snake_case)]
#![allow(unused_parens)]
use logos::{Lexer, Logos};
use crate::lexer::Token;
use std::collections::HashMap;



#[derive(PartialEq, Clone, Debug)]
pub enum ExprAST {
    VariableExpr(String),
    NumberExpr(f64),
    BinaryExpr {op: Token, lhs: Box<ExprAST>, rhs: Box<ExprAST>, opChar: String},
    CallExpr {func_name: String, parameters: Vec<ExprAST>},
    IfExpr{ cond: Box<ExprAST>, Then: Box<ExprAST>, Else: Box<ExprAST>},
    ForExpr{ var: String, start: Box<ExprAST>, end: Box<ExprAST>, stepFunc: Box<ExprAST>, body: Box<ExprAST>},
    InclusiveForExpr{ var: String, start: Box<ExprAST>, end: Box<ExprAST>, stepFunc: Box<ExprAST>, body: Box<ExprAST>},
    UnaryExpr {Opcode: String, Operand: Box<ExprAST>},
    CommentExpr(String)
}

#[derive(PartialEq, Clone, Debug)]
pub struct ProtoAST {
    pub Name: String,
    pub Args: Vec<String>,
    pub IsOperator: bool,
    pub Precedence: i64
}

impl ProtoAST{
    pub fn isUnaryOp(&self) -> bool{
        return self.IsOperator && self.Args.len() == 1;
    }

    pub fn isBinaryOp(&self) -> bool{
        return self.IsOperator && self.Args.len() == 2;
    }

    pub fn getOperatorName(&self) -> String {
        let mut operator = self.Name.clone();
        if(self.isBinaryOp()){
            operator =  operator.replace("binary", "");
        }
        if(self.isUnaryOp()){
            operator =  operator.replace("unary", "");
        }
        return operator;
    }
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
    pub BinOpPrecedence: HashMap<String, i64>,
    pub TokensToSkip: Vec<Token>
    // Add Operation Precedence
}

impl<'a> Parser <'a>{
    pub fn new(input: &'a str) -> Self{
        let mut BinOp = HashMap::new();
        BinOp.insert("<".to_string(), 10);
        BinOp.insert(">".to_string(), 10);
        BinOp.insert("+".to_string(), 20);
        BinOp.insert("-".to_string(), 20);
        BinOp.insert("*".to_string(), 40);
        BinOp.insert("/".to_string(), 30);

        let skipToken = [Token::WhiteSpace].to_vec();
        Parser {
            tokens: Vec::<Token>::new()
            ,current_token: Some(Token::WhiteSpace)
            ,lexer: Token::lexer(input)
            ,BinOpPrecedence: BinOp.clone()
            ,TokensToSkip: skipToken.clone()
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
        let mut currStr = self.lexer.slice();

        // if self.current_token.unwrap_or(Token::Error) == Token::Comment {
        //     self.ParseSingleLineComment();
        //     break;
        // }


        //println!("Current Token is: {:?} Current slice is: {}", self.current_token.clone().unwrap_or(Token::WhiteSpace), self.lexer.slice());
        if self.current_token.is_none() || !self.TokensToSkip.contains(&self.current_token.unwrap()) {
            break;
        }

        }
    }

    pub fn parse(&mut self) -> Option<Vec<ASTNode>> {
        let mut program: Vec<ASTNode> = Vec::new();
        loop {
            //println!("{:?}", program);
            if self.current_token.is_none() {
                break;
            }

            if(self.current_token.unwrap() == Token::WhiteSpace || self.current_token.unwrap() == Token::MultilineCommentEnd){
                self.getNewToken();
            }

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

    pub fn ParseSingleLineComment(&mut self) -> Option<ExprAST> {
        let mut comment = "".to_string();
        self.getNewToken(); //Eat '//'
        loop{
            if self.current_token.is_none() || (self.current_token.unwrap() == Token::WhiteSpace && self.lexer.slice() == "\n"){
                break;
            }
            comment += self.lexer.slice();
            self.current_token = self.lexer.next();
        }
        let commentExpr = ExprAST::CommentExpr(comment);
        return Some(commentExpr);
    }

    pub fn ParseMultiLineComment(&mut self) -> Option<ExprAST>{
        let mut comment = "".to_string();
        self.getNewToken(); //Eat '/*'
        loop{
            if self.current_token.is_none() || (self.current_token.unwrap() == Token::MultilineCommentEnd){
                break;
            }
            comment += self.lexer.slice();
            self.current_token = self.lexer.next();
        }
        let commentExpr = ExprAST::CommentExpr(comment);
        return Some(commentExpr);
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
        //self.getNewToken(); //Consume ')'
        if prototype.is_none() {
            return None;
        }
        if self.current_token.unwrap() != Token::FuncBegin {
            return self.LogErrorASTNode("Expected a ':' here");
        }
        self.getNewToken(); //Consume ':'
        //TODO: Change body to allow multiple statments
        let body = self.ParseExpr();
        if body.is_none() {
            return None;
        }
        if self.current_token.is_none() || self.current_token.unwrap() != Token::FuncEnd {
            return self.LogErrorASTNode("Expected a 'end' here");
        }
        self.getNewToken(); //Consume End
        let funcNode = FuncAST{
            Proto: prototype.unwrap(),
            Body: body.unwrap()
        };
        let astNode = ASTNode::FunctionNode(funcNode);
        return Some(astNode);
    }

    pub fn ParsePrototype(&mut self) -> Option<ProtoAST>{
        let mut Kind: usize = 0;
        let mut BinaryPrecedence: i64 = 0;
        let mut prototypeName: String = "".to_string();
        // if self.current_token.is_none() || self.current_token.unwrap() != Token::Ident  {
        //     return self.LogErrorProtoAST("Expected function name here");
        // }

        match self.current_token.unwrap() {
            Token::Ident => {
                prototypeName = self.lexer.slice().to_owned();
                Kind = 0;        
                self.getNewToken(); //Consume Identifer
            },
            Token::Binary => {
                self.getNewToken(); //Consume Binary
                prototypeName = "binary".to_string() + self.lexer.slice();
                Kind = 2;
                self.getNewToken(); //Consume Operator
                BinaryPrecedence =  self.lexer.slice().parse::<i64>().unwrap();
                if(BinaryPrecedence  < 1 || BinaryPrecedence > 100){
                    return self.LogErrorProtoAST("Invalid precedence must be between 1 and 100 inclusive");
                }
                self.getNewToken(); //Consume Binary precedence
            },
            Token::Unary => {
                self.getNewToken(); //Consume Binary
                let unaryName = self.lexer.slice();
                if(!unaryName.is_ascii()){
                    return self.LogErrorProtoAST("Expected unary operator");
                }
                prototypeName = "unary".to_string() + unaryName;
                Kind = 1;
                self.getNewToken(); //Consume Operator

            }
            _ => {return self.LogErrorProtoAST("Expected function name here")}
        }

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
        self.getNewToken(); //Consume ')'
        if(newArgs.len() != Kind && Kind != 0){
            return self.LogErrorProtoAST("Invalid number of operands for operator");
        }
        let proto: ProtoAST = ProtoAST { Name: prototypeName, Args: newArgs.clone(), IsOperator: Kind != 0, Precedence: BinaryPrecedence };
        if proto.isBinaryOp() {
        self.BinOpPrecedence.insert(proto.getOperatorName(), proto.Precedence);
        }
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
            },
            Token::If => self.ParseIfElseExpr(),
            Token::For => self.ParseForExpr(),
            Token::Comment => self.ParseSingleLineComment(),
            Token::MultilineCommentBegin => self.ParseMultiLineComment(),
            _ => {return self.LogErrorExprAST("Unkown Token");}
        }
    }

    pub fn ParseUnaryExpr(&mut self) -> Option<ExprAST>{
        if(!self.lexer.slice().is_ascii() || self.lexer.slice().chars().all(char::is_alphanumeric) || self.current_token.unwrap() == Token::OpeningParenthesis || self.current_token.unwrap() == Token::Comma || self.current_token.unwrap() == Token::Comment || self.current_token.unwrap() == Token::MultilineCommentBegin){
            return self.ParsePrimaryExpr();
        }

        let Opc = self.lexer.slice();
        self.getNewToken();
        let Operand = self.ParseUnaryExpr();
        if (Operand.is_some()){
            return Some(ExprAST::UnaryExpr { Opcode: Opc.to_string(), Operand: Box::new(Operand.unwrap())});
        }
        return self.LogErrorExprAST("Can not compile operand");
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
        let LHS_EXPR = self.ParseUnaryExpr();
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

        let mut BinOp = self.current_token;
        let charBinOp = self.lexer.slice();
        
        if(self.GetTokPrecedence() == -1){
            //Handle Error
        }

        if(!"+-/*<>".contains(self.lexer.slice())){
            BinOp = Some(Token::CustomBinOp);
        }
        self.getNewToken();
        
        let mut RHS = self.ParseUnaryExpr();

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
        LHS = Some(ExprAST::BinaryExpr { op: BinOp.unwrap(), lhs: LHS_BOX, rhs: RHS_BOX, opChar: charBinOp.to_string() })
        }
    }

    pub fn UpdateSourceString(&mut self, newSource: &'a String){
        self.lexer = Token::lexer(&newSource);
    }

    pub fn ParseIfElseExpr(&mut self) -> Option<ExprAST>{
        self.getNewToken(); //eat the if
        let cond = self.ParseExpr();
        if(cond.is_none()){
            self.LogErrorExprAST("Cam not parse condition");
        }
        if (self.current_token.is_none() || self.current_token.unwrap() != Token::Then){
            self.LogErrorExprAST("Expected a then here");
        }
        self.getNewToken();
        if (self.current_token.is_none() || self.current_token.unwrap() != Token::FuncBegin){
            self.LogErrorExprAST("Expected a : here");
        }
        self.getNewToken(); //eat the :
        let then = self.ParseExpr();
        if (then.is_none()){
            self.LogErrorExprAST("Could not parse then statements");
        }
        if(self.current_token.unwrap() != Token::Else){
            self.LogErrorExprAST("Expected an else here");
        }
        self.getNewToken(); //eat the end
        if (self.current_token.is_none() || self.current_token.unwrap() != Token::FuncBegin){
            self.LogErrorExprAST("Expected a : here");
        }
        self.getNewToken(); //eat the :
        let Else = self.ParseExpr();
        if(Else.is_none()){
            self.LogErrorExprAST("Error parsing else block");
        }
        if (self.current_token.is_none() || self.current_token.unwrap() != Token::FuncEnd){
            self.LogErrorExprAST("Expected a end here");
        }
        self.getNewToken(); //eat the end

        Some(ExprAST::IfExpr { cond: Box::new(cond.unwrap()), Then: Box::new(then.unwrap()), Else: Box::new(Else.unwrap()) })

        //Add Else Parse
        
    }

    pub fn ParseForExpr(&mut self) -> Option<ExprAST> {
        let mut inclusiveForLoop = false;
        self.getNewToken(); //Consume for

        if(self.current_token.unwrap() != Token::Ident){
            self.LogErrorExprAST("Need a identifer here");
        }
        let varName = self.lexer.slice();
        self.getNewToken(); //Eat identifer
        if(self.current_token.unwrap() != Token::Equals){
            self.LogErrorExprAST("Need a = here");
        }
        self.getNewToken(); // Eat =
        let Start = self.ParseExpr();
        if(Start.is_none()){
            self.LogErrorASTNode("Something wrong with start value of loop");
        }
        let Start = Start.unwrap();
        if(self.current_token.unwrap() != Token::ForLoopTo && self.current_token.unwrap() != Token::InclusiveForLoopTo){
            self.LogErrorExprAST("Need a -> here");
        } 
        if(self.current_token.unwrap() == Token::InclusiveForLoopTo){
            inclusiveForLoop = true;
        }
        self.getNewToken(); //Eat -> or Eat ->*
        let End = self.ParseExpr();
        if(End.is_none()){
            self.LogErrorASTNode("Something wrong with end value of loop");
        }
        let End = End.unwrap();
        if(self.current_token.unwrap() != Token::Comma){
            self.LogErrorExprAST("Need a , here");
        }
        self.getNewToken(); //Eat ,
        let stepBy = self.ParseExpr();
        if(stepBy.is_none()){
            self.LogErrorExprAST("Can't compile step-by value here");
        }
        let stepBy = stepBy.unwrap();
        if(self.current_token.unwrap() != Token::FuncBegin){
            self.LogErrorExprAST("Expected a : here");
        }
        self.getNewToken(); //Eats :
        let body = self.ParseExpr();
        if(body.is_none()){
            self.LogErrorExprAST("Something wrong with parsing body of for loop");
        }
        let body = body.unwrap();
        if(self.current_token.unwrap() != Token::FuncEnd){
            self.LogErrorExprAST("Expected a end here");
        }
        self.getNewToken(); //Eats end
        if(inclusiveForLoop){
            return Some(ExprAST::InclusiveForExpr{ var: varName.to_owned().to_string(), start: Box::new(Start.clone()), end: Box::new(End.clone()), stepFunc: Box::new(stepBy.clone()), body: Box::new(body.clone()) });
        }
        Some(ExprAST::ForExpr { var: varName.to_owned().to_string(), start: Box::new(Start.clone()), end: Box::new(End.clone()), stepFunc: Box::new(stepBy.clone()), body: Box::new(body.clone()) })
    }

}

mod tests {
    use crate::parser::Parser;
    use std::fs;
    
    
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

    #[test]
    fn checkIfParsing(){
        let source = "if boo(a) then: zoo(a) else: bar(a) end";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 1);
    }

    #[test]
    fn checkForLoopParsing(){
        let source = "for i=1->10, 1: a * i end";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 1);
    }

    #[test]
    fn checkForSingleLineComment(){
        let source = "/* This is a test */";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 1);
    }

    #[test]
    fn parseFile(){
        let contents = fs::read_to_string("exampleCode/test1.toast").expect("Expected file here");
        let mut parser = Parser::new(&contents);
        let parsedFile = parser.parse();
        println!("{:?}", parsedFile);
    }

    #[test]
    fn parseBinaryFunc(){
        let source = "def binary| 5 (LHS, RHS): \n if LHS then: 1 else: if RHS then: 1 else: 0 end end end \n 2 < 3 | 4 > 2";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        println!("{:?}", test.to_owned().unwrap()[1]);
        assert_eq!(test.unwrap().len(), 2);
    }

    #[test]
    fn parseUnaryFunc(){
        let source = "def unary!(v): \n if v then: 0 else: 1 end end";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 1);
    }
}