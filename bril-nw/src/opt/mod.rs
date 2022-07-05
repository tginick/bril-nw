use crate::basicblock::{BasicBlock, FunctionBlocks};

pub mod global;
pub mod local;

pub trait GlobalOptimizationPass {
    fn run(function: &mut FunctionBlocks);
}

pub trait LocalOptimizationPass {
    fn run(block: &mut BasicBlock);
}
