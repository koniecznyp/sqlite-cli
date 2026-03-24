use anyhow::{Ok, bail};

use crate::{
    database::Database,
    query_plan::{ QueryPlan, PlanNode, SeqScanNode },
    parser::{ SelectStatement, Statement }
};

pub struct Planner<'a> {
    database: &'a Database
}

impl<'a> Planner<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub fn compile(&self, statement: &Statement) -> anyhow::Result<QueryPlan> {
        match statement {
            Statement::Select(select_statement) => self.compile_select(select_statement),
            _ => bail!("unsupported statement")
        }
    }

    fn compile_select(&self, select_statement: &SelectStatement) -> anyhow::Result<QueryPlan> {
        let table = self.database.tables
            .iter()
            .find(|t| t.name == select_statement.from)
            .ok_or_else(|| anyhow::anyhow!("table not exists"))?;

        Ok(QueryPlan::new(PlanNode::SeqScan(SeqScanNode {
            scanner: self.database.get_scanner(),
            rootpage: table.rootpage
        })))
    }
}