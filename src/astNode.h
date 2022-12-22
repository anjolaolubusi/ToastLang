#ifndef ASTNODE_H
#define ASTNODE_H
#include <string>
#include <map>
#include <vector>
#include "tokens.h"
#include "lexer.h"
#include "llvm/IR/Value.h"

//Declaration
struct CodeVisitor;
struct ExprAST;
struct VariableExpr;
struct NumberExpr;
struct BinaryExpr;
struct CallExpr;
struct UnaryExpr;
struct CommentExpr;

struct ASTNode;
struct ProtoAST;
struct FuncAST;
struct ExprNode;


// Implementation
struct CodeVisitor{
    virtual llvm::Value* visit(NumberExpr&) = 0;
    virtual llvm::Value* visit(VariableExpr&) = 0;
    virtual llvm::Value* visit(BinaryExpr&) = 0;
    virtual llvm::Value* visit(CallExpr&) = 0;

    virtual llvm::Function* visit(ProtoAST&) = 0;
    virtual llvm::Function* visit(FuncAST&) = 0;
    virtual llvm::Function* visit(ExprNode&) = 0;

};

struct ExprAST{
    ExprAST() {}
    virtual ~ExprAST() {}
    virtual llvm::Value* compile(CodeVisitor&) = 0;
};

struct VariableExpr: ExprAST
{
    std::string varName;
    VariableExpr(std::string varName): varName(varName) {}
    llvm::Value* compile(CodeVisitor& cv) override {
        return cv.visit(*this);
    }
};

struct NumberExpr: ExprAST
{
    std::string number;
    NumberExpr(std::string number) : number(number) {}
    llvm::Value* compile(CodeVisitor& cv) override {
        return cv.visit(*this);
    }
}; 

struct BinaryExpr: ExprAST{
    Token op;
    std::unique_ptr<ExprAST> lhs, rhs;
    std::string opChar;
    BinaryExpr(Token op, std::unique_ptr<ExprAST> lhs, std::unique_ptr<ExprAST> rhs, std::string opChar)
        : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)), opChar(opChar) {}
    llvm::Value* compile(CodeVisitor& cv) override {
        return cv.visit(*this);
    }
};

struct CallExpr: ExprAST{
    std::string funcName;
    std::vector<std::unique_ptr<ExprAST>> parameters;
    CallExpr(std::string funcName, std::vector<std::unique_ptr<ExprAST>> parameters)
        :funcName(funcName), parameters(std::move(parameters)) {}
    llvm::Value* compile(CodeVisitor& cv) override {
        return cv.visit(*this);
    }
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
    llvm::Value* compile(CodeVisitor& cv) override {
        return nullptr;
    }
};

struct CommentExpr: ExprAST{
    std::string comment;
    llvm::Value* compile(CodeVisitor& cv) override {
        return nullptr;
    }
};


struct ASTNode{
    ASTNode() {}
    virtual ~ASTNode() {}
    virtual llvm::Function* compile(CodeVisitor&) = 0;
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
    llvm::Function* compile(CodeVisitor& cv) override{
        return cv.visit(*this);
    }
};



struct FuncAST : ASTNode
{
    std::unique_ptr<ProtoAST> Proto;
    std::unique_ptr<ExprAST> Body;
    FuncAST(std::unique_ptr<ProtoAST> Proto, std::unique_ptr<ExprAST> Body)
        : Proto(std::move(Proto)), Body(std::move(Body)) {}
    llvm::Function* compile(CodeVisitor& cv) override{
        return cv.visit(*this);
    }
};

struct ExprNode : ASTNode{
    std::unique_ptr<ExprAST> expr;
    ExprNode(std::unique_ptr<ExprAST> expr)
        : expr(std::move(expr)) {}
    llvm::Function* compile(CodeVisitor& cv) override{
        return cv.visit(*this);
    }
};



#endif