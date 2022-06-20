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
    id: usize,
    pub instrs: Vec<Rc<Instruction>>,
}

#[derive(Debug)]
pub struct FunctionBlocks {
    blocks: Vec<BasicBlock>,
}

impl BasicBlock {
    pub fn new(id: usize, instrs: Vec<Rc<Instruction>>) -> Self {
        BasicBlock { id, instrs }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}

pub fn load_function_blocks(function: Rc<Function>) -> FunctionBlocks {
    let mut blocks: Vec<BasicBlock> = Vec::new();

    let mut cur_id: usize = 0;

    let mut cur_block_instrs: Vec<Rc<Instruction>> = Vec::new();
    for instr in &function.instrs {
        if instr.is_instr() {
            cur_block_instrs.push(instr.clone());

            if TERMINATOR_INSTS.contains(instr.get_op_code().unwrap()) {
                blocks.push(BasicBlock {
                    id: cur_id,
                    instrs: cur_block_instrs.clone(),
                });

                cur_id += 1;
                cur_block_instrs.clear();
            }
        } else if instr.is_label() {
            if !cur_block_instrs.is_empty() {
                blocks.push(BasicBlock {
                    id: cur_id,
                    instrs: cur_block_instrs.clone(),
                });

                cur_id += 1;
                cur_block_instrs.clear();
            }

            // the label will go in the beginning of the next basicblock
            cur_block_instrs.push(instr.clone());
        }
    }

    // yield the final basic block
    if !cur_block_instrs.is_empty() {
        blocks.push(BasicBlock {
            id: cur_id,
            instrs: cur_block_instrs,
        });
    }

    FunctionBlocks { blocks }
}

impl FunctionBlocks {
    pub fn get_blocks(&self) -> &Vec<BasicBlock> {
        &self.blocks
    }

    pub fn get_mut_blocks(&mut self) -> &mut Vec<BasicBlock> {
        &mut self.blocks
    }
}
