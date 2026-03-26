use crate::{scanner::Scanner };

#[derive(Debug)]
pub struct QueryPlan<'a> {
    pub root: PlanNode<'a>
}

impl<'a> QueryPlan<'a> {
    pub fn new(root: PlanNode<'a>) -> Self {
        Self { root }
    }
}
#[derive(Debug)]
pub enum PlanNode<'a> {
    SeqScan(SeqScanNode<'a>)
}

#[derive(Debug)]
pub struct SeqScanNode<'a> {
    pub scanner: Scanner<'a>,
    pub rootpage: usize
}