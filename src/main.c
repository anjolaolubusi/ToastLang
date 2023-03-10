#include <stdlib.h>
#include "tokens.h"
#include "codeGen.h"


int main(int argc, char* argv[]) {
    printf("Number of args: %i, Args: ", argc);
    for(int i = 0; i < argc; i++){
        printf("%s", argv[i]);
        if(i != argc-1){
            printf(", ");
        }
    }
    printf("\n");
    CodeGen codeGen;
    if(argc > 2){
        printf("Usage: \n ToastLang (Opens the shell) \n ToastLang [file] (Compiles file)");
        return 0;
    }
    return 0;
}
