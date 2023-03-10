#include "codeGen.h"

void openCodeGenFile(CodeGen* codeGen, const char* filePath){
    codeGen->filePointer = fopen(filePath, "wb");
    if(codeGen->filePointer == NULL){
        printf("file can not be opened\n");
        exit(1);
    }
}

void closeCodeGenFile(CodeGen* codeGen){
    fclose(codeGen->filePointer);
}

void writeToFile(CodeGen* codeGen, uint16_t instruction){
    fwrite(&instruction, sizeof(uint16_t), 1, codeGen->filePointer);
}

int compileExpr(ExprAST* expr, CodeGen* codeGen){
    // switch (expr->exprType)
    // {
    // case ExprType::NumberExpr : {
    //     char *ptr;
    //     int value = strtol(expr->value.c_str(), &ptr, 10);
    //     int regIndex = -1;
    //     for(int i = 0; i < 8; i++){
    //         if(!codeGen->reg[i]){
    //             regIndex = i;
    //             break;
    //         }
    //     }
    //     uint16_t instruction = (OP_LOAD << 12) | (regIndex << 8) | (value);
    //     writeToFile(codeGen, instruction);
    //     return regIndex;
    // }
    // case ExprType::BinaryExpr : {
    //     int l_reg = 0;
    // }    
    // default:
    //     printf("ExprType compliation not developed");
    //     break;
    // }
    return 0;
}