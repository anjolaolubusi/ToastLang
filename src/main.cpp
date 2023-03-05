#include <iostream>
#include <fstream>
#include "lexer.h"
#include "tokens.h"
#include "parser.h"
#include <vector>
#include <string>
#include "codegen.h"
#include "llvm/IR/Function.h"


#ifdef _WIN32
#define DLLEXPORT __declspec(dllexport)
#else
#define DLLEXPORT
#endif

/// putchard - putchar that takes a double and returns 0.
extern "C" DLLEXPORT double putchard(double X) {
  fputc((char)X, stderr);
  return 0;
}

/// printd - printf that takes a double prints it as "%f\n", returning 0.
extern "C" DLLEXPORT float printTest(float X) {
  fprintf(stderr, "%f\n", X);
  fprintf(stdout, "%f\n", X);
  printf("This is a test \n");
  return 0;
}


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
                if(parser.parsedTokens.back()->astNodeType == AstNodeTypes::ASTTopLevelExpr){
                FnIR->eraseFromParent();
                }
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
