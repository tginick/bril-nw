use std::collections::{HashMap, HashSet};

use crate::{
    basicblock::FunctionBlocks,
    bril::types::{Instruction, OpCode, Type},
    cfg::{graph::DominatorTree, ControlFlowGraph},
};

pub fn insert_phi_nodes(
    cfg: &ControlFlowGraph,
    dom_tree: &DominatorTree,
    blocks: &mut FunctionBlocks,
) {
    let mut all_vars = find_all_vars(blocks);
    let mut added_phi_nodes: HashMap<String, HashSet<usize>> = HashMap::new();

    for (var, block_ids) in all_vars.iter_mut() {
        for (block_id, var_type) in block_ids.iter() {
            let dom_frontier = cfg.get_dominance_frontier(dom_tree, *block_id);
            for dom_frontier_block_id in dom_frontier {
                let phi = Instruction::new_value(
                    OpCode::Phi,
                    var.clone(),
                    *var_type,
                    vec![], // to be filled in later after variable renaming
                    vec![],
                    vec![],
                );

                blocks
                    .get_mut_block_by_id(dom_frontier_block_id)
                    .unwrap()
                    .instrs
                    .insert(0, phi.clone());
            }
        }
    }
}

fn find_all_vars(blocks: &FunctionBlocks) -> HashMap<String, HashSet<(usize, Type)>> {
    let mut r: HashMap<String, HashSet<(usize, Type)>> = HashMap::new();

    for block in blocks.get_blocks() {
        for instr in &block.instrs {
            let maybe_dest = instr.get_dest();
            let maybe_type = instr.get_type();
            if let Some(dest) = maybe_dest {
                r.entry(dest.to_string())
                    .or_insert(HashSet::from([(block.get_id(), maybe_type.unwrap())]))
                    .insert((block.get_id(), maybe_type.unwrap()));
            }
        }
    }

    r
}
