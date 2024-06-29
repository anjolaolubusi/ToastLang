#![allow(non_snake_case)]
#![allow(unused_parens)]
use logos::{Lexer, Logos};
use crate::lexer::Token;
use std::collections::HashMap;


///Expression AST node
#[derive(PartialEq, Clone, Debug)]
pub enum ExprAST {
    // Represents a number experssion ast node
    NumberExpr(f64),
    ///Represents a variable experssion ast node
    VariableExpr(String),
    VariableAssignExpr {
        varObject: Box<ExprAST>,
        value: Box<ExprAST>
    },
    VariableHeader {
        name: String,
        typeName: String
    },
    ///Represents a binary expression ast node
    BinaryExpr {
        ///Token for operation
        op: Token,
        /// Represents left hand of operation
        lhs: Box<ExprAST>,
        /// Represents right hand of operation
        rhs: Box<ExprAST>,
        /// Represents the string character of the operation 
        opChar: String},
    ///Represents a function call expression ast node
    CallExpr {
        /// Name of called function
        func_name: String,
        /// Vector of parameters
        parameters: Vec<ExprAST>},
    ///Represents a if statment expression ast node
    IfExpr{
        /// Condition expression ast node
        cond: Box<ExprAST>,
        /// Expression ast node for statments if the condition is true
        Then: Box<ExprAST>,
        /// Represents optional else statement
        Else: Option<Box<ExprAST>>},
    ///Represents a for loop statment
    ForExpr{
        ///Name of iterator variable
        var: String, 
        //Starting value
        start: Box<ExprAST>,
        ///Ending value
        end: Box<ExprAST>,
        ///Stepping function
        stepFunc: Box<ExprAST>,
        ///Body of the for loop
        body: Box<ExprAST>},
    ///Represents a unary expression 
    UnaryExpr {
        ///Represents character of unary operation
        Opcode: String,
        ///Represents operand 
        Operand: Box<ExprAST>},
    ///Represents comments
    CommentExpr(String),
    ///Represents Functions
    FuncExpr {
        ///Name of function
        name: String,
        ///List of Arugments
        args: Vec<ExprAST>,
        ///Body of Functions
        // body: Option<Box<ExprAST>>
        body: Vec<ExprAST>
    }
}

///Parser object
#[derive(Clone, Debug)]
pub struct Parser<'a>{
    ///List of tokens
    pub tokens: Vec<Token>,
    ///Currently parsed token
    pub current_token: Option<Token>,
    ///Lexer 
    pub lexer: Lexer<'a, Token>,
    ///Hashmap of binary operator precedence
    pub BinOpPrecedence: HashMap<String, i64>,
    ///List of tokens to skip over
    pub TokensToSkip: Vec<Token>,
    pub line_num: usize,
    pub col_num: usize
}

impl<'a> Parser <'a>{
    /// Instantiate a Parser object 
    pub fn new(input: &'a str) -> Self{
        let mut BinOp = HashMap::new();
        BinOp.insert("<".to_string(), 10);
        BinOp.insert(">".to_string(), 10);
        BinOp.insert("+".to_string(), 20);
        BinOp.insert("-".to_string(), 20);
        BinOp.insert("*".to_string(), 40);
        BinOp.insert("/".to_string(), 30);
        BinOp.insert("=".to_string(), 10);

        let skipToken = [Token::WhiteSpace].to_vec();
        Parser {
            tokens: Vec::<Token>::new()
            ,current_token: Some(Token::WhiteSpace)
            ,lexer: Token::lexer(input)
            ,BinOpPrecedence: BinOp.clone()
            ,TokensToSkip: skipToken.clone()
            ,line_num: 0
            ,col_num: 0
        }
    }

    /// Gets the next token
    pub fn getNewToken(&mut self){
        loop{
        self.current_token = self.lexer.next();
        if self.lexer.slice().contains('\n'){
            self.line_num += 1;
            self.col_num = 0;
        }
        if self.current_token.is_none() || !self.TokensToSkip.contains(&self.current_token.unwrap()) {
            break;
        }

        }
        self.col_num += (self.lexer.span().end - self.lexer.span().start)
    }

