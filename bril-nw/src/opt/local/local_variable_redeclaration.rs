use std::{
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
};

use crate::{basicblock::BasicBlock, bril::types::Instruction, opt::LocalOptimizationPass};

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
        loop {
            let did_delete_instructions = delete_unused_assignments(block);
            if !did_delete_instructions {
                break;
            }
        }
    }
}

fn delete_unused_assignments(block: &mut BasicBlock) -> bool {
    let mut last_def: HashMap<String, Rc<Instruction>> = HashMap::new();
    let mut instrs_to_delete: HashSet<*const Instruction> = HashSet::new();
    for instr in &block.instrs {
        // check for uses
        let maybe_args = instr.get_args();
        if let Some(args) = maybe_args {
            for arg in args {
                last_def.remove(arg);
            }
        }

        // check for assigns
        let maybe_dest = instr.get_dest();
        if let Some(dest) = maybe_dest {
            if last_def.contains_key(dest) {
                // actually stage the instruction for deletion
                instrs_to_delete.insert(Rc::as_ptr(last_def.get(dest).unwrap()));
            }
            last_def.insert(dest.to_string(), instr.clone());
        }
    }

    if instrs_to_delete.is_empty() {
        return false;
    }

    let mut new_instrs: Vec<Rc<Instruction>> = Vec::new();
    mem::swap(&mut new_instrs, &mut block.instrs);

    let mut filtered_instrs = new_instrs
        .into_iter()
        .filter(|instr| !instrs_to_delete.contains(&Rc::as_ptr(instr)))
        .collect();

    mem::swap(&mut filtered_instrs, &mut block.instrs);

    return true;
}
