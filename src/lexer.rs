use logos::Logos;

#[derive(PartialEq, Clone, Copy, Debug, Logos)]
pub enum Token {
    #[token("def")]
    Def,
    #[token("extern")]
    Extern,
    #[token("(")]
    OpeningParenthesis,
    #[token(")")]
    ClosingParenthesis,
    #[token(",")]
    Comma,
    #[regex("[A-Za-z0-9]+")]
    Ident,
    #[regex("(-)?[0-9]*(.[0-9]+)?")]
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
    #[regex(" +")]
    WhiteSpace,
    #[token(":")]
    FuncBegin,
    #[token("end")]
    FuncEnd,
    #[error]
    Error
}

mod tests {
    use super::*;

    fn lex_check_word(word: &'static str, correctToken: Token){
        let mut lex = Token::lexer(word);
        assert_eq!(lex.next(), Some(correctToken));
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
        lex_check_word("-4e45", Token:: Number);
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
    fn lex_spaces(){
        let spaces = "       ";
        let mut lex = Token::lexer(&spaces);
        assert_eq!(lex.next(), Some(Token::WhiteSpace));
        assert_eq!(lex.slice(), spaces);
    }


}