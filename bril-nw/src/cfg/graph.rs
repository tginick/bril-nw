use std::collections::HashMap;

use crate::basicblock::BasicBlock;

const BLOCK_NAME_PFX: &'static str = "block_";

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub predecessors: HashMap<usize, Vec<usize>>,
    pub successors: HashMap<usize, Vec<usize>>,
}

impl ControlFlowGraph {
    pub fn create_from_basic_blocks(blocks: &Vec<BasicBlock>) -> Self {
        let identifiers = identify_basic_blocks(&blocks);
        let mut predecessors: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut successors: HashMap<usize, Vec<usize>> = HashMap::new();

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
                    .map(|l| format!("{}{}", BLOCK_NAME_PFX, l))
                    .collect();
                let mut target_idxs = Vec::new();
                for target in targets {
                    if identifiers.contains_key(&target) {
                        target_idxs.push(*identifiers.get(&target).unwrap());
                    } else {
                        // TODO: bad
                    }
                }

                // add predecessors
                for target_idx in &target_idxs {
                    if predecessors.contains_key(target_idx) {
                        predecessors
                            .get_mut(target_idx)
                            .unwrap()
                            .push(blocks[i].get_id());
                    } else {
                        predecessors.insert(*target_idx, vec![blocks[i].get_id()]);
                    }
                }

                successors.insert(blocks[i].get_id(), target_idxs);
            } else if blocks[i].instrs[last_instr_idx].is_ret() {
                // do nothing. this block has no successors
                continue;
            } else if i < blocks.len() - 1 {
                // not a jump or ret but last instr so just point to next basic block
                successors.insert(blocks[i].get_id(), vec![i + 1]);

                let next_idx = i + 1;
                if predecessors.contains_key(&next_idx) {
                    predecessors
                        .get_mut(&next_idx)
                        .unwrap()
                        .push(blocks[i].get_id());
                } else {
                    predecessors.insert(next_idx, vec![blocks[i].get_id()]);
                }
            }
        }

        ControlFlowGraph {
            predecessors,
            successors,
        }
    }
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
                    "{}{}",
                    BLOCK_NAME_PFX,
                    cur_block.instrs[0].get_label().unwrap().to_string()
                ),
                i,
            );
        } else {
            identifiers.insert(format!("{}{}", BLOCK_NAME_PFX, identifiers.len()), i);
        }
    }

    identifiers
}
