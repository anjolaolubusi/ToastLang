#include "lexer.h"
//LexedToken stuff


std::ostream& operator<<(std::ostream& out, const LexedToken value){
    return out << "Token: " << value.token << " Value: " << value.value;
}

Lexer::Lexer(){
stringToTokenMap["def"] = tok_def;
stringToTokenMap["extern"] = tok_extern;
stringToTokenMap["if"] = tok_if;
stringToTokenMap["then"] = tok_then;
stringToTokenMap["else"] = tok_else;
stringToTokenMap["endif"] = tok_endif;
stringToTokenMap["for"] = tok_for;
stringToTokenMap["binary"] = tok_binary;
stringToTokenMap["unary"] = tok_unary;
stringToTokenMap["("] = tok_openingPara;
stringToTokenMap[")"] = tok_closingPara;
stringToTokenMap["+"] = tok_plus;
stringToTokenMap["-"] = tok_minus;
stringToTokenMap["="] = tok_equals;
stringToTokenMap["<"] = tok_lessThan;
stringToTokenMap[">"] = tok_greaterThan;
stringToTokenMap["//"] = tok_comment;
stringToTokenMap["/*"] = tok_multlineCommentBegin;
stringToTokenMap["*/"] = tok_multlineCommentEnd;
stringToTokenMap[":"] = tok_funcBegin;
stringToTokenMap["end"] = tok_funcEnd;
stringToTokenMap["let"] = tok_varDeclare;
stringToTokenMap[","] = tok_comma;
stringToTokenMap["->"] = tok_forLoopTo;
stringToTokenMap["->*"] = tok_forLoopToInclusive;
stringToTokenMap["*"] = tok_multiply;
}

// Lexer stuff
void Lexer::lexLine(const std::string line){
    std::string word = "";
    std::string nonAlphaNum = "";
    lineNumber++;
    int i = 0;
    while(i < line.length()){
        while(isspace(line[i])){
          word = "";
          i++;
        }
        while(isalnum(line[i]) && i < line.length()){
            word += line[i];
            i++;
        };
        while( (isdigit(line[i]) || line[i] == '.') && i < line.length()){
            word += line[i];
            i++;
        };
        if(word.length() > 0){
            std::cout << "Word: " << word << std::endl;
            getTokenFromString(word);
            word = "";
            continue;
        }
        while(!isalnum(line[i]) && !isspace(line[i]) && std::string("-+/*->->*").find(word) != std::string::npos && i < line.length()){
            word += line[i];
            i++;
        }
        std::cout << "Word: " << word << std::endl;
        getTokenFromString(word);
        word = "";
        continue;
    }
}

LexedToken Lexer::getTokenFromString(std::string tokenName){
    LexedToken lt;
    lt.value = tokenName;
    try{
        lt.token = stringToTokenMap.at(tokenName);
        std::cout << lt << std::endl;
        lexedTokens.push_back(lt);
        return lt;
    }catch(const std::out_of_range& oor){
        if (std::regex_match (tokenName, std::regex("([A-Za-z])+([A-Za-z0-9]+)?") )){
            lt.token = Token::tok_ident;
            std::cout << lt << std::endl;
            lexedTokens.push_back(lt);
            return lt;
        }
        if (std::regex_match (tokenName, std::regex("(-)?[0-9]*(\.[0-9]+)?")) ){
            lt.token = Token::tok_number;
            std::cout << lt << std::endl;
            lexedTokens.push_back(lt);
            return lt;
        }
        bool isPunt = true;
        for(int i = 0; i < tokenName.length(); i++){
            if(!ispunct(tokenName[i])){
                isPunt = false;
                std::cout << "Error lexing: '" << tokenName << "' at line " << lineNumber << std::endl;
            }
        }
        if(isPunt){
            lt.token = Token::tok_customBinOP;
            std::cout << lt << std::endl;
            lexedTokens.push_back(lt);
            return lt;
        }else{
            lt.token = tok_error;
            std::cout << "Error lexing: '" << tokenName << "' at line " << lineNumber << std::endl;
            return lt;
        }
        
    }
}