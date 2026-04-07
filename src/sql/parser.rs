use std::iter::Peekable;
use std::vec::IntoIter;

use anyhow::{Context, Ok};
use tokenizer::Token;

use crate::sql::tokenizer;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    CreateTable(CreateTableStatement),
}

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    pub from: String,
    pub filter: Option<Condition>,
}

#[derive(Debug, PartialEq)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: ColumnType,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ColumnType {
    Integer,
    Text,
    Float,
}

#[derive(Debug, PartialEq)]
pub struct Condition {
    pub key: String,
    pub value: String,
    pub op: Operator,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Operator {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}

pub fn parse_sql(query: &str) -> anyhow::Result<Statement> {
    let tokens = tokenizer::tokenize(query);
    let mut parser = Parser::new(tokens);
    let statement = parser.parse_statement()?;
    Ok(statement)
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse_statement(&mut self) -> anyhow::Result<Statement> {
        let first_token = self.tokens.peek().context("query is empty")?;

        match first_token {
            Token::Select => self.parse_select().map(Statement::Select),
            _ => anyhow::bail!("unsupported statement"),
        }
    }

    pub fn parse_create_table_statement(&mut self) -> anyhow::Result<CreateTableStatement> {
        self.expect(Token::Create)?;
        self.expect(Token::Table)?;

        let table_name = self.expect_identifier()?;

        self.expect(Token::LeftParen)?;

        let mut columns = Vec::new();
        loop {
            let col_name = self.expect_identifier()?;
            let col_type = self.expect_type()?;

            columns.push(ColumnDefinition {
                name: col_name,
                data_type: col_type,
            });

            match self.tokens.next() {
                Some(Token::Comma) => continue,
                Some(Token::RightParen) => break,
                Some(_) => anyhow::bail!("expected ',' or ')'"),
                None => anyhow::bail!("parsing create statement failed"),
            }
        }

        Ok(CreateTableStatement {
            table_name,
            columns,
        })
    }

    fn parse_select(&mut self) -> anyhow::Result<SelectStatement> {
        self.expect(Token::Select)?;
        self.expect(Token::Star)?;
        self.expect(Token::From)?;
        let table_name = self.expect_identifier()?;

        let mut simple_condition = None;
        if let Some(Token::Where) = self.tokens.peek() {
            self.expect(Token::Where)?;
            let key = self.expect_identifier()?;
            let op = self.expect_operator()?;
            let value = self.expect_value()?;

            simple_condition = Some(Condition { key, value, op });
        }

        Ok(SelectStatement {
            from: table_name,
            filter: simple_condition,
        })
    }

    fn expect(&mut self, expected: Token) -> anyhow::Result<()> {
        let next_token = self.tokens.next().context("unexpected end of input")?;

        if next_token != expected {
            anyhow::bail!("Expected token {:?}", expected);
        }
        Ok(())
    }

    fn expect_identifier(&mut self) -> anyhow::Result<String> {
        let next_token = self.tokens.next().context("unexpected end of input")?;

        match next_token {
            Token::Identifier(name) => Ok(name),
            _ => anyhow::bail!("expected identifier"),
        }
    }

    fn expect_operator(&mut self) -> anyhow::Result<Operator> {
        let next_token = self.tokens.next().context("unexpected end of input")?;

        match next_token {
            Token::Operator(op) => Ok(op),
            _ => anyhow::bail!("expected operator"),
        }
    }

    fn expect_type(&mut self) -> anyhow::Result<ColumnType> {
        let next_token = self.tokens.next().context("unexpected end of input")?;

        match next_token {
            Token::Type(ColumnType::Integer) => Ok(ColumnType::Integer),
            Token::Type(ColumnType::Text) => Ok(ColumnType::Text),
            _ => anyhow::bail!("expected type"),
        }
    }

    fn expect_value(&mut self) -> anyhow::Result<String> {
        let next_token = self.tokens.next().context("unexpected end of input")?;

        match next_token {
            Token::Literal(value) => Ok(value),
            _ => anyhow::bail!("expected value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_basic() {
        let tokens = vec![
            Token::Select,
            Token::Star,
            Token::From,
            Token::Identifier("users".to_string()),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse_statement().unwrap();

        let expected = Statement::Select(SelectStatement {
            from: String::from("users"),
            filter: None,
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_select_with_where() {
        let tokens = vec![
            Token::Select,
            Token::Star,
            Token::From,
            Token::Identifier("users".to_string()),
            Token::Where,
            Token::Identifier("id".to_string()),
            Token::Operator(Operator::Eq),
            Token::Literal(String::from("20")),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse_statement().unwrap();

        let expected = Statement::Select(SelectStatement {
            from: String::from("users"),
            filter: Some(Condition {
                key: String::from("id"),
                value: String::from("20"),
                op: Operator::Eq,
            }),
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_create_table() {
        let tokens = vec![
            Token::Create,
            Token::Table,
            Token::Identifier("users".to_string()),
            Token::LeftParen,
            Token::Identifier("id".to_string()),
            Token::Type(ColumnType::Integer),
            Token::Comma,
            Token::Identifier("name".to_string()),
            Token::Type(ColumnType::Text),
            Token::RightParen,
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse_create_table_statement().unwrap();

        assert_eq!(result.table_name, "users");
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].name, "id");
        assert_eq!(result.columns[0].data_type, ColumnType::Integer);
        assert_eq!(result.columns[1].name, "name");
        assert_eq!(result.columns[1].data_type, ColumnType::Text);
    }
}
