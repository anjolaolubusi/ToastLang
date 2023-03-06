#ifndef ASTNODE_H
#define ASTNODE_H
#include <string>
#include <map>
#include <vector>
#include "tokens.h"
#include "lexer.h"

enum AstNodeTypes{
    ASTNodeFunc,
    ASTTopLevelExpr,
    ASTProto
};

//Declaration
struct CodeVisitor;
struct ExprAST;
struct VariableExpr;
struct NumberExpr;
struct BinaryExpr;
struct CallExpr;
struct UnaryExpr;
struct CommentExpr;
struct IfExpr;
struct ForExpr;

struct ASTNode;
struct ProtoAST;
struct FuncAST;
struct ExprNode;


typedef enum {
    VariableExpr,
    NumberExpr,
} ExprType;

struct ExprAST{
    ExprAST() {}
    std::string value;
    ExprType exprType;
    ExprAST(std::string value, ExprType exprType): value(value), exprType(exprType) {}
    virtual ~ExprAST() {}
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

struct IfExpr: ExprAST{
    std::unique_ptr<ExprAST> condExpr;
    std::unique_ptr<ExprAST> thenExpr;
    std::unique_ptr<ExprAST> elseExpr;
    IfExpr(std::unique_ptr<ExprAST> condExpr, std::unique_ptr<ExprAST> thenExpr, std::unique_ptr<ExprAST> elseExpr)
        :condExpr(std::move(condExpr)), thenExpr(std::move(thenExpr)), elseExpr(std::move(elseExpr)) {}
};

struct ForExpr: ExprAST{
    std::string var;
    std::unique_ptr<ExprAST> start;
    std::unique_ptr<ExprAST> end;
    std::unique_ptr<ExprAST> cond;
    std::unique_ptr<ExprAST> stepFunc;
    std::unique_ptr<ExprAST> body;
    ForExpr(std::string var, std::unique_ptr<ExprAST> start, std::unique_ptr<ExprAST> end, std::unique_ptr<ExprAST> cond, std::unique_ptr<ExprAST> stepFunc, std::unique_ptr<ExprAST> body)
    : var(var), start(std::move(start)), end(std::move(end)), cond(std::move(cond)), stepFunc(std::move(stepFunc)), body(std::move(body)){}
};

struct UnaryExpr: ExprAST{
    std::string opCode;
    std::unique_ptr<ExprAST> operand;
    UnaryExpr(std::string opCode, std::unique_ptr<ExprAST> operand)
        :opCode(opCode), operand(std::move(operand)) {}
};

struct CommentExpr: ExprAST{
    std::string comment;
};


struct ASTNode{
    ASTNode() {}
    AstNodeTypes astNodeType;
    virtual ~ASTNode() {}
};

struct ProtoAST: ASTNode
{
    std::string name;
    std::vector<std::string> args;
    bool isOperator;
    int Precedence;

    ProtoAST(std::string name, std::vector<std::string> args, bool isOperator, int Precedence)
        : name(name), args(args), isOperator(isOperator), Precedence(Precedence) {
            this->astNodeType = AstNodeTypes::ASTProto;
        }
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
        : Proto(std::move(Proto)), Body(std::move(Body)) {
            this->astNodeType = AstNodeTypes::ASTNodeFunc;
        }
};

struct ExprNode : ASTNode{
    std::unique_ptr<ExprAST> expr;
    ExprNode(std::unique_ptr<ExprAST> expr)
        : expr(std::move(expr)) {
            this->astNodeType = ASTTopLevelExpr;
        }
};



#endif