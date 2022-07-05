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

#[cfg(test)]
mod tests {
    use crate::{
        basicblock::{BasicBlock, FunctionBlocks},
        bril::types::{Instruction, Type, Value},
        opt::GlobalOptimizationPass,
    };

    use super::DeadCodeElimination;

    #[test]
    fn test_1() {
        let instrs = vec![
            Instruction::new_const("const", "a".to_string(), Type::Int, Value::Int(4)),
            Instruction::new_const("const", "b".to_string(), Type::Int, Value::Int(2)),
            // following instr is eliminated
            Instruction::new_const("const", "c".to_string(), Type::Int, Value::Int(1)),
            Instruction::new_value(
                "add",
                "d".to_string(),
                Type::Int,
                vec!["a".to_string(), "b".to_string()],
                vec![],
                vec![],
            ),
            // following instr is eliminated
            Instruction::new_value(
                "add",
                "e".to_string(),
                Type::Int,
                vec!["c".to_string(), "d".to_string()],
                vec![],
                vec![],
            ),
            Instruction::new_effect("print", vec!["d".to_string()], vec![], vec![]),
        ];

        let bb = BasicBlock::new(0, instrs);
        let mut f = FunctionBlocks::new(vec![bb]);

        DeadCodeElimination::run(&mut f);

        let updated_bb = &f.get_blocks()[0];
        assert_eq!(updated_bb.instrs.len(), 4);

        assert_eq!(updated_bb.instrs[0].get_dest(), Some("a"));
        assert_eq!(updated_bb.instrs[1].get_dest(), Some("b"));
        assert_eq!(updated_bb.instrs[2].get_dest(), Some("d"));
    }
}
