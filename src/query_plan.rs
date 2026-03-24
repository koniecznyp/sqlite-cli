use crate::{scanner::Scanner };

#[derive(Debug)]
pub struct QueryPlan{
    root: PlanNode
}

impl QueryPlan {
    pub fn new(root: PlanNode) -> Self {
        Self { root }
    }
}
#[derive(Debug)]
pub enum PlanNode {
    SeqScan(SeqScanNode)
}

#[derive(Debug)]
pub struct SeqScanNode {
    pub scanner: Scanner,
    pub rootpage: usize
}