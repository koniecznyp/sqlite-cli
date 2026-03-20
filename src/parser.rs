use crate::tokenizer;

pub struct Statement { }

pub fn parse_sql(query: &str) -> anyhow::Result<Statement> {
    let tokens = tokenizer::tokenize(query);

    for token in tokens {
        println!("{:?}", token);
    }
    
    Ok(Statement {})
}