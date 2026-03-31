#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Select,
    Star,
    From,
    Table(String),
}

pub fn tokenize(query: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for keyword in query.split_whitespace() {
        match keyword.to_lowercase().as_str() {
            "select" => tokens.push(Token::Select),
            "*" => tokens.push(Token::Star),
            "from" => tokens.push(Token::From),
            _ => tokens.push(Token::Table(keyword.to_lowercase())),
        }
    }
    tokens
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
                Token::Table("cars".to_string()),
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
                Token::Table("foo".to_string())
            ]
        );
    }

    #[test]
    fn test_whitespaces() {
        let input = "SELECT   *     from";
        let result = tokenize(input);

        assert_eq!(result, vec![Token::Select, Token::Star, Token::From]);
    }
}
