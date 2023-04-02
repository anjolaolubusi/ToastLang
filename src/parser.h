#ifndef PARSER_H
#define PARSER_H

#include <string>
#include <map>
#include <vector>
#include "tokens.h"
#include "lexer.h"
#include <memory>
#include "astNode.h"

// struct VariableExpr;
// struct NumberExpr;
// struct BinaryExpr;

// struct CodeVisitor{
//     virtual llvm::Value* visit(VariableExpr&) = 0;
// };


// struct ExprAST{
//     ExprAST() {}
//     virtual ~ExprAST() {}
//     virtual llvm::Value* compile(CodeVisitor&) = 0;
// };

// struct ASTNode{
//     ASTNode() {}
//     virtual ~ASTNode() {}
// };
// struct ProtoAST;
// struct FuncAST;
// struct ExprNode;

class Parser
{
private:
    /* data */
public:
    Parser(/* args */);
    // ~Parser();
    std::unique_ptr<ExprAST> LogError(const char* str);
    std::unique_ptr<ASTNode> LogErrorASTNode(const char* str);
    std::unique_ptr<ProtoAST> LogErrorProto(const char* str);
    
    std::unique_ptr<ExprAST> ParseNumberExpr();
    std::unique_ptr<ExprAST> ParseIdentExpr();
    std::unique_ptr<ProtoAST> ParsePrototype();
    std::unique_ptr<ExprAST> ParsePrimaryExpr();
    std::unique_ptr<ExprAST> ParseBinOpRHS(int ExprPrec, std::unique_ptr<ExprAST> LHS);
    std::unique_ptr<ExprAST> ParseUnaryExpr();
    std::unique_ptr<ExprAST> ParseIfElseExpr();
    std::unique_ptr<ExprAST> ParseExpr();
    std::unique_ptr<ASTNode> ParseDef();
    std::unique_ptr<ProtoAST> ParseExtern();
    std::unique_ptr<ExprNode> ParseTopLevel();
    
    void parse(std::vector<LexedToken> lexedTokens);
    void getNextToken();

    int GetBinOPPrec();

    std::map<std::string, int> BinOpMap;
    std::vector<std::unique_ptr<ASTNode>> parsedTokens;
    std::vector<LexedToken>::iterator currentTokenITR;
    std::vector<LexedToken>::iterator endOfLexedTokenITR;
    std::vector<LexedToken> lexedTokens;
    //Lexer lex;
};

#endif