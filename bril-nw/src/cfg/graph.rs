use std::collections::{HashMap, HashSet, VecDeque};

use crate::basicblock::BasicBlock;

const BLOCK_NAME_PFX: &'static str = "block_";

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub predecessors: HashMap<usize, Vec<usize>>,
    pub successors: HashMap<usize, Vec<usize>>,
    all_block_ids: Vec<usize>,
}

pub type Dominators = HashMap<usize, HashSet<usize>>;
pub type StrictDominators = Dominators;

pub type ImmediateDominators = HashMap<usize, usize>;
pub type DominatorTree = HashMap<usize, HashSet<usize>>;

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

    pub fn find_dominators(&self) -> Dominators {
        let mut dominators: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut should_continue = true;

        let all_block_ids_set = self
            .all_block_ids
            .iter()
            .copied()
            .collect::<HashSet<usize>>();
        for block_id in &self.all_block_ids {
            if *block_id == 0 {
                dominators.insert(0, HashSet::from([0]));
            } else {
                dominators.insert(*block_id, all_block_ids_set.clone());
            }
        }

        while should_continue {
            should_continue = false;

            // traversing in reverse post-order is most optimal for well-behaved reducible cfgs
            // but this isn't too bad
            // natural loop - single entry (in-edge) into the cycle
            // c-like languages (minus goto) mostly only have natural loops
            // back edge - an edge A (tail) -> B (head) where B dominates A
            // more formally - for a back edge A -> B: smallest set of vertices L including A and B s.t. for all v in L, PREDS(v) in L OR v = B
            // reducible control flow: every back edge has a natural loop
            // e.g. if you remove all edges traversed after a BFS, the remainder are back edges
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

    pub fn find_immediate_dominators(&self, dominators: &Dominators) -> ImmediateDominators {
        let mut result: HashMap<usize, usize> = HashMap::new();
        for block_id in &self.all_block_ids {
            if *block_id == 0 {
                continue; // entry node has no immediate dominator
            }

            result.insert(
                *block_id,
                self.find_immediate_dominator(*block_id, dominators.get(block_id).unwrap()),
            );
        }

        result
    }

    pub fn find_immediate_dominator(
        &self,
        block_id: usize,
        block_dominators: &HashSet<usize>,
    ) -> usize {
        // run bfs through predecessors, returning the first node that is a dominator of block_id
        let mut open_set: VecDeque<usize> = VecDeque::new();
        let mut closed_set: HashSet<usize> = HashSet::new();
        for pred in self.predecessors.get(&block_id).unwrap_or(&Vec::new()) {
            open_set.push_back(*pred);
        }

        while !open_set.is_empty() {
            let next = open_set.pop_front().unwrap();
            if block_dominators.contains(&next) {
                return next;
            } else {
                closed_set.insert(next);
                for pred in self.predecessors.get(&next).unwrap_or(&Vec::new()) {
                    if !closed_set.contains(pred) {
                        open_set.push_back(*pred);
                    }
                }
            }
        }

        // every node has an immediate dominator. don't think we should be getting here.
        0
    }

    pub fn create_dominator_tree(&self, dominators: Dominators) -> DominatorTree {
        let strict_dominators = retain_only_strict_dominators(dominators);
        let immediate_dominators = self.find_immediate_dominators(&strict_dominators);

        let mut result = DominatorTree::new();

        for block_id in immediate_dominators.keys() {
            let immediate_dominator = immediate_dominators.get(block_id).unwrap();
            if result.contains_key(immediate_dominator) {
                result
                    .get_mut(immediate_dominator)
                    .unwrap()
                    .insert(*block_id);
            } else {
                result.insert(*immediate_dominator, HashSet::from([*block_id]));
            }
        }

        result
    }
}

pub fn retain_only_strict_dominators(dominators: Dominators) -> StrictDominators {
    let block_ids = dominators.keys().copied().collect::<Vec<usize>>();

    let mut result: HashMap<usize, HashSet<usize>> = HashMap::new();

    for block_id in block_ids {
        let block_dominators = dominators.get(&block_id);

        if let None = block_dominators {
            continue;
        }

        let mut block_dominators = block_dominators.unwrap().clone();
        block_dominators.remove(&block_id);

        result.insert(block_id, block_dominators);
    }

    result
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
