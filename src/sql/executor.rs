use crate::core::scanner::{Record, RecordIter, RecordValue};
use crate::sql::parser::Operator;
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
            let record = match self.rows_iter.next() {
                Some(Ok(r)) => r,
                _ => return Ok(None),
            };

            let filter = match &self.filter {
                Some(f) => f,
                None => return Ok(Some(record)),
            };

            let field_value = record.field(filter.column_index)?;

            let is_match = match field_value {
                Some(RecordValue::Int(i)) => self.filter_integer(i, &filter.op, &filter.value)?,
                Some(RecordValue::String(ref s)) => {
                    self.filter_string(s, &filter.op, &filter.value)?
                }
                _ => false,
            };

            if is_match {
                return Ok(Some(record));
            }
        }
    }

    fn filter_integer(
        &self,
        record_val: i64,
        op: &Operator,
        filter_val: &str,
    ) -> anyhow::Result<bool> {
        let parsed_filter_val = filter_val.parse()?;
        let is_match = match op {
            Operator::Eq => record_val == parsed_filter_val,
            Operator::Neq => record_val != parsed_filter_val,
            Operator::Lt => record_val < parsed_filter_val,
            Operator::Lte => record_val <= parsed_filter_val,
            Operator::Gt => record_val > parsed_filter_val,
            Operator::Gte => record_val >= parsed_filter_val,
        };

        Ok(is_match)
    }

    fn filter_string(
        &self,
        record_val: &str,
        op: &Operator,
        filter_val: &str,
    ) -> anyhow::Result<bool> {
        let is_match = match op {
            Operator::Eq => record_val == filter_val,
            Operator::Neq => record_val != filter_val,
            _ => anyhow::bail!("string filter error - not supported"),
        };

        Ok(is_match)
    }
}
