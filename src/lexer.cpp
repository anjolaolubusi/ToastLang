#include "lexer.h"
//LexedToken stuff

/*
How the Lexer works:
1. Get each indiviual lexer string ("def", "foo", ":")
    a. Start if the first non space character. Peek to the next char. If the next char is of the same time as the fist. Then add it
2. Figure out the right token
    a. Using regex and the stringToTokenMap object
3. Create lexedtoken object for each token
*/

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
    LexedToken lt;
        int i = 0;
        while(i < curLine.length()){            
            while(isspace(curLine[i]) && i < curLine.length()){
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
                getToken(word, lexedTokens);
                word = "";
                continue;
            }
            while(!isalnum(curLine[i]) && !isspace(curLine[i]) && std::string("-+/*->").find(word) != std::string::npos && i < curLine.length()){
                word += curLine[i];
                i++;
            }
            std::cout << "Word: " << word << std::endl;
            getToken(word, lexedTokens);
            word = "";
            continue;
        }

}

void Lexer::getToken(std::string tokenName, std::vector<LexedToken>& lexedTokens){
    LexedToken lt;
    lt.value = tokenName;
    std::smatch regexMatch;
    if(std::regex_match (tokenName, std::regex("([A-Za-z])+([A-Za-z0-9]+)?"))){
        if(stringToTokenMap.count(tokenName) == 1){
            lt.token = stringToTokenMap[lt.value];
        }else{
            lt.token = tok_ident;
        }
        lexedTokens.push_back(lt);
        return;
    }

    if(std::regex_match (tokenName, std::regex("(-)?[0-9]*(\.[0-9]+)?"))){
        lt.token = tok_number;
        lexedTokens.push_back(lt);
        return;
    }

    while(std::regex_search(tokenName, regexMatch, std::regex("(->)|\\W"))){
        lt.value = regexMatch.str(0);
        if(stringToTokenMap.count(lt.value) == 1){
            lt.token = stringToTokenMap[lt.value];
        }else{
            lt.token = tok_customBinOP;
        }
        lexedTokens.push_back(lt);
        tokenName = regexMatch.suffix().str();
    }

}