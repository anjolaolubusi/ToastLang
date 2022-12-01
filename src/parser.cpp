#include "parser.h"
struct VariableExpr: ExprAST
{
    std::string varName;
    VariableExpr(std::string varName): varName(varName) {}
};

struct NumberExpr: ExprAST
{
    std::string number;
    NumberExpr(std::string number) : number(number) {}
}; 

struct BinaryExpr: ExprAST{
    Token op;
    std::unique_ptr<ExprAST> lhs, rhs;
    std::string opChar;
    BinaryExpr(Token op, std::unique_ptr<ExprAST> lhs, std::unique_ptr<ExprAST> rhs, std::string opChar)
        : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)), opChar(opChar) {}
};

struct CallExpr: ExprAST{
    std::string funcName;
    std::vector<std::unique_ptr<ExprAST>> parameters;
    CallExpr(std::string funcName, std::vector<std::unique_ptr<ExprAST>> parameters)
        :funcName(funcName), parameters(std::move(parameters)) {}
};

// struct IfExpr: ExprAST{
//     ExprAST condExpr;
//     ExprAST thenExpr;
//     ExprAST elseExpr;
// };

// struct ForExpr: ExprAST{
//     std::string var;
//     ExprAST start;
//     ExprAST end;
//     ExprAST stepFunc;
//     ExprAST body;
// };

struct UnaryExpr: ExprAST{
    std::string opCode;
    std::unique_ptr<ExprAST> operand;
    UnaryExpr(std::string opCode, std::unique_ptr<ExprAST> operand)
        :opCode(opCode), operand(std::move(operand)) {}
};

struct CommentExpr: ExprAST{
    std::string comment;
};


struct ProtoAST: ASTNode
{
    std::string name;
    std::vector<std::string> args;
    bool isOperator;
    int Precedence;

    ProtoAST(std::string name, std::vector<std::string> args, bool isOperator, int Precedence)
        : name(name), args(args), isOperator(isOperator), Precedence(Precedence) {}
    bool isUnaryOP(){
        return isOperator && args.size() == 1;
    }
    bool isBinaryOP(){
        return isOperator && args.size() == 2;
    }
    std::string getOperatorName(){
        std::string operatorName = name;
        if(isBinaryOP()){
            operatorName = operatorName.erase(0, 6);
        }
        if(isUnaryOP()){
            operatorName = operatorName.erase(0, 5);
        }
        return operatorName;
    }
    
};



struct FuncAST : ASTNode
{
    std::unique_ptr<ProtoAST> Proto;
    std::unique_ptr<ExprAST> Body;
    FuncAST(std::unique_ptr<ProtoAST> Proto, std::unique_ptr<ExprAST> Body)
        : Proto(std::move(Proto)), Body(std::move(Body)) {}
};

struct ExprNode : ASTNode{
    std::unique_ptr<ExprAST> expr;
    ExprNode(std::unique_ptr<ExprAST> expr)
        : expr(std::move(expr)) {}
};


Parser::Parser(){
    BinOpMap["<"] = 10;
    BinOpMap[">"] = 10;
    BinOpMap["+"] = 20;
    BinOpMap["-"] = 20;
    BinOpMap["*"] = 40;
    BinOpMap["/"] = 30;
    BinOpMap["="] = 10;
}

std::unique_ptr<ExprAST> Parser::LogError(const char* str){
    fprintf(stderr, "LogError: %s\n", str);
    return nullptr;
}

std::unique_ptr<ASTNode> Parser::LogErrorASTNode(const char* str){
    this->LogError(str);
    return nullptr;
}

