#ifndef ASTNODE_H
#define ASTNODE_H
#include "tokens.h"
#include <stdlib.h>


typedef enum {
    VariableExpr = 0,
    NumberExpr,
    BinaryExpr,
    CallExpr,
    IfExpr,
    ForExpr,
} ExprType;



typedef struct{
    Token op;
    ExprType exprType;
    struct ExprAST* children;
} ExprAST;



#endif