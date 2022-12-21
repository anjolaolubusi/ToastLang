#ifndef LEXER_H
#define LEXER_H
#include  "tokens.h"
#include <vector>
#include <iostream>
#include <regex>
#include <string>
#include <sstream>
#include <map>
#include <stdexcept> 
#include <fstream>

struct LexedToken{
    Token token;
    std::string value;
    const char* getOutputString(){
        std::string out = "Token: ";
        out += getTokenString(token);
        out += " Value: " + value;
        return out.c_str();
    }
};

class Lexer{
    public:
    std::map<std::string, Token> stringToTokenMap;
    int lineNumber = 0;
    std::string curLine; 
    std::string fileName;
    std::ifstream fileStream;
    bool isLexing = true;
    
    Lexer();
    void getToken(std::string tokenName, std::vector<LexedToken>& lexedTokens);
    std::vector<LexedToken> lex();
    void lexLine(std::vector<LexedToken>& lexedTokens);
    bool finishedLex();
    // ~Lexer();
};

std::ostream& operator<<(std::ostream& out, const LexedToken value);
#endif