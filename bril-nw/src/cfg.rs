use std::collections::HashMap;

use crate::basicblock::BasicBlock;

pub struct ControlFlowGraph {
    basic_blocks: Vec<BasicBlock>,
}

fn identify_basic_blocks(blocks: &Vec<BasicBlock>) -> HashMap<String, usize> {
    let mut identifiers = HashMap::new();
    for i in 0..blocks.len() {
        let cur_block = &blocks[i];
        if cur_block.instrs.is_empty() {
            continue;
        }

        if cur_block.instrs[0].is_label() {
            identifiers.insert(cur_block.instrs[0].get_label().unwrap().to_string(), i);
        }
    }

    identifiers
}
