use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
};

use crate::bril::types::{Function, FunctionArg, Instruction, OpCode};

lazy_static! {
    static ref TERMINATOR_INSTS: HashSet<OpCode> = {
        let mut insts = HashSet::new();
        insts.insert(OpCode::Branch);
        insts.insert(OpCode::Jump);
        insts.insert(OpCode::Ret);

        insts
    };
}

#[derive(Debug)]
pub struct BasicBlock {
    id: usize,
    name: RefCell<String>,
    pub instrs: Vec<Rc<Instruction>>,
}

#[derive(Debug)]
pub struct FunctionBlocks {
    name: String,
    args: Vec<Rc<FunctionArg>>,
    blocks: Vec<BasicBlock>,
    block_id_to_idx: HashMap<usize, usize>,
}

impl BasicBlock {
    pub fn new(id: usize, instrs: Vec<Rc<Instruction>>) -> Self {
        BasicBlock {
            id,
            instrs,
            name: RefCell::new("".to_string()),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn set_name(&self, new_name: &str) {
        *self.name.borrow_mut() = new_name.to_string();
    }

    pub fn get_name(&self) -> String {
        self.name.borrow().clone()
    }
}

pub fn load_function_blocks(function: Rc<Function>) -> FunctionBlocks {
    let mut blocks: Vec<BasicBlock> = Vec::new();
    let mut block_id_to_idx: HashMap<usize, usize> = HashMap::new();

    let mut cur_id: usize = 0;

    let mut cur_block_instrs: Vec<Rc<Instruction>> = Vec::new();
    for instr in &function.instrs {
        if instr.is_instr() {
            cur_block_instrs.push(instr.clone());

            if TERMINATOR_INSTS.contains(&instr.get_op_code().unwrap()) {
                add_block(
                    &mut blocks,
                    &mut block_id_to_idx,
                    cur_id,
                    cur_block_instrs.clone(),
                );

                cur_id += 1;
                cur_block_instrs.clear();
            }
        } else if instr.is_label() {
            if !cur_block_instrs.is_empty() {
                add_block(
                    &mut blocks,
                    &mut block_id_to_idx,
                    cur_id,
                    cur_block_instrs.clone(),
                );

                cur_id += 1;
                cur_block_instrs.clear();
            }

            // the label will go in the beginning of the next basicblock
            cur_block_instrs.push(instr.clone());
        }
    }

    // yield the final basic block
    if !cur_block_instrs.is_empty() {
        add_block(&mut blocks, &mut block_id_to_idx, cur_id, cur_block_instrs);
    }

    FunctionBlocks::new(
        &function.name,
        function.args.clone(),
        blocks,
        block_id_to_idx,
    )
}

fn add_block(
    blocks: &mut Vec<BasicBlock>,
    block_id_to_idx: &mut HashMap<usize, usize>,
    new_id: usize,
    instrs: Vec<Rc<Instruction>>,
) {
    let next_idx = blocks.len();
    block_id_to_idx.insert(new_id, next_idx);

    blocks.push(BasicBlock::new(new_id, instrs));
}

impl FunctionBlocks {
    pub fn new(
        name: &str,
        args: Vec<Rc<FunctionArg>>,
        blocks: Vec<BasicBlock>,
        block_id_to_idx: HashMap<usize, usize>,
    ) -> Self {
        FunctionBlocks {
            name: name.to_string(),
            args,
            blocks,
            block_id_to_idx,
        }
    }
    pub fn get_blocks(&self) -> &Vec<BasicBlock> {
        &self.blocks
    }

    pub fn get_mut_blocks(&mut self) -> &mut Vec<BasicBlock> {
        &mut self.blocks
    }

    pub fn get_block_by_id(&self, id: usize) -> Option<&BasicBlock> {
        let idx = self.block_id_to_idx.get(&id);
        if let None = idx {
            return None;
        }

        Some(&self.blocks[*idx.unwrap()])
    }

    pub fn get_mut_block_by_id(&mut self, id: usize) -> Option<&mut BasicBlock> {
        let idx = self.block_id_to_idx.get(&id);
        if let None = idx {
            return None;
        }

        Some(&mut self.blocks[*idx.unwrap()])
    }

    pub fn get_args(&self) -> &Vec<Rc<FunctionArg>> {
        &self.args
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_block_name(&self, id: usize) -> Option<String> {
        self.get_block_by_id(id).map(|b| b.get_name())
    }
}

impl fmt::Display for FunctionBlocks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{}(", self.get_name())?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(
                f,
                "{}: {}{}",
                &arg.name,
                arg.arg_type,
                if i < self.args.len() - 1 {
                    ", "
                } else {
                    ") {\n"
                }
            )?;
        }

        for block in &self.blocks {
            writeln!(f, "#{}", block.name.borrow())?;
            write!(f, "{}", block)?;
        }

        Ok(())
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for instr in &self.instrs {
            write!(f, "{}", instr)?;
        }

        Ok(())
    }
}
