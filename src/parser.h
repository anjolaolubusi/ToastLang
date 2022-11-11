#ifndef PARSER_H
#define PARSER_H

#include <string>
#include <map>
#include <vector>

struct ExprAST;
struct VariableExpr;
struct NumberExpr;


class Parser
{
private:
    /* data */
public:
    Parser(/* args */);
    // ~Parser();
    void parse();
    void getNextToken();
    std::map<std::string, int> BinOpMap;
    std::vector<ExprAST> parsedTokens;
};

#endif