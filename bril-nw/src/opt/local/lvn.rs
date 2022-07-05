use crate::{basicblock::BasicBlock, opt::LocalOptimizationPass};

pub struct LocalValueNumbering();

struct LVNCanonicalExpression {
    op: String,
    args: Vec<usize>,
}

struct LVNTableEntry {
    canonical_expression: LVNCanonicalExpression,
    canonical_name: String,
}

impl LocalOptimizationPass for LocalValueNumbering {
    fn run(block: &mut BasicBlock) {}
}

fn canonicalize_instruction() {}
