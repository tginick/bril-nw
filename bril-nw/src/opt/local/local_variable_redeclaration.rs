use std::collections::HashSet;

use crate::{basicblock::BasicBlock, opt::LocalOptimizationPass};

pub struct LocalVariableRedeclaration();

/*
    The intuition behind this optimization is the following:
    - If there are two independent assignments to a variable, then the first is not needed
        v = 1; <- can be eliminated
        v = 2;

    - This is true even if there are instructions in between these
        v = 1; <- can be eliminated
        x = 2;
        y = 2 * x;
        v = 2;

    - However, dependencies will prevent elimination of previous assignments
        v = 1; <- CANNOT be eliminated
        v = v + 1;

    - A subsequent assignment can eliminate previously dependant assignments
        v = 1; <- can be eliminated
        v = v + 1; <- can ALSO be eliminated
        v = 2;
*/
impl LocalOptimizationPass for LocalVariableRedeclaration {
    fn run(block: &mut BasicBlock) {
        // can be written more optimally, but running to convergence WILL work
        loop {
            let should_continue = delete_unused_assignments(block);
            if !should_continue {
                break;
            }
        }
    }
}

fn delete_unused_assignments(block: &mut BasicBlock) -> bool {
    let mut already_assigned = HashSet::<String>::new();
    let mut reassignments_removed = 0;

    for instr in &block.instrs {}

    reassignments_removed > 0
}
