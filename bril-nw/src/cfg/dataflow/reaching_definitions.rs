use std::{
    collections::{hash_map::DefaultHasher, BTreeSet, HashMap},
    hash::{Hash, Hasher},
};

use crate::{
    basicblock::{BasicBlock, FunctionBlocks},
    cfg::ControlFlowGraph,
};

type IdentifiedDeclaration = (usize, String);

pub struct ReachingDefinitions();

/*
    A instruction d defining variable v REACHES another instruction u iff
    there exists a path in the CFG from d to u where along that path
        - there are no other assignments to v

    "use" - an instruction uses all its arguments
    "define" - an instruction defines the variable it writes to
    "available" - definitions that REACH a given program point are available there
    "kill" - Any definition kills all of the currently available definitions

    For every definition and every use, determine whether the definition reaches the use
*/
impl ReachingDefinitions {
    pub fn new() -> Self {
        ReachingDefinitions()
    }

    pub fn analyze(&self, cfg: &ControlFlowGraph, function: &FunctionBlocks) {
        if function.get_blocks().is_empty() {
            return;
        }

        let mut inputs: HashMap<usize, BTreeSet<IdentifiedDeclaration>> = HashMap::new();
        let mut outputs: HashMap<usize, BTreeSet<IdentifiedDeclaration>> = HashMap::new();

        let init_block_id = function.get_blocks()[0].get_id();

        // init state of this data flow analysis is the args of the function
        let init_inputs: BTreeSet<IdentifiedDeclaration> = function
            .get_args()
            .iter()
            .map(|a| (init_block_id, a.name.clone()))
            .collect();

        // TODO: i think idx 0 should be the first function block?
        inputs.insert(init_block_id, init_inputs);

        // add all blocks to the worklist
        let mut work_list: BTreeSet<usize> = BTreeSet::new();
        for block in function.get_blocks() {
            work_list.insert(block.get_id());
        }

        // forward worklist algorithm
        while !work_list.is_empty() {
            let block_id = *work_list.iter().next().unwrap();
            let block = function.get_block_by_id(block_id).unwrap();

            if block_id != init_block_id {
                // merge
                // in[b] = merge (out[p] for each predecessor p of b)
                let maybe_predecessors = cfg.predecessors.get(&block_id);
                if let Some(predecessors) = maybe_predecessors {
                    let merged_input: BTreeSet<IdentifiedDeclaration> = predecessors
                        .iter()
                        .map(|pred_id| outputs.get(pred_id).map_or(BTreeSet::new(), |o| o.clone()))
                        .fold(
                            BTreeSet::<IdentifiedDeclaration>::new(),
                            |mut accum, out| {
                                accum.extend(out);
                                accum
                            },
                        );
                    inputs.insert(block_id, merged_input);
                }
            }

            // transfer
            // out[b] = transfer(b, in[b])
            // for reaching definitions this is DEF[b] U (in[b] - KILL[b])
            let defs = get_defs(block);

            let input_copy = inputs.get(&block_id).unwrap().clone();

            let updated_outputs = transfer_defs(block_id, defs, input_copy);
            let maybe_current_outputs = outputs.get(&block_id);
            if let Some(current_outputs) = maybe_current_outputs {
                if check_different(&updated_outputs, current_outputs) {
                    // successors need to be added to work list
                    let successors = cfg.successors.get(&block_id);
                    if let Some(successors) = successors {
                        for successor in successors {
                            work_list.insert(*successor);
                        }
                    }
                }
            }
            outputs.insert(block_id, updated_outputs);
        }
    }
}

// gets all vars that have been assigned to in this block
fn get_defs(block: &BasicBlock) -> BTreeSet<String> {
    block
        .instrs
        .iter()
        .fold(BTreeSet::<String>::new(), |mut accum, instr| {
            let maybe_dest = instr.get_dest();
            if let Some(dest) = maybe_dest {
                accum.insert(dest.to_string());
            }

            accum
        })
}

fn transfer_defs(
    block_id: usize,
    defs: BTreeSet<String>,
    input: BTreeSet<IdentifiedDeclaration>,
) -> BTreeSet<IdentifiedDeclaration> {
    let mut kills: BTreeSet<IdentifiedDeclaration> = BTreeSet::new();
    for (other_block_id, def_name) in &input {
        if defs.contains(def_name) {
            kills.insert((*other_block_id, def_name.clone()));
        }
    }

    let diff: BTreeSet<IdentifiedDeclaration> = input.difference(&kills).cloned().collect();
    let identified_defs: BTreeSet<IdentifiedDeclaration> =
        defs.into_iter().map(|decl| (block_id, decl)).collect();

    diff.union(&identified_defs).cloned().collect()
}

fn check_different(
    updated: &BTreeSet<IdentifiedDeclaration>,
    current: &BTreeSet<IdentifiedDeclaration>,
) -> bool {
    if updated.len() != current.len() {
        return true;
    }

    if calc_hash(updated) != calc_hash(current) {
        return true;
    }

    return false;
}

fn calc_hash(d: &BTreeSet<IdentifiedDeclaration>) -> u64 {
    let mut h = DefaultHasher::new();
    d.hash(&mut h);

    h.finish()
}
