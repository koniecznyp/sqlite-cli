use crate::query_plan::QueryPlan;
use crate::query_plan::PlanNode;
use crate::scanner::RecordIter;
use crate::scanner::Record;

pub struct Executor {
    rows_iter: RecordIter
}

impl Executor {
    pub fn new(query_plan: &QueryPlan) -> anyhow::Result<Self> {
        let select = &query_plan.root;
        let node = match select {
            PlanNode::SeqScan(node) => node
        };
        
        let rows_iter = node.scanner.scan(node.rootpage)?;
        Ok(Self { rows_iter })
    }

    pub fn get_next_row(&mut self) -> anyhow::Result<Option<Record>> {
        match self.rows_iter.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}