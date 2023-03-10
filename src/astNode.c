#include "astNode.h"
void deallocateExpr(ExprAST* node){
    if(node == NULL){
        return;
    }
    // for(int i = 0; i < 5; i++){
    //     deallocateExpr(node->children[i]);
    // }
    free(node);
}