std::unique_ptr<ProtoAST> Parser::LogErrorProto(const char* str){
    this->LogError(str);
    return nullptr;
}

 std::unique_ptr<ExprAST> Parser::ParseNumberExpr(){
  std::unique_ptr<NumberExpr> numExpr = std::make_unique<NumberExpr>(currentTokenITR->value);
  this->getNextToken(); //Consume number
  return std::move(numExpr);
 }

 std::unique_ptr<ExprAST> Parser::ParseIdentExpr(){
    std::string IdName = currentTokenITR->value;
    getNextToken(); //Consume Identifer
    if(currentTokenITR->token != tok_openingPara /*|| Add condition to check if end of parse line */){
        return std::make_unique<VariableExpr>(IdName);
    }
  this->getNextToken(); //Consumes '('
  std::vector<std::unique_ptr<ExprAST>> newArgs;
  while(true){
    newArgs.push_back(std::move(ParseExpr()));
    if(currentTokenITR->token != tok_comma){
        break;
    }
    getNextToken();
  }
  if( currentTokenITR->token != tok_closingPara){
    LogError("Expected a '(' here");
  }
  getNextToken(); //Consume ')'
  
  return std::move(std::make_unique<CallExpr>(IdName, std::move(newArgs)));
 }

std::unique_ptr<ProtoAST> Parser::ParsePrototype() {
    int Kind = 0;
    int BinaryPrecedence = 0;
    std::string prototypeName = "";
    std::string opName = "";

    switch (currentTokenITR->token)
    {
    case tok_ident:
        prototypeName = currentTokenITR->value;
        Kind = 0;
        getNextToken(); //Consume Identifer
        break;
    case tok_binary:
        getNextToken(); //Consume Binary
        opName = currentTokenITR->value;
        prototypeName = "binary" + opName;
        Kind = 2;
        getNextToken(); //Consume Operator
        BinaryPrecedence = stoi(currentTokenITR->value);
        if (BinaryPrecedence < 1 || BinaryPrecedence > 100){
            return LogErrorProto("Invalid precedence must be between 1 and 100 inclusive");
        }
        getNextToken(); //Consume Binary precedence
        break;
    case tok_unary:
        getNextToken(); //Consume unary
        opName = currentTokenITR->value;
        for(int i = 0; i < opName.length(); i++){
            if(isalnum(opName[i])){
                return LogErrorProto("Unary operator must have alphanumeric characters");
            }
        }
        prototypeName = "unary" + opName;
        Kind = 1;
        getNextToken();
        break;
    default:
        return LogErrorProto("Expected function name here");
        break;
    }

    if(currentTokenITR->token != tok_openingPara){
        return LogErrorProto("Expected a '(' here");
    }
    getNextToken(); //Consume '('
    std::vector<std::string> newArgs;
    while(currentTokenITR->token != Token::tok_closingPara){
        switch (currentTokenITR->token)
        {
        case tok_ident:
            newArgs.push_back(currentTokenITR->value);
            getNextToken();
            break;
        case tok_comma:
            getNextToken();
            break;
        default:
            break;
        }
    }
    if(currentTokenITR->token != tok_closingPara){
        return LogErrorProto("Expected a ')' here");
    }
    getNextToken(); //Consume ')'
    if(newArgs.size() != Kind && Kind != 0){
        LogErrorProto("Invalid number of operands for operator");
    }
    if(Kind == 2){
        BinOpMap[opName] = BinaryPrecedence;
    }
    return std::make_unique<ProtoAST>(prototypeName, newArgs, Kind != 0, BinaryPrecedence);
}

std::unique_ptr<ExprAST> Parser::ParsePrimaryExpr(){
    if(currentTokenITR->token == tok_ident){
        std::unique_ptr<ExprAST> identExpr = ParseIdentExpr();
        return std::move(identExpr);        
    }

    if(currentTokenITR->token == tok_number){
        std::unique_ptr<ExprAST> numExpr = ParseNumberExpr();
        return std::move(numExpr);
    }

    if(currentTokenITR->token == tok_openingPara){
        getNextToken(); //Consume '('
        std::unique_ptr<ExprAST> expr = ParseExpr();
        if (currentTokenITR->token != tok_closingPara){
            return LogError("Expected a ')' here");
        }
        getNextToken(); //Consume ')'
        return std::move(expr);
    }

    LogError("Can not parse unkown token");
    return nullptr;
}

