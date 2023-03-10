#ifndef TOKEN_H
#define TOKEN_H

typedef enum{
    tok_def,
    tok_extern,
    tok_if,
    tok_then,
    tok_else,
    tok_endif,
    tok_for,
    tok_binary,
    tok_unary,
    tok_openingPara,
    tok_closingPara,
    tok_comma,
    tok_forLoopTo,
    tok_forLoopToInclusive,
    tok_ident,
    tok_number,
    tok_multiply,
    tok_plus,
    tok_minus,
    tok_equals,
    tok_lessThan,
    tok_greaterThan,
    tok_customBinOP,
    tok_whitespace,
    tok_comment,
    tok_multlineCommentBegin,
    tok_multlineCommentEnd,
    tok_funcBegin,
    tok_funcEnd,
    tok_error,
    tok_varDeclare,
} Token;

const char* getTokenString(Token token);
//std::ostream& operator<<(std::ostream& out, const Token value);

#endif
