use std::collections::HashMap;

use crate::basicblock::BasicBlock;

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub edges: HashMap<usize, Vec<usize>>,
}

pub fn create_control_flow_graph(blocks: &Vec<BasicBlock>) -> ControlFlowGraph {
    let identifiers = identify_basic_blocks(&blocks);
    let mut edges: HashMap<usize, Vec<usize>> = HashMap::new();

    for i in 0..blocks.len() {
        let last_instr_idx = blocks[i].instrs.len() - 1;

        // check if last instr is a jump
        // if yes, create edges based on jump target
        // if not, create an edge to the next block
        if blocks[i].instrs[last_instr_idx].is_jump() {
            let targets: Vec<String> = blocks[i].instrs[last_instr_idx]
                .get_jump_target()
                .unwrap()
                .iter()
                .map(|l| format!("block_{}", l))
                .collect();
            let mut target_idxs = Vec::new();
            for target in targets {
                if identifiers.contains_key(&target) {
                    target_idxs.push(*identifiers.get(&target).unwrap());
                }
            }

            edges.insert(blocks[i].get_id(), target_idxs);

            // the rest of the instructions in this block will not be executed
            // this is guaranteed, as a jump is a terminator instruction
            break;
        } else if blocks[i].instrs[last_instr_idx].is_ret() {
            // do nothing. this block has no successors
            continue;
        } else if i < blocks.len() - 1 {
            // not a jump or ret but last instr so just point to next basic block
            edges.insert(blocks[i].get_id(), vec![i + 1]);
        }
    }

    ControlFlowGraph { edges }
}

fn identify_basic_blocks(blocks: &Vec<BasicBlock>) -> HashMap<String, usize> {
    let mut identifiers = HashMap::new();
    for i in 0..blocks.len() {
        let cur_block = &blocks[i];
        if cur_block.instrs.is_empty() {
            continue;
        }

        if cur_block.instrs[0].is_label() {
            identifiers.insert(
                format!(
                    "block_{}",
                    cur_block.instrs[0].get_label().unwrap().to_string()
                ),
                i,
            );
        } else {
            identifiers.insert(format!("block_{}", identifiers.len()), i);
        }
    }

    identifiers
}
