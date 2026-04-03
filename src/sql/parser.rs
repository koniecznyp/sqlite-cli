use std::iter::Peekable;
use std::vec::IntoIter;

use anyhow::{Context, Ok};
use tokenizer::Token;

use crate::sql::tokenizer;

pub enum Statement {
    Select(SelectStatement),
    CreateTable(CreateTableStatement),
}

pub struct SelectStatement {
    pub from: String,
}

pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<ColumnDefinition>,
}

pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String, // todo text, integer etc
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

        Ok(SelectStatement { from: table_name })
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

    fn expect_type(&mut self) -> anyhow::Result<String> {
        let next_token = self.tokens.next().context("unexpected end of input")?;

        match next_token {
            Token::Type(name) => Ok(name),
            _ => anyhow::bail!("expected type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_table() {
        let tokens = vec![
            Token::Create,
            Token::Table,
            Token::Identifier("users".to_string()),
            Token::LeftParen,
            Token::Identifier("id".to_string()),
            Token::Type("integer".to_string()),
            Token::Comma,
            Token::Identifier("name".to_string()),
            Token::Type("text".to_string()),
            Token::RightParen,
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse_create_table_statement().unwrap();

        assert_eq!(result.table_name, "users");
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].name, "id");
        assert_eq!(result.columns[0].data_type, "integer");
        assert_eq!(result.columns[1].name, "name");
        assert_eq!(result.columns[1].data_type, "text");
    }
}
