use crate::query_plan::QueryPlan;
use crate::query_plan::PlanNode;
use crate::scanner::Record;

pub struct Executor<'a> {
    query_plan: &'a QueryPlan,
    rows_iter: Box<dyn Iterator<Item = Record> + 'a>,
}

impl<'a> Executor<'a> {
    pub fn new(query_plan: &'a QueryPlan) -> anyhow::Result<Self> {
        let select = &query_plan.root;
        let node = match select {
            PlanNode::SeqScan(node) => node
        };
        
        let rows = node.scanner.scan(node.rootpage)?;
        Ok(Self {
            query_plan,
            rows_iter: Box::new(rows.into_iter()),
        })
    }

    pub fn get_next_row(&mut self) -> anyhow::Result<Option<Record>> {
        Ok(self.rows_iter.next())
    }
}