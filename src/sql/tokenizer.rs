use std::iter::Peekable;
use std::str::Chars;

use crate::sql::parser::{ColumnType, Operator};

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Select,
    Star,
    From,
    Where,
    Literal(String),
    Identifier(String),
    Create,
    Table,
    LeftParen,
    RightParen,
    Comma,
    Type(ColumnType),
    Operator(Operator),
}

pub fn tokenize(query: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = query.chars().peekable();
    let mut expect_literal = false;

    while let Some(char) = chars.next() {
        match char {
            char if char.is_whitespace() => continue,
            '*' => tokens.push(Token::Star),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            ',' => tokens.push(Token::Comma),
            '\'' | '"' => {
                let mut literal = String::new();
                for c in chars.by_ref() {
                    if c == char {
                        break;
                    }
                    literal.push(c);
                }
                tokens.push(Token::Literal(literal));
                expect_literal = false;
            }
            char if char.is_alphanumeric() => {
                let keyword = parse_keyword_token(char, &mut chars);

                if expect_literal {
                    tokens.push(Token::Literal(keyword));
                    expect_literal = false;
                } else {
                    let keyword_lower = keyword.to_lowercase();
                    match keyword_lower.as_str() {
                        "select" => tokens.push(Token::Select),
                        "from" => tokens.push(Token::From),
                        "where" => tokens.push(Token::Where),
                        "create" => tokens.push(Token::Create),
                        "table" => tokens.push(Token::Table),
                        k => {
                            if let Some(col_type) = as_column_type(k) {
                                tokens.push(Token::Type(col_type));
                            } else {
                                tokens.push(Token::Identifier(keyword_lower));
                            }
                        }
                    }
                }
            }
            char => {
                let operator = parse_operator(char, &mut chars);

                match operator.as_str() {
                    "=" => tokens.push(Token::Operator(Operator::Eq)),
                    "<" => tokens.push(Token::Operator(Operator::Lt)),
                    ">" => tokens.push(Token::Operator(Operator::Gt)),
                    "<=" => tokens.push(Token::Operator(Operator::Lte)),
                    ">=" => tokens.push(Token::Operator(Operator::Gte)),
                    "!=" => tokens.push(Token::Operator(Operator::Neq)),
                    _ => {}
                }
                expect_literal = true;
            }
        }
    }
    tokens
}

fn parse_keyword_token(char: char, chars: &mut Peekable<Chars>) -> String {
    let mut keyword = char.to_string();
    while let Some(w) = chars.next_if(|f| f.is_alphanumeric()) {
        keyword.push(w);
    }
    keyword
}

fn parse_operator(char: char, chars: &mut Peekable<Chars>) -> String {
    let mut operator = char.to_string();
    if let Some(&next) = chars.peek() {
        match (char, next) {
            ('<', '=') | ('>', '=') | ('!', '=') => {
                operator.push(chars.next().unwrap());
            }
            _ => {}
        }
    }
    operator
}

fn as_column_type(keyword: &str) -> Option<ColumnType> {
    match keyword {
        "integer" => Some(ColumnType::Integer),
        "text" => Some(ColumnType::Text),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test]
    fn test_query_basic_select_all() {
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
    fn test_query_case_insensitivity() {
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
    fn test_query_whitespaces() {
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
            vec![
                Token::Star,
                Token::Operator(Operator::Eq),
                Token::Star,
                Token::Operator(Operator::Eq),
                Token::Operator(Operator::Eq)
            ]
        );
    }

    #[test_case("= 5", Operator::Eq)]
    #[test_case("!=5", Operator::Neq)]
    #[test_case("> 5", Operator::Gt)]
    #[test_case("<   5", Operator::Lt)]
    #[test_case(">= 5", Operator::Gte)]
    #[test_case("<= 5", Operator::Lte)]
    fn test_where_number(input: &str, op: Operator) {
        let result = tokenize(input);
        assert_eq!(
            result,
            vec![Token::Operator(op), Token::Literal(String::from("5"))]
        );
    }

    #[test_case("= 'aAa_1'", Operator::Eq)]
    #[test_case("!=   'aAa_1'", Operator::Neq)]
    fn test_where_text(input: &str, op: Operator) {
        let result = tokenize(input);
        assert_eq!(
            result,
            vec![Token::Operator(op), Token::Literal(String::from("aAa_1"))]
        );
    }

    #[test]
    fn test_sql_where_condition() {
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
                Token::Operator(Operator::Eq),
                Token::Literal("5".to_string()),
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
                Token::Type(ColumnType::Integer),
                Token::Comma,
                Token::Identifier("name".to_string()),
                Token::Type(ColumnType::Text),
                Token::RightParen
            ]
        );
    }
}
