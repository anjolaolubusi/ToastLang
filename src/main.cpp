#include <iostream>
#include <fstream>
#include "lexer.h"
#include "tokens.h"
#include "parser.h"
#include <vector>
#include <string>
#include "codegen.h"
#include "llvm/IR/Function.h"

int main(int argc, char** argv) {
    printf("Number of args: %i, Args: ", argc);
    for(int i = 0; i < argc; i++){
        printf("%s", argv[i]);
        if(i != argc-1){
            printf(", ");
        }
    }
    printf("\n");
    Lexer lex;
    Parser parser;
    CodeGenerator codeGen;
    if(argc > 2){
        printf("Usage: \n ToastLang (Opens the shell) \n ToastLang [file] (Compiles file)");
        return 0;
    }else if(argc == 1){
        std::string line; //Stores entered line
        while(true){ 
        std::vector<LexedToken> lexedLine = lex.lex();
        if(lexedLine.empty()){
            continue;
        }
        parser.parse(lexedLine);
            if(llvm::Function* FnIR = parser.parsedTokens.back()->compile(codeGen)){
                fprintf(stdout, "Printing LLVM IR Output: \n");
                FnIR->print(llvm::errs());
            }
        }
    } else{
        // Input fle file
        const char* fileName = argv[1];
        lex.fileName = fileName;
        lex.fileStream = std::ifstream(lex.fileName);        
        if(lex.fileStream.is_open()){
                std::vector<LexedToken> lexedfile = lex.lex();
                parser.parse(lexedfile);
        }else{
            std::cout << "Unable to open file" << std::endl;
        }
    }
}
