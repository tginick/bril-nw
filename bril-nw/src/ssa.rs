use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    basicblock::FunctionBlocks,
    bril::types::{Instruction, InstructionScaffold, OpCode, Type},
    cfg::{graph::DominatorTree, ControlFlowGraph},
};

struct SSABuilder<'a> {
    cfg: &'a ControlFlowGraph,
    dom_tree: &'a DominatorTree,
    blocks: &'a mut FunctionBlocks,
    all_vars: HashMap<String, HashSet<(usize, Type)>>,
    staged_phi_nodes: HashMap<usize, HashMap<String, InstructionScaffold>>,

    rename_vars_stacks: HashMap<String, Vec<String>>, // for each var, have a stack of renamed vars
}

impl<'a> SSABuilder<'a> {
    pub fn new(
        cfg: &'a ControlFlowGraph,
        dom_tree: &'a DominatorTree,
        blocks: &'a mut FunctionBlocks,
    ) -> Self {
        let mut ssa_builder = SSABuilder {
            cfg,
            dom_tree,
            blocks,
            all_vars: HashMap::new(),
            staged_phi_nodes: HashMap::new(),

            rename_vars_stacks: HashMap::new(),
        };

        let all_vars = ssa_builder.find_all_vars();

        ssa_builder.all_vars = all_vars;

        ssa_builder
    }

    pub fn convert_to_ssa_form(&mut self) {
        self.insert_phi_nodes();
        self.rename_vars();
    }

    fn find_all_vars(&self) -> HashMap<String, HashSet<(usize, Type)>> {
        let mut r: HashMap<String, HashSet<(usize, Type)>> = HashMap::new();

        for block in self.blocks.get_blocks() {
            for instr in &block.instrs {
                let maybe_dest = instr.get_dest();
                if let Some(dest) = maybe_dest {
                    let var_type = instr.get_type().unwrap();

                    r.entry(dest.to_string())
                        .or_insert(HashSet::from([(block.get_id(), var_type)]))
                        .insert((block.get_id(), var_type));
                }
            }
        }

        r
    }

    fn insert_phi_nodes(&mut self) {
        let mut staged_phi_nodes: HashMap<usize, HashMap<String, InstructionScaffold>> =
            HashMap::new();

        for (var, block_ids_declaring_var) in self.all_vars.iter_mut() {
            let mut phi_insertion_candidate_blocks = VecDeque::from(
                block_ids_declaring_var
                    .iter()
                    .cloned()
                    .collect::<Vec<(usize, Type)>>(),
            );

            while !phi_insertion_candidate_blocks.is_empty() {
                let (block_id, var_type) = phi_insertion_candidate_blocks.pop_front().unwrap();
                let dom_frontier = self.cfg.get_dominance_frontier(self.dom_tree, block_id);
                for dom_frontier_block_id in dom_frontier {
                    // if we already added a phi node for this var into this block, don't do so again
                    if staged_phi_nodes
                        .entry(dom_frontier_block_id)
                        .or_insert(HashMap::new())
                        .get(var)
                        .map_or(false, |_| true)
                    {
                        continue;
                    }

                    let phi = Instruction::new_value(
                        OpCode::Phi,
                        var.clone(),
                        var_type,
                        vec![], // to be filled in later after variable renaming
                        vec![],
                        vec![],
                    );

                    staged_phi_nodes.get_mut(&dom_frontier_block_id).unwrap().insert(var.clone(), (&phi).into());

                    block_ids_declaring_var.insert((dom_frontier_block_id, var_type));

                    // this dom frontier block now declares v so we need to add it to the queue
                    phi_insertion_candidate_blocks.push_back((dom_frontier_block_id, var_type));
                }
            }
        }

        self.staged_phi_nodes = staged_phi_nodes;
    }

    fn rename_vars(&mut self) {
        self.rename_vars_rec(0);
    }

    // this function should only be called from rename_vars
    fn rename_vars_rec(&mut self, block_id: usize) {
        let mut block = self.blocks.get_mut_block_by_id(block_id).unwrap();
        for instr in &mut block.instrs {
            
        }
    }
}

pub fn convert_to_ssa_form(
    cfg: &ControlFlowGraph,
    dom_tree: &DominatorTree,
    blocks: &mut FunctionBlocks,
) {
    /*

    // variable decl -> block id where it was added
    let mut added_phi_nodes: HashMap<String, HashSet<usize>> = HashMap::new();

    insert_phi_nodes(cfg, dom_tree, blocks, &mut all_vars, &mut added_phi_nodes);
    */

    let mut ssa_builder = SSABuilder::new(cfg, dom_tree, blocks);
    ssa_builder.convert_to_ssa_form();
}
