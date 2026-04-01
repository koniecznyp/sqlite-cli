use std::iter::Peekable;
use std::vec::IntoIter;

use anyhow::{Context, Ok};
use tokenizer::Token;

use crate::sql::tokenizer;

pub enum Statement {
    Select(SelectStatement),
}

pub struct SelectStatement {
    pub from: String,
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

    fn parse_select(&mut self) -> anyhow::Result<SelectStatement> {
        self.expect(Token::Select)?;
        self.expect(Token::Star)?;
        self.expect(Token::From)?;

        if let Some(Token::Identifier(name)) = self.tokens.next() {
            Ok(SelectStatement { from: name })
        } else {
            anyhow::bail!("expected table name")
        }
    }

    fn expect(&mut self, expected: Token) -> anyhow::Result<()> {
        let next_token = self.tokens.next().context("unexpected end of input")?;

        if next_token != expected {
            anyhow::bail!("Expected token {:?}", expected);
        }
        Ok(())
    }
}
