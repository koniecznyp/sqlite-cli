#[derive(Debug, Eq, PartialEq)]
pub enum Token
{
    Select,
    Star,
    From,
    Table(String)
}

pub fn tokenize(query: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for word in query.split_whitespace() {
        match word.to_lowercase().as_str() {
            "select" => tokens.push(Token::Select),
            "*" => tokens.push(Token::Star),
            "from" => tokens.push(Token::From),
            _ => tokens.push(Token::Table(word.to_string()))
        }
    }
    tokens
}