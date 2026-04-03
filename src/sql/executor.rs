use anyhow::bail;

use crate::core::scanner::{Record, RecordIter, RecordValue};
use crate::sql::query_plan::{PlanNode, QueryFilter, QueryPlan};

pub struct Executor<'a> {
    rows_iter: RecordIter<'a>,
    filter: Option<QueryFilter>,
}
impl<'a> Executor<'a> {
    pub fn new(query_plan: &'a QueryPlan<'a>) -> anyhow::Result<Self> {
        let select = &query_plan.root;
        let node = match select {
            PlanNode::SeqScan(node) => node,
        };

        let rows_iter = node.scanner.scan(node.rootpage)?;
        Ok(Self {
            rows_iter,
            filter: node.filter.clone(),
        })
    }

    pub fn get_next_row(&mut self) -> anyhow::Result<Option<Record>> {
        loop {
            match self.rows_iter.next() {
                Some(Ok(record)) => match &self.filter {
                    None => return Ok(Some(record)),
                    Some(filter) => {
                        if let Some(value) = record.field(filter.column_index)? {
                            let num_value = match value {
                                RecordValue::Int(i) => i,
                                _ => bail!("not supported yet"),
                            };

                            if num_value == filter.value.parse::<i64>().unwrap() {
                                return Ok(Some(record));
                            }
                        }
                    }
                },
                Some(Err(e)) => return Err(e),
                None => return Ok(None),
            }
        }
    }
}
