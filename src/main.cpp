#include <iostream>
#include <fstream>
#include "lexer.h"
#include "tokens.h"
#include <vector>
#include <string>


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

    if(argc > 2){
        printf("Usage: \n ToastLang (Opens the shell) \n ToastLang [file] (Compiles file)");
        return 0;
    }else if(argc == 1){
        std::string line; //Stores entered line
        std::vector<std::string> str_vec; //Vector that stored entered strings
        while(true){
        std::cout << "> "; 
        while(std::getline(std::cin, line)){ // Grabs user input
            if(line.empty()){
                break;
            }
            lex.lexLine(line);
            str_vec.push_back(line);
        }

        str_vec.clear();
        
        }


    } else{
        // Input fle file
        const char* fileName = argv[1];
        std::ifstream toastFile (fileName);
        if(toastFile.is_open()){
            std::string line; //Stores entered line
            std::vector<std::string> str_vec; //Vector that stored entered strings
            while(std::getline(toastFile, line)){
                lex.lexLine(line);
                str_vec.push_back(line);
            }
            toastFile.close();
        }else{
            std::cout << "Unable to open file" << std::endl;
        }
    }
}
