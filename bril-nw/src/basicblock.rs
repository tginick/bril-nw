use std::{collections::HashSet, rc::Rc};

use crate::bril::types::{Function, Instruction};

lazy_static! {
    static ref TERMINATOR_INSTS: HashSet<&'static str> = {
        let mut insts = HashSet::new();
        insts.insert("br");
        insts.insert("jmp");
        insts.insert("ret");

        insts
    };
}

#[derive(Debug)]
pub struct BasicBlock {
    pub instrs: Vec<Rc<Instruction>>,
}

impl BasicBlock {
    pub fn new(instrs: Vec<Rc<Instruction>>) -> Self {
        BasicBlock { instrs }
    }
}

pub fn load_function_blocks(function: Rc<Function>) -> Vec<BasicBlock> {
    let mut blocks: Vec<BasicBlock> = Vec::new();

    let mut cur_block_instrs: Vec<Rc<Instruction>> = Vec::new();
    for instr in &function.instrs {
        if instr.is_instr() {
            cur_block_instrs.push(instr.clone());

            if TERMINATOR_INSTS.contains(instr.get_op_code().unwrap()) {
                blocks.push(BasicBlock {
                    instrs: cur_block_instrs.clone(),
                });
                cur_block_instrs.clear();
            }
        } else if instr.is_label() {
            blocks.push(BasicBlock {
                instrs: cur_block_instrs.clone(),
            });
            cur_block_instrs.clear();

            // the label will go in the beginning of the next basicblock
            cur_block_instrs.push(instr.clone());
        }
    }

    // yield the final basic block
    blocks.push(BasicBlock {
        instrs: cur_block_instrs,
    });

    blocks
}
