#ifndef codegen_h
#define codegen_h



#include <stdio.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "astNode.h"


typedef enum {
    OP_RETURN = 0,
    OP_LOAD,
    OP_ADD,
    OP_SUB,
    OP_MUL,
    OP_DIV,
} OpCode;


typedef struct{
    FILE* filePointer;
} CodeGen;

void openCodeGenFile(CodeGen* codeGen, const char* filePath);
void closeCodeGenFile(CodeGen* codeGen);
void writeToCodeGenFile(CodeGen* codeGen, uint16_t instruction);
void compileExpr(ExprAST* expr, CodeGen* codeGen);

#endif