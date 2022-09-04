use logos::Logos;

#[derive(PartialEq, Clone, Debug, Logos)]
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
    #[regex("[A-Za-z][A-Za-z0-9]+")]
    Ident,
    #[regex(r"(-)?[0-9]*(\.)?[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+")]
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
    #[regex(" +")]
    WhiteSpace,
    #[error]
    Error
}

mod tests {
    use super::*;

    fn lex_check_word(word: &String, correctToken: Token){
        let mut lex = Token::lexer(word);
        assert_eq!(lex.next(), Some(correctToken));
    }

    #[test]
    fn lex_def(){
        lex_check_word(&"def".to_string(), Token::Def);
    }

    #[test]
    fn lex_extern(){
        lex_check_word(&"extern".to_string(), Token::Extern);
    }

    #[test]
    fn lex_open_paren(){
        lex_check_word(&"(".to_string(), Token::OpeningParenthesis);
    }

    #[test]
    fn lex_close_paren(){
        lex_check_word(&")".to_string(), Token::ClosingParenthesis);
    }

    #[test]
    fn lex_commas(){
        lex_check_word(&",".to_string(), Token::Comma);
    }

    #[test]
    fn lex_ident(){
        lex_check_word(&"adjsjuia".to_string(), Token::Ident);
        lex_check_word(&"var".to_string(), Token::Ident);
        lex_check_word(&"jsidiu34".to_string(), Token::Ident);
    }

    #[test]
    fn lex_numbers(){
        lex_check_word(&"12213".to_string(), Token:: Number);
        lex_check_word(&"38478.23324".to_string(), Token:: Number);
        lex_check_word(&"-7848734".to_string(), Token:: Number);
        lex_check_word(&"-894783847.559".to_string(), Token:: Number);
        lex_check_word(&"-4e45".to_string(), Token:: Number);
    }

    #[test]
    fn lex_operators(){
        lex_check_word(&"+".to_string(), Token::Plus);
        lex_check_word(&"-".to_string(), Token::Minus);
        lex_check_word(&"*".to_string(), Token::Multiply);
        lex_check_word(&"/".to_string(), Token::Divide);
        lex_check_word(&"%".to_string(), Token:: Modulus);
    }

    #[test]
    fn lex_spaces(){
        let spaces = "       ";
        let mut lex = Token::lexer(&spaces);
        assert_eq!(lex.next(), Some(Token::WhiteSpace));
        assert_eq!(lex.slice(), spaces);
    }


}