#include "codeGen.h"

void openCodeGenFile(CodeGen* codeGen, const char* filePath){
    codeGen->filePointer = fopen(filePath, "w");
    if(codeGen->filePointer == NULL){
        printf("file can not be opened\n");
        exit(1);
    }
}

void closeCodeGenFile(CodeGen* codeGen){
    fclose(codeGen->filePointer);
}

void writeToFile(CodeGen* codeGen, uint16_t instruction){
    fputc(instruction, codeGen->filePointer);
}

void compileExpr(ExprAST* expr, CodeGen* codeGen){
    switch (expr->exprType)
    {
    case ExprType::NumberExpr : {
        char *ptr;
        int value = strtol(expr->value.c_str(), &ptr, 10);
        uint16_t instruction = (OP_LOAD << 12) | (0 << 8) | (value);
        writeToFile(codeGen, instruction);
        break;
    }    
    default:
        printf("ExprType compliation not developed");
        break;
    }
}