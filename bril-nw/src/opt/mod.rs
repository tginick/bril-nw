use crate::basicblock::FunctionBlocks;

mod global;

pub trait GlobalOptimizationPass {
    fn run(function: &mut FunctionBlocks);
}