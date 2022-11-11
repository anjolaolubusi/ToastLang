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
    std::vector<LexedToken> lexedTokens;
    std::map<std::string, Token> stringToTokenMap;
    int lineNumber = 0;
    Lexer();
    LexedToken getTokenFromString(std::string tokenName);
    void lexLine(const std::string line);
    // ~Lexer();
};

std::ostream& operator<<(std::ostream& out, const LexedToken value);
#endif