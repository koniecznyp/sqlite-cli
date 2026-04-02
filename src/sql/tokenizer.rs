use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Select,
    Star,
    From,
    Where,
    Eq,
    Number(String),
    Identifier(String),
    Create,
    Table,
    LeftParen,
    RightParen,
    Comma,
    Type(String),
}

pub fn tokenize(query: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let query_lowered = query.to_lowercase();
    let mut chars = query_lowered.chars().peekable();

    while let Some(char) = chars.next() {
        match char {
            char if char.is_whitespace() => continue,
            '*' => tokens.push(Token::Star),
            '=' => tokens.push(Token::Eq),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            ',' => tokens.push(Token::Comma),
            char if char.is_ascii_digit() => {
                tokens.push(parse_number_token(char, &mut chars));
            }
            char if char.is_alphabetic() => {
                let keyword = parse_keyword_token(char, &mut chars);

                match keyword.as_str() {
                    "select" => tokens.push(Token::Select),
                    "from" => tokens.push(Token::From),
                    "where" => tokens.push(Token::Where),
                    "create" => tokens.push(Token::Create),
                    "table" => tokens.push(Token::Table),
                    k if is_type_name(k) => tokens.push(Token::Type(keyword)),
                    _ => tokens.push(Token::Identifier(keyword)),
                }
            }
            _ => {}
        }
    }
    tokens
}

fn parse_number_token(first_char: char, chars: &mut Peekable<Chars>) -> Token {
    let mut number = first_char.to_string();
    while let Some(n) = chars.next_if(|f| f.is_ascii_digit()) {
        number.push(n);
    }
    Token::Number(number)
}

fn parse_keyword_token(char: char, chars: &mut Peekable<Chars>) -> String {
    let mut keyword = char.to_string();
    while let Some(w) = chars.next_if(|f| f.is_alphanumeric()) {
        keyword.push(w);
    }
    keyword
}

fn is_type_name(keyword: &str) -> bool {
    match keyword {
        "integer" | "text" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_select_all() {
        let input = "SELECT * FROM cars";

        let result = tokenize(input);

        assert_eq!(
            result,
            vec![
                Token::Select,
                Token::Star,
                Token::From,
                Token::Identifier("cars".to_string()),
            ]
        );
    }

    #[test]
    fn test_case_insensitivity() {
        let input = "seLeCt * fRoM foO";

        let result = tokenize(input);

        assert_eq!(
            result,
            vec![
                Token::Select,
                Token::Star,
                Token::From,
                Token::Identifier("foo".to_string())
            ]
        );
    }

    #[test]
    fn test_whitespaces() {
        let input = "SELECT   *     from";
        let result = tokenize(input);

        assert_eq!(result, vec![Token::Select, Token::Star, Token::From]);
    }

    #[test]
    fn test_single_tokens() {
        let input = "*=*==";
        let result = tokenize(input);

        assert_eq!(
            result,
            vec![Token::Star, Token::Eq, Token::Star, Token::Eq, Token::Eq]
        );
    }

    #[test]
    fn test_where_with_number() {
        let input = "select * from cars where id = 5";
        let result = tokenize(input);

        assert_eq!(
            result,
            vec![
                Token::Select,
                Token::Star,
                Token::From,
                Token::Identifier("cars".to_string()),
                Token::Where,
                Token::Identifier("id".to_string()),
                Token::Eq,
                Token::Number("5".to_string()),
            ]
        );
    }

    #[test]
    fn test_create_table() {
        let input = "CREATE TABLE cars(id integer, name text);";
        let result = tokenize(input);

        assert_eq!(
            result,
            vec![
                Token::Create,
                Token::Table,
                Token::Identifier("cars".to_string()),
                Token::LeftParen,
                Token::Identifier("id".to_string()),
                Token::Type("integer".to_string()),
                Token::Comma,
                Token::Identifier("name".to_string()),
                Token::Type("text".to_string()),
                Token::RightParen
            ]
        );
    }
}
