use std::collections::{HashMap, HashSet};

use crate::basicblock::BasicBlock;

const BLOCK_NAME_PFX: &'static str = "block_";

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub predecessors: HashMap<usize, Vec<usize>>,
    pub successors: HashMap<usize, Vec<usize>>,
    all_block_ids: Vec<usize>,
}

impl ControlFlowGraph {
    pub fn create_from_basic_blocks(blocks: &Vec<BasicBlock>) -> Self {
        let identifiers = identify_basic_blocks(&blocks);
        let mut predecessors: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut successors: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut all_block_ids: Vec<usize> = Vec::with_capacity(blocks.len());

        for i in 0..blocks.len() {
            let last_instr_idx = blocks[i].instrs.len() - 1;

            all_block_ids.push(blocks[i].get_id());

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
            all_block_ids,
        }
    }

    pub fn get_dominators(&self) -> HashMap<usize, HashSet<usize>> {
        let mut dominators: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut should_continue = true;

        while should_continue {
            should_continue = false;
            for block_id in &self.all_block_ids {
                // a block A is "dominated" by another block B if B dominates all of A's predecessors
                let block_predecessors = self.predecessors.get(block_id);
                if let None = block_predecessors {
                    continue;
                }

                let block_predecessors = block_predecessors.unwrap();
                let block_pred_dominator_estimates: Vec<HashSet<usize>> = block_predecessors
                    .iter()
                    .map(|pred_id| {
                        dominators
                            .get(pred_id)
                            .map_or(HashSet::new(), |v| v.clone())
                    })
                    .collect();

                let mut block_pred_dominator_intersection = block_pred_dominator_estimates
                    .into_iter()
                    .fold(HashSet::new(), |a, h| a.intersection(&h).copied().collect());

                // domination is reflexive
                block_pred_dominator_intersection.insert(*block_id);

                let current_dominator_set = dominators.get(block_id);
                if let None = current_dominator_set {
                    should_continue = true;
                }

                let current_dominator_set = current_dominator_set.unwrap();
                if current_dominator_set != &block_pred_dominator_intersection {
                    should_continue = true;
                }

                dominators.insert(*block_id, block_pred_dominator_intersection);
            }
        }

        dominators
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
            let block_name = format!(
                "{}{}",
                BLOCK_NAME_PFX,
                cur_block.instrs[0].get_label().unwrap().to_string()
            );

            cur_block.set_name(&block_name);

            identifiers.insert(block_name, i);
        } else {
            let block_name = format!("{}{}", BLOCK_NAME_PFX, identifiers.len());
            cur_block.set_name(&block_name);

            identifiers.insert(block_name, i);
        }
    }

    identifiers
}
