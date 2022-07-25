use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use crate::{
    basicblock::FunctionBlocks,
    bril::types::{Instruction, InstructionScaffold, OpCode, Type},
    cfg::{graph::DominatorTree, ControlFlowGraph},
};

struct SSAStack {
    stack: Vec<String>,
    next_name_id: usize,
}

struct SSABuilder<'a> {
    cfg: &'a ControlFlowGraph,
    dom_tree: &'a DominatorTree,
    blocks: &'a mut FunctionBlocks,
    all_vars: HashMap<String, HashSet<(usize, Type)>>,
    staged_phi_nodes: HashMap<usize, HashMap<String, InstructionScaffold>>,

    rename_vars_stacks: HashMap<String, SSAStack>, // for each var, have a stack of renamed vars

    // mostly for dev/debug purposes. vec of (block id, var name that couldn't be renamed)
    rename_failures: Vec<(usize, String)>,
}

impl SSAStack {
    pub fn new() -> Self {
        SSAStack {
            stack: Vec::new(),
            next_name_id: 0,
        }
    }

    pub fn peek(&self) -> Option<&String> {
        if !self.is_empty() {
            Some(&self.stack[self.stack.len() - 1])
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len() > 0
    }

    pub fn create_new_name(&mut self, old_name: &str) -> String {
        let result = format!("{}.{}", old_name, self.next_name_id);
        self.next_name_id += 1;

        self.stack.push(result.clone());

        result
    }
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
            rename_failures: Vec::new(),
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

                    staged_phi_nodes
                        .get_mut(&dom_frontier_block_id)
                        .unwrap()
                        .insert(var.clone(), (&phi).into());

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
        let block = self.blocks.get_mut_block_by_id(block_id).unwrap();
        let mut num_names_created: HashMap<String, usize> = HashMap::new();

        for instr in &mut block.instrs {
            let mut new_instr = instr.as_ref().clone();
            let maybe_new_instr_args = new_instr.get_args_mut();
            if let Some(new_instr_args) = maybe_new_instr_args {
                for arg in new_instr_args {
                    let arg_name_stack = self
                        .rename_vars_stacks
                        .entry(arg.clone())
                        .or_insert(SSAStack::new());

                    if arg_name_stack.is_empty() {
                        self.rename_failures.push((block_id, arg.clone()));
                        break;
                    }

                    *arg = arg_name_stack.peek().unwrap().clone();
                }
            }

            let maybe_old_dest = new_instr.get_dest();
            if let Some(old_dest) = maybe_old_dest {
                let arg_name_stack = self
                    .rename_vars_stacks
                    .entry(old_dest.to_string())
                    .or_insert(SSAStack::new());

                let new_dest = arg_name_stack.create_new_name(old_dest);

                let num_names_created_for_var =
                    num_names_created.entry(old_dest.to_string()).or_insert(0);
                *num_names_created_for_var += 1;

                new_instr.set_dest(new_dest);
            }

            *instr = Rc::new(new_instr);
        }

        for successor_id in self.cfg.successors.get(&block_id).unwrap_or(&Vec::new()) {
            for (var_name, successor_phi_node) in self
                .staged_phi_nodes
                .get(successor_id)
                .unwrap_or(&HashMap::new())
            {
                let instr = (*successor_phi_node.0).borrow_mut();

                let arg_name_stack = self
                    .rename_vars_stacks
                    .entry(var_name.clone())
                    .or_insert(SSAStack::new());
            }
        }

        for dominated_id in self.dom_tree.get(&block_id).unwrap_or(&HashSet::new()) {
            self.rename_vars_rec(*dominated_id);
        }

        for (var_name, num_names_created_for_var) in num_names_created {
            let stack = &mut self.rename_vars_stacks.get_mut(&var_name).unwrap().stack;
            let new_stack_len = stack.len().saturating_sub(num_names_created_for_var);
            stack.truncate(new_stack_len);
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
