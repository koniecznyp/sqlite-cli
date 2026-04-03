use anyhow::Ok;

use crate::core::database::Database;
use crate::sql::parser::{SelectStatement, Statement};
use crate::sql::query_plan::{PlanNode, QueryFilter, QueryPlan, SeqScanNode};

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
            _ => anyhow::bail!("not supported statement"),
        }
    }

    fn compile_select(&self, select_statement: &SelectStatement) -> anyhow::Result<QueryPlan<'a>> {
        let table = self
            .database
            .tables
            .iter()
            .find(|t| t.name == select_statement.from)
            .ok_or_else(|| anyhow::anyhow!("table not exists"))?;

        let mut query_filter = None;
        if let Some(condition) = &select_statement.filter {
            let column_index = table
                .columns
                .iter()
                .position(|c| c.name == condition.key)
                .unwrap(); // fix?

            query_filter = Some(QueryFilter {
                column_index,
                value: condition.value.clone(),
            });
        }

        Ok(QueryPlan::new(PlanNode::SeqScan(SeqScanNode {
            scanner: self.database.get_scanner(),
            rootpage: table.rootpage,
            filter: query_filter,
        })))
    }
}
