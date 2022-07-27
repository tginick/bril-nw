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

const BLOCK_NAME_PFX: &'static str = "block_";

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
    block_name_to_id: HashMap<String, usize>,
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

pub struct FunctionBlocksLoader {
    function: Rc<Function>,
    cur_id: usize,
    block_id_to_idx: HashMap<usize, usize>,
    block_name_to_id: HashMap<String, usize>,

    already_used_labels: HashSet<String>,

    blocks: Vec<BasicBlock>,
    pub load_errors: Vec<String>,
}

impl FunctionBlocksLoader {
    pub fn new(function: Rc<Function>) -> Self {
        FunctionBlocksLoader {
            function,
            cur_id: 0,
            block_id_to_idx: HashMap::new(),
            block_name_to_id: HashMap::new(),
            already_used_labels: HashSet::new(),
            blocks: Vec::new(),
            load_errors: Vec::new(),
        }
    }

    pub fn load(mut self) -> Result<FunctionBlocks, Vec<String>> {
        let mut cur_block_instrs: Vec<Rc<Instruction>> = Vec::new();
        let function = self.function.clone();

        for instr in &function.instrs {
            if instr.is_instr() {
                cur_block_instrs.push(instr.clone());

                if TERMINATOR_INSTS.contains(&instr.get_op_code().unwrap()) {
                    self.add_block(&mut cur_block_instrs);
                }
            } else if instr.is_label() {
                if !cur_block_instrs.is_empty() {
                    self.add_block(&mut cur_block_instrs);
                }

                // the label will go in the beginning of the next basicblock
                cur_block_instrs.push(instr.clone());
            }
        }

        if !cur_block_instrs.is_empty() {
            self.add_block(&mut cur_block_instrs);
        }

        if !self.load_errors.is_empty() {
            return Err(self.load_errors);
        }

        Ok(FunctionBlocks::new(
            &self.function.name,
            self.function.args.clone(),
            self.blocks,
            self.block_id_to_idx,
            self.block_name_to_id,
        ))
    }

    fn add_block(&mut self, cur_block_instrs: &mut Vec<Rc<Instruction>>) {
        let next_idx = self.blocks.len();
        let new_id = self.cur_id;
        self.cur_id += 1;

        self.block_id_to_idx.insert(new_id, next_idx);

        // assign the block's name. if the first elem is a label, then that is it's name
        // otherwise we make one up
        let block_name = if !cur_block_instrs.is_empty() && cur_block_instrs[0].is_label() {
            let block_name = cur_block_instrs[0].get_label().unwrap().to_string();

            if self.already_used_labels.contains(&block_name) {
                self.load_errors.push(format!(
                    "Label {} has been used multiple times",
                    &block_name
                ));
            }

            self.already_used_labels.insert(block_name.clone());

            block_name
        } else {
            // there's no label in this basic block. add one
            let new_block_name = format!("{}{}", BLOCK_NAME_PFX, new_id);

            if self.already_used_labels.contains(&new_block_name) {
                self.load_errors.push(format!(
                    "Label {} has been used multiple times",
                    &new_block_name
                ));
            }

            self.already_used_labels.insert(new_block_name.clone());

            new_block_name
        };

        let newbb = BasicBlock::new(new_id, cur_block_instrs.clone());
        newbb.set_name(&block_name);

        self.block_name_to_id.insert(block_name, new_id);

        self.blocks.push(newbb);
        cur_block_instrs.clear();
    }
}

impl FunctionBlocks {
    pub fn new(
        name: &str,
        args: Vec<Rc<FunctionArg>>,
        blocks: Vec<BasicBlock>,
        block_id_to_idx: HashMap<usize, usize>,
        block_name_to_id: HashMap<String, usize>,
    ) -> Self {
        FunctionBlocks {
            name: name.to_string(),
            args,
            blocks,
            block_id_to_idx,
            block_name_to_id,
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

    pub fn get_block_by_name(&self, name: &str) -> Option<&BasicBlock> {
        let id = self.block_name_to_id.get(name);
        if let None = id {
            return None;
        }

        Some(self.get_block_by_id(*id.unwrap()).unwrap())
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
        if !self.args.is_empty() {
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
        } else {
            write!(f, ") {{\n")?;
        }

        for block in &self.blocks {
            writeln!(f, "#{}", block.name.borrow())?;
            write!(f, "{}", block)?;
        }

        writeln!(f, "}}")?;

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
