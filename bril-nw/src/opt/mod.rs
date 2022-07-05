use crate::basicblock::{BasicBlock, FunctionBlocks};

pub mod global;
pub mod local;

pub trait GlobalOptimizationPass {
    fn run(&mut self, function: &mut FunctionBlocks);
}

pub trait LocalOptimizationPass {
    fn run(&mut self, block: &mut BasicBlock);
}
