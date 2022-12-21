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
std::vector<LexedToken> Lexer::lex(){
    std::vector<LexedToken> lexedTokens;
    if(fileName.empty()){
        fprintf(stdout, "> ");
        for(curLine; std::getline(std::cin, curLine);){
            if(curLine.empty()){
                break;
            }
            lexLine(lexedTokens);
        }
    }else{
        for(curLine; std::getline(fileStream, curLine);){
            lexLine(lexedTokens);
        }
        fileStream.close();        
    }
    return lexedTokens;
}

void Lexer::lexLine(std::vector<LexedToken>& lexedTokens){
    std::string word = "";
    std::string nonAlphaNum = "";
        int i = 0;
        while(i < curLine.length()){
            while(isspace(curLine[i])){
              word = "";
              i++;
            }
            while(isalnum(curLine[i]) && i < curLine.length()){
                word += curLine[i];
                i++;
            };
            while( (isdigit(curLine[i]) || curLine[i] == '.') && i < curLine.length()){
                word += curLine[i];
                i++;
            };
            if(word.length() > 0){
                std::cout << "Word: " << word << std::endl;
                lexedTokens.push_back(getTokenFromString(word));
                word = "";
                continue;
            }
            while(!isalnum(curLine[i]) && !isspace(curLine[i]) && std::string("-+/*->->*").find(word) != std::string::npos && i < curLine.length()){
                word += curLine[i];
                i++;
            }
            std::cout << "Word: " << word << std::endl;
            lexedTokens.push_back(getTokenFromString(word));
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
        return lt;
    }catch(const std::out_of_range& oor){
        if (std::regex_match (tokenName, std::regex("([A-Za-z])+([A-Za-z0-9]+)?") )){
            lt.token = Token::tok_ident;
            std::cout << lt << std::endl;
            return lt;
        }
        if (std::regex_match (tokenName, std::regex("(-)?[0-9]*(\.[0-9]+)?")) ){
            lt.token = Token::tok_number;
            std::cout << lt << std::endl;
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
            return lt;
        }else{
            lt.token = tok_error;
            std::cout << "Error lexing: '" << tokenName << "' at line " << lineNumber << std::endl;
            return lt;
        }
        
    }
}