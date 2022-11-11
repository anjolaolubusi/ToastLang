#include "parser.h"
struct ExprAST
{
    /* data */
};

struct VariableExpr: ExprAST
{
    std::string varName;

};

struct NumberExpr: ExprAST
{
    std::string number;
    
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