    pub fn LogError(&mut self, error : &str) -> Option<ExprAST>{
        println!("(Line Num {}, Col {}): Error: {} Col", self.line_num, self.col_num, error);
        return None;
    }

    /// Parses given string
    pub fn parse(&mut self) -> Option<Vec<ExprAST>> {
        let mut program: Vec<ExprAST> = Vec::new();
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
                Token::Def => self.ParseDef(),
                _ => self.ParseExpr()
            };
            if result.is_none() {
                return  None;
            }
            program.push(result.clone().unwrap());
        }
        return Some(program);
    }
    /// Parses a single line comment
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
    /// Parses a multi line comment
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

    /// Parse function declaration
    pub fn ParseDef(&mut self) -> Option<ExprAST>{
        self.getNewToken(); //Consume Def
        let mut prototype : ExprAST = self.ParseFunctionHeader().expect("Could not parse function prototype");
        if self.current_token.unwrap() != Token::FuncBegin {
            return self.LogError("Expected a ':' here");
        }
        self.getNewToken(); //Consume ':'
        //TODO: Change body to allow multiple statments
        // let funcBody = self.ParseExpr().expect("Could not parse body");
        let mut funcBody = Vec::<ExprAST>::new();
        while self.current_token.unwrap() != Token::FuncEnd{
            let curExpr = self.ParseExpr();
            if curExpr.is_none() {
                return self.LogError(("Could not parse body"));
            } 
            funcBody.push(curExpr.unwrap());
        }
        if self.current_token.is_none() || self.current_token.unwrap() != Token::FuncEnd {
            return self.LogError("Expected a 'end' here");
        }
        self.getNewToken(); //Consume End

        if let ExprAST::FuncExpr { name: _, args: _, ref mut body } = prototype {
            *body = funcBody.clone();
        }

        return Some(prototype);
    }
    /// Parses funciton prototype
    pub fn ParseFunctionHeader(&mut self) -> Option<ExprAST>{
        let prototypeName: String;

        match self.current_token.unwrap() {
            Token::Ident => {
                prototypeName = self.lexer.slice().to_owned();
                self.getNewToken(); //Consume Identifer
            },
            _ => {return self.LogError("Expected function name here")}
        }
        

        if self.current_token.unwrap() != Token::OpeningParenthesis {
            return self.LogError("Expected a '(' here");
        }
        self.getNewToken(); //Consume '('
        let mut newArgs: Vec<ExprAST> = Vec::new();
        loop{
            match self.current_token.unwrap() {
                Token::Ident => {
                    let arg = self.ParseIdentExpr().unwrap();
                    if let ExprAST::VariableHeader { name: _, typeName: _ } = arg.clone() {
                        // newArgs.push(self.ParseIdentExpr().unwrap());
                        newArgs.push(arg.clone());
                    } else {
                        self.LogError("Expected something like [Variable Name] : [Type]");
                    }
                },
                Token::Comma => self.getNewToken(),
                _ => break
            }
        }
        if self.current_token.unwrap() != Token::ClosingParenthesis {
            return self.LogError("Expected a ')' here");
        }
        self.getNewToken(); //Consume ')'
        let funcExpression : ExprAST = ExprAST::FuncExpr { name: prototypeName, args: newArgs, body: Vec::<ExprAST>::new() };
        return Some(funcExpression);
        
    }
    /// Parses primary expression
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
                let expr = self.ParseExpr().expect("Could not parse expression within parenthesis");
                if self.current_token.unwrap() != Token::ClosingParenthesis {
                    return self.LogError("Expected a ')' here");
                }
                self.getNewToken(); //Consumes ')'
                return Some(expr);
            },
            Token::If => self.ParseIfElseExpr(),
            Token::For => self.ParseForExpr(),
            Token::Comment => self.ParseSingleLineComment(),
            Token::MultilineCommentBegin => self.ParseMultiLineComment(),
            Token::VarDeclare => self.ParseVarDeclar(),
            _ => {
                println!("Unkown Token: {:?} ", self.current_token.unwrap());
                return self.LogError("Unkown Token");
            }
        }
    }
    /// Parses unary expression
    pub fn ParseUnaryExpr(&mut self) -> Option<ExprAST>{
        if(!self.lexer.slice().is_ascii() || self.current_token.unwrap() == Token::Number || self.lexer.slice().chars().all(char::is_alphanumeric) || self.current_token.unwrap() == Token::OpeningParenthesis || self.current_token.unwrap() == Token::Comma || self.current_token.unwrap() == Token::Comment || self.current_token.unwrap() == Token::MultilineCommentBegin){
            return self.ParsePrimaryExpr();
        }

        let Opc = self.lexer.slice();
        self.getNewToken();
        let Operand = self.ParseUnaryExpr().expect("Could not parse Operand");
        return Some(ExprAST::UnaryExpr { Opcode: Opc.to_string(), Operand: Box::new(Operand)});
    }
    /// Parses identifier
    pub fn ParseIdentExpr(&mut self) -> Option<ExprAST>{
        let IdName = self.lexer.slice().to_owned();
        self.getNewToken(); //Consume Ident
        if self.current_token.is_none() || !vec![Token::OpeningParenthesis, Token::FuncBegin].contains(&self.current_token.unwrap()) {
            return Some(ExprAST::VariableExpr(IdName));
        }
        if self.current_token.unwrap() == Token::OpeningParenthesis {
            self.getNewToken(); //Consume '('
            let mut newArgs: Vec<ExprAST> = Vec::new();
            loop{
                let parameter = self.ParseExpr().expect("Could not parse parameter");
                newArgs.push(parameter);
                if self.current_token.unwrap() != Token::Comma {
                    break;
                }
                self.getNewToken(); //Consume Comma
            }
            if self.current_token.unwrap() != Token::ClosingParenthesis {
                return self.LogError("Expected a '(' here");
            }
            self.getNewToken(); //Consume ')'
            return Some(ExprAST::CallExpr { func_name: IdName, parameters: newArgs.clone() })
        }

        if self.current_token.unwrap() == Token::FuncBegin {
            // consume :
            self.getNewToken();
            let TypeName = self.lexer.slice().to_owned();
            // consumes type
            self.getNewToken();
            return Some(ExprAST::VariableHeader { name:IdName, typeName: TypeName });
        }

        return  None;
    }
    /// Returns binary operation precedence
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
    /// Parses expression
    pub fn ParseExpr(&mut self) -> Option<ExprAST>{
        let LHS_EXPR = self.ParseUnaryExpr();
        if LHS_EXPR.is_none() {
            return None;
        }
        //self.getNewToken(); //Eat LHS
        return self.ParseBinOpRHS(0, LHS_EXPR);
    }
    /// Parse right hand side of expression
    pub fn ParseBinOpRHS(&mut self, ExprPrec: i64, mut LHS: Option<ExprAST>) -> Option<ExprAST>{
        loop{
        let TokPrec = self.GetTokPrecedence();

        if TokPrec < ExprPrec {
            return LHS;
        }

        let mut BinOp = self.current_token;
        let charBinOp = self.lexer.slice();
        
        if(self.GetTokPrecedence() == -1){
            return  self.LogError("Unkown Operator");
        }

        if(!"+-/*<>=".contains(self.lexer.slice())){
            BinOp = Some(Token::CustomBinOp);
        }
        self.getNewToken();
        
        let mut RHS = self.ParseUnaryExpr();

        if RHS.is_none() {
            return self.LogError("Empty Right Hand of Equation");
        }

        let NextPrec = self.GetTokPrecedence();
        if TokPrec < NextPrec {
            RHS = self.ParseBinOpRHS(TokPrec + 1, RHS.clone());
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

    /// Parse if expression
    pub fn ParseIfElseExpr(&mut self) -> Option<ExprAST>{
        self.getNewToken(); //eat the if
        let cond = self.ParseExpr().expect("Can not parse condition");
        if (self.current_token.is_none() || self.current_token.unwrap() != Token::FuncBegin){
            self.LogError("Expected a : here");
        }
        self.getNewToken(); //eat the :
        let then = self.ParseExpr().expect("Could not parse then statements");
        if(self.current_token.unwrap() != Token::Else && self.current_token.unwrap() != Token::EndIf){
            self.LogError("Expected an 'else' or 'endif' here");
        }
        if(self.current_token.unwrap() == Token::EndIf){
            self.getNewToken();
            return Some(ExprAST::IfExpr { cond: Box::new(cond), Then: Box::new(then), Else: None });
        }
        self.getNewToken(); //eat the 'else'
        if (self.current_token.is_none() || self.current_token.unwrap() != Token::FuncBegin){
            self.LogError("Expected a : here");
        }
        self.getNewToken(); //eat the :
        let Else = self.ParseExpr().expect("Error parsing else block");
        if (self.current_token.is_none() || self.current_token.unwrap() != Token::EndIf){
            self.LogError("Expected a 'endif' here");
        }
        self.getNewToken(); //eat the endif

        Some(ExprAST::IfExpr { cond: Box::new(cond), Then: Box::new(then), Else: Some(Box::new(Else)) })

        //Add Else Parse
        
    }
    /// Parses for loop expression
    pub fn ParseForExpr(&mut self) -> Option<ExprAST> {
        self.getNewToken(); //Consume for

        if(self.current_token.unwrap() != Token::Ident){
            self.LogError("Need a identifer here");
        }
        let varName = self.lexer.slice();
        self.getNewToken(); //Eat identifer
        if(self.current_token.unwrap() != Token::Equals){
            self.LogError("Need a = here");
        }
        self.getNewToken(); // Eat =
        let Start = self.ParseExpr().expect("Something wrong with start value of loop");
        if(self.current_token.unwrap() != Token::PointTo){
            self.LogError("Need a -> here");
        } 
        self.getNewToken(); //Eat -> 
        let End = self.ParseExpr().expect("Something wrong with end value of loop");
        if(self.current_token.unwrap() != Token::Comma){
            self.LogError("Need a , here");
        }
        self.getNewToken(); //Eat ,
        let stepBy = self.ParseExpr().expect("Can't parse step-by value here");
        if(self.current_token.unwrap() != Token::FuncBegin){
            self.LogError("Expected a : here");
        }
        self.getNewToken(); //Eats :
        let body = self.ParseExpr().expect("Something wrong with parsing body of for loop");
        if(self.current_token.unwrap() != Token::FuncEnd){
            self.LogError("Expected a end here");
        }
        self.getNewToken(); //Eats end
        Some(ExprAST::ForExpr { var: varName.to_owned().to_string(), start: Box::new(Start.clone()), end: Box::new(End.clone()), stepFunc: Box::new(stepBy.clone()), body: Box::new(body.clone()) })
    }

    pub fn ParseVarDeclar(&mut self) -> Option<ExprAST>{
        self.getNewToken(); //consume 'let'
        let mut newVarExpr = self.ParseExpr()?; //Parses variable declaration
        if let ExprAST::BinaryExpr { ref mut op, ref mut lhs, ref mut rhs, opChar: _ } = newVarExpr {
            *op = Token::VarDeclare;
            if let ExprAST::VariableHeader { name: _, typeName: _ } = *lhs.to_owned(){
                let temp = ExprAST::VariableAssignExpr { varObject: Box::new(*lhs.clone()), value: Box::new(*rhs.clone()) };
                return Some(temp);
            }else{
                return self.LogError("Left hand needs to be in format: let [Varible name] : [Type]");
            }
        } else {return self.LogError("Error caused by wrong Expr variant");}
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
        use std::fs;
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

    #[test]
    fn parseVarDeclare(){
        let source = "let a = 5";
        let mut parser = Parser::new(source);
        let test = parser.parse();
        println!("{:?}", test);
        assert_eq!(test.unwrap().len(), 1);
    }
}