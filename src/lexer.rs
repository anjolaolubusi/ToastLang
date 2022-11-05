use logos::Logos;


#[derive(PartialEq, Clone, Copy, Debug, Logos)]
pub enum Token {
    ///Token for 'def' keyword
    #[token("def")]
    Def,
    ///Token for 'extern' keyword
    #[token("extern")]
    Extern,
    ///Token for 'if' keyword
    #[token("if")]
    If,
    ///Token for 'then' keyword
    #[token("then")]
    Then,
    ///Token for 'else' keyword
    #[token("else")]
    Else,   
    ///Token for 'endif' keyword
    #[token("endif")]
    EndIf,    
    ///Token for 'for' keyword
    #[token("for")]
    For,    
    #[token("binary")]
    Binary,
    #[token("unary")]
    Unary,    
    #[token("(")]
    OpeningParenthesis,
    #[token(")")]
    ClosingParenthesis,
    #[token(",")]
    Comma,
    #[token("->")]
    ForLoopTo,
    #[token("->*")]
    InclusiveForLoopTo,
    #[regex("([A-Za-z])+([A-Za-z0-9]+)?")]
    Ident,
    #[regex(r"(-)?[0-9]*(\.[0-9]+)?")]
    Number,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("%")]
    Modulus,
    #[token("=")]
    Equals,
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    CustomBinOp,
    #[regex(r"[ ]+|(\n|\r|\r\n)")]
    WhiteSpace,
    //#[regex(r"([\\])[\\][\w]+|([\\][\*])[\w|\n|\r|\r\n]+[\*][\\]")]
    //Comment,
    #[token(r"//")]
    Comment,
    #[token(r"/*")]
    MultilineCommentBegin,
    #[token(r"*/")]
    MultilineCommentEnd,
    #[token(":")]
    FuncBegin,
    #[token("end")]
    FuncEnd,
    #[error]
    Error,
    #[token("let")]
    VarDeclare
}

mod tests {
    use super::*;
    use std::fs;

    ///Checks if the lexer returns the correct token
    fn lex_check_word(word: &'static str, correct_token: Token){
        let mut lex = Token::lexer(word);
        assert_eq!(lex.next(), Some(correct_token));
    }

    #[test]
    fn lex_def(){
        lex_check_word("def", Token::Def);
    }

    #[test]
    fn lex_extern(){
        lex_check_word("extern", Token::Extern);
    }

    #[test]
    fn lex_open_paren(){
        lex_check_word("(", Token::OpeningParenthesis);
    }

    #[test]
    fn lex_close_paren(){
        lex_check_word(")", Token::ClosingParenthesis);
    }

    #[test]
    fn lex_equals(){
        lex_check_word("=", Token::Equals);
    }

    #[test]
    fn lex_commas(){
        lex_check_word(",", Token::Comma);
    }

    #[test]
    fn lex_ident(){
        lex_check_word("adjsjuia", Token::Ident);
        lex_check_word("var", Token::Ident);
        lex_check_word("jsidiu34", Token::Ident);
    }

    #[test]
    fn lex_numbers(){
        lex_check_word("12213", Token:: Number);
        lex_check_word("38478.23324", Token:: Number);
        lex_check_word("-7848734", Token:: Number);
        lex_check_word("-894783847.559", Token:: Number);
    }

    #[test]
    fn lex_operators(){
        lex_check_word("+", Token::Plus);
        lex_check_word("-", Token::Minus);
        lex_check_word("*", Token::Multiply);
        lex_check_word("/", Token::Divide);
        lex_check_word("%", Token:: Modulus);
    }

    #[test]
    fn lex_comment(){
        lex_check_word("//", Token::Comment);
        lex_check_word("/*", Token::MultilineCommentBegin);
        lex_check_word("*/", Token::MultilineCommentEnd);
    }

    #[test]
    fn lex_spaces(){
        let spaces = "       ";
        let mut lex = Token::lexer(&spaces);
        assert_eq!(lex.next(), Some(Token::WhiteSpace));
        assert_eq!(lex.slice(), spaces);
    }

    #[test]
    fn lex_file(){
        let contents = fs::read_to_string("exampleCode/test1.toast").expect("Expected file here");
        let lex = Token::lexer(&contents);
        let lexedFile: Vec<_> = lex.spanned().collect();
        println!("{:?}", lexedFile);
    }

    #[test]
    fn lex_varDeclare(){
        lex_check_word("let", Token::VarDeclare);
    }

}