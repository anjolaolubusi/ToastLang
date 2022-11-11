#include "tokens.h"

const char* getTokenString(Token token){
    const char* s = 0;
#define PROCESS_VAL(p) case(p): s = #p; break;
    switch(token){
    PROCESS_VAL(tok_def);
    PROCESS_VAL(tok_extern);
    PROCESS_VAL(tok_if);
    PROCESS_VAL(tok_then);
    PROCESS_VAL(tok_else);
    PROCESS_VAL(tok_endif);
    PROCESS_VAL(tok_for);
    PROCESS_VAL(tok_binary);
    PROCESS_VAL(tok_unary);
    PROCESS_VAL(tok_openingPara);
    PROCESS_VAL(tok_closingPara);
    PROCESS_VAL(tok_comma);
    PROCESS_VAL(tok_forLoopTo);
    PROCESS_VAL(tok_forLoopToInclusive);
    PROCESS_VAL(tok_ident);
    PROCESS_VAL(tok_number);
    PROCESS_VAL(tok_plus);
    PROCESS_VAL(tok_minus);
    PROCESS_VAL(tok_equals);
    PROCESS_VAL(tok_lessThan);
    PROCESS_VAL(tok_greaterThan);
    PROCESS_VAL(tok_customBinOP);
    PROCESS_VAL(tok_whitespace);
    PROCESS_VAL(tok_comment);
    PROCESS_VAL(tok_multlineCommentBegin);
    PROCESS_VAL(tok_multlineCommentEnd);
    PROCESS_VAL(tok_funcBegin);
    PROCESS_VAL(tok_funcEnd);
    PROCESS_VAL(tok_error);
    PROCESS_VAL(tok_varDeclare);
    }
#undef PROCESS_VAL
return s;
}

std::ostream& operator<<(std::ostream& out, const Token value){

    return out << getTokenString(value);
}