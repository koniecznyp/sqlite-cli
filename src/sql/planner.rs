use anyhow::Ok;

use crate::core::database::Database;
use crate::sql::parser::{SelectStatement, Statement};
use crate::sql::query_plan::{PlanNode, QueryPlan, SeqScanNode};

pub struct Planner<'a> {
    database: &'a Database,
}

impl<'a> Planner<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub fn compile(&self, statement: &Statement) -> anyhow::Result<QueryPlan<'a>> {
        match statement {
            Statement::Select(select_statement) => self.compile_select(select_statement),
        }
    }

    fn compile_select(&self, select_statement: &SelectStatement) -> anyhow::Result<QueryPlan<'a>> {
        let table = self
            .database
            .tables
            .iter()
            .find(|t| t.name == select_statement.from)
            .ok_or_else(|| anyhow::anyhow!("table not exists"))?;

        Ok(QueryPlan::new(PlanNode::SeqScan(SeqScanNode {
            scanner: self.database.get_scanner(),
            rootpage: table.rootpage,
        })))
    }
}
