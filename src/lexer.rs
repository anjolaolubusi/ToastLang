use logos::Logos;


#[derive(PartialEq, Clone, Copy, Debug, Logos)]
pub enum Token {
    ///Token for 'def' keyword
    #[token("def")]
    Def,
    // ///Token for 'extern' keyword
    // #[token("extern")]
    // Extern,
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
    #[token("(")]
    OpeningParenthesis,
    #[token(")")]
    ClosingParenthesis,
    #[token(",")]
    Comma,
    #[token("->")]
    PointTo,
    #[regex("([A-Za-z])+([A-Za-z0-9]+)?")]
    Ident,
    #[regex(r"([0-9]*)([.][0-9]+)?")]
    Number,
    #[regex("\"([A-Za-z0-9\\s])*\"")]
    String,
    #[regex(r"'([A-Za-z0-9])?'")]
    Char,
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
    #[token(";")]
    SemiColon,
    #[error]
    Error,
    #[token("let")]
    VarDeclare,
    #[token("[")]
    OpenSquareBracket,
    #[token("]")]
    CloseSquareBracket,
    // #[regex(r"[a-zA-Z]+\d*\[")]
    // ArrayElementAcces
}

mod tests {
    use super::*;

    

    #[test]
    fn lex_def(){
        {
            let correct_token = super::Token::Def;
            let mut lex = Token::lexer("def");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

    // #[test]
    // fn lex_extern(){
    //     {
    //         let correct_token = Token::Extern;
    //         let mut lex = Token::lexer("extern");
    //         assert_eq!(lex.next(), Some(correct_token));
    //     };
    // }

    #[test]
    fn lex_open_paren(){
        {
            let correct_token = Token::OpeningParenthesis;
            let mut lex = Token::lexer("(");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

    #[test]
    fn lex_close_paren(){
        {
            let correct_token = Token::ClosingParenthesis;
            let mut lex = Token::lexer(")");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

    #[test]
    fn lex_equals(){
        {
            let correct_token = Token::Equals;
            let mut lex = Token::lexer("=");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

    #[test]
    fn lex_commas(){
        {
            let correct_token = Token::Comma;
            let mut lex = Token::lexer(",");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

    #[test]
    fn lex_ident(){
        {
            let correct_token = Token::Ident;
            let mut lex = Token::lexer("adjsjuia");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token::Ident;
            let mut lex = Token::lexer("var");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token::Ident;
            let mut lex = Token::lexer("jsidiu34");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

    // #[test]
    // fn lex_numbers(){
    //     {
    //         let correct_token = Token:: Int;
    //         let mut lex = Token::lexer("12213");
    //         assert_eq!(lex.next(), Some(correct_token));
    //     };
    //     {
    //         let correct_token = Token:: Float;
    //         let mut lex = Token::lexer("38478.23324");
    //         assert_eq!(lex.next(), Some(correct_token));
    //     };
    //     {
    //         let correct_token = Token:: Int;
    //         let mut lex = Token::lexer("-7848734");
    //         assert_eq!(lex.next(), Some(correct_token));
    //     };
    //     {
    //         let correct_token = Token:: Float;
    //         let mut lex = Token::lexer("-894783847.559");
    //         assert_eq!(lex.next(), Some(correct_token));
    //     };
    // }

    #[test]
    fn lex_operators(){
        {
            let correct_token = Token::Plus;
            let mut lex = Token::lexer("+");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token::Minus;
            let mut lex = Token::lexer("-");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token::Multiply;
            let mut lex = Token::lexer("*");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token::Divide;
            let mut lex = Token::lexer("/");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token:: Modulus;
            let mut lex = Token::lexer("%");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

    #[test]
    fn lex_comment(){
        {
            let correct_token = Token::Comment;
            let mut lex = Token::lexer("//");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token::MultilineCommentBegin;
            let mut lex = Token::lexer("/*");
            assert_eq!(lex.next(), Some(correct_token));
        };
        {
            let correct_token = Token::MultilineCommentEnd;
            let mut lex = Token::lexer("*/");
            assert_eq!(lex.next(), Some(correct_token));
        };
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
        use std::fs;
        let contents = fs::read_to_string("exampleCode/string-test.toast").expect("Expected file here");
        let lex = Token::lexer(&contents);
        let lexed_file: Vec<_> = lex.spanned().collect();
        println!("{:?}", lexed_file);
    }

    #[test]
    fn lex_var_declare(){
        {
            let correct_token = Token::VarDeclare;
            let mut lex = Token::lexer("let");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }


    #[test]
    fn lex_string(){
        {
            let correct_token = Token::String;
            let mut lex = Token::lexer("\"Hello World\"");
            assert_eq!(lex.next(), Some(correct_token));
        };
    }

}