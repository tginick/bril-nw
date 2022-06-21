use std::{collections::HashSet, mem};

use crate::{basicblock::FunctionBlocks, opt::GlobalOptimizationPass};

pub struct DeadCodeElimination();

impl GlobalOptimizationPass for DeadCodeElimination {
    fn run(function: &mut FunctionBlocks) {
        loop {
            // delete unused vars until convergence
            // this is not the most efficient way to implement this, but it works
            let any_deleted = delete_unused_vars(function);
            if !any_deleted {
                break;
            }
        }
    }
}

// returns true if any instructions were deleted. false otherwise
fn delete_unused_vars(function: &mut FunctionBlocks) -> bool {
    let mut used_args: HashSet<String> = HashSet::new();
    let mut dests: HashSet<String> = HashSet::new();

    for block in function.get_blocks() {
        for instr in &block.instrs {
            let args = instr.get_args_copy();
            for arg in args.into_iter() {
                used_args.insert(arg);
            }

            let dest = instr.get_dest();
            if let Some(dest_str) = dest {
                dests.insert(dest_str.to_string());
            }
        }
    }

    // to find unused vars, we want to find elements in dests not in used_args
    let unused: HashSet<_> = dests.difference(&used_args).collect();
    for block in function.get_mut_blocks() {
        let mut new_instrs = Vec::new();
        mem::swap(&mut block.instrs, &mut new_instrs);
        new_instrs = new_instrs
            .into_iter()
            .filter(|instr| {
                instr.get_dest().is_none()
                    || !unused.contains(&instr.get_dest().unwrap().to_string())
            })
            .collect();

        mem::swap(&mut block.instrs, &mut new_instrs);
    }

    unused.len() > 0
}