std::unique_ptr<ExprAST> Parser::ParseUnaryExpr() {
    std::vector<Token> TokensToPass = {tok_openingPara, tok_comma, tok_comment, tok_multlineCommentBegin, tok_ident, tok_number};
    if(std::find(TokensToPass.begin(), TokensToPass.end(), currentTokenITR->token) != TokensToPass.end()){
        return ParsePrimaryExpr();
    }
    std::string Opc = currentTokenITR->value;
    getNextToken(); //Consume operand
    std::unique_ptr<ExprAST> Operand = ParseUnaryExpr();
    return std::make_unique<UnaryExpr>(Opc, std::move(Operand));
}

std::unique_ptr<ExprAST> Parser::ParseBinOpRHS(int ExprPrec, std::unique_ptr<ExprAST> LHS){
while(true){
 int TokPrec = GetBinOPPrec();
 if (TokPrec < ExprPrec) {
    return LHS;
 }

 Token BinOp = currentTokenITR->token;
 std::string charBinOp = currentTokenITR->value;
 getNextToken(); //Consumes Binary Operator

 std::unique_ptr<ExprAST> RHS = ParseUnaryExpr();
 if (RHS == nullptr){
    return nullptr;
 }


 int NextPrec = GetBinOPPrec();
 if (TokPrec < NextPrec){
    RHS = ParseBinOpRHS(TokPrec+1, std::move(LHS));
    if (RHS == nullptr){
        return nullptr;
    }
 }
 LHS = std::make_unique<BinaryExpr>(BinOp, std::move(LHS), std::move(RHS), charBinOp);
}
}

std::unique_ptr<ExprAST> Parser::ParseExpr() {
    std::unique_ptr<ExprAST> LHS_EXPR = ParseUnaryExpr();
    if(LHS_EXPR == nullptr){
        LogError("Can not parse LHS_EXPR");
    }
    return ParseBinOpRHS(0, std::move(LHS_EXPR));
}

std::unique_ptr<ASTNode> Parser::ParseDef() {
    getNextToken();
    std::unique_ptr<ProtoAST> prototype = ParsePrototype();
    if (currentTokenITR->token != tok_funcBegin){
        LogErrorASTNode("Expected a ':' here");
    }
    getNextToken(); //Consume ':'
    std::unique_ptr<ExprAST> body = ParseExpr();
    if (currentTokenITR->token != tok_funcEnd){
        LogError("Expected a 'end' here");
    }
    getNextToken(); //Consume 'end'
    return std::make_unique<FuncAST>(std::move(prototype), std::move(body));
}

std::unique_ptr<ProtoAST> Parser::ParseExtern(){
    getNextToken(); //Consume 'extern'
    return std::move(ParsePrototype());
}

std::unique_ptr<ExprNode> Parser::ParseTopLevel() {
    return std::make_unique<ExprNode>(std::move(ParseExpr()));
}

 void Parser::parse(std::vector<LexedToken> lexedTokens){
    //While loop
    this->lexedTokens = lexedTokens;
    currentTokenITR = lexedTokens.begin();
    endOfLexedTokenITR = lexedTokens.end();
    while(currentTokenITR != endOfLexedTokenITR){
        std::unique_ptr<ASTNode> astNode;
        switch (currentTokenITR->token)
        {
        case tok_def:
            astNode = ParseDef();
            break;
        case tok_extern:
            astNode = ParseExtern();
            break;
        default:
            astNode = ParseTopLevel();
            break;
        }
        parsedTokens.push_back(std::move(astNode));
    }
 }

 void Parser::getNextToken(){
    if(currentTokenITR != endOfLexedTokenITR){
        currentTokenITR++;
    }else{
        lexedTokens.clear();
        currentTokenITR = lexedTokens.begin();
        endOfLexedTokenITR = lexedTokens.end();
    }
 }

int Parser::GetBinOPPrec(){
    if(currentTokenITR == endOfLexedTokenITR){
        return -1;
    }
    std::map<std::string, int>::const_iterator it = BinOpMap.find(currentTokenITR->value);
    if(it == BinOpMap.end()){
        return -1;
    }else{
        return it->second;
    }
}