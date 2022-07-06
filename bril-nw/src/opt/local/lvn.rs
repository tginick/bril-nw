use std::{collections::HashMap, rc::Rc};

use crate::{basicblock::BasicBlock, bril::types::Instruction, opt::LocalOptimizationPass};

#[derive(Debug)]
pub enum LVNError {
    UndeclaredArgName,
}

pub struct LocalValueNumbering {
    env: HashMap<String, usize>,
    table: HashMap<LVNCanonicalExpression, usize>,
    names: HashMap<usize, String>,
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct LVNCanonicalExpression {
    op: String,
    args: Vec<usize>,
}

impl LocalOptimizationPass for LocalValueNumbering {
    fn run(&mut self, block: &mut BasicBlock) {
        for instr in &mut block.instrs {
            self.canonicalize_instruction(instr);
            self.reconstruct_instruction(instr);
        }
    }
}

impl LocalValueNumbering {
    fn get_current_ordinal(&self) -> usize {
        self.table.len()
    }

    fn register_canonicalized_instr(
        &mut self,
        instr: &Rc<Instruction>,
        canon_instr: LVNCanonicalExpression,
    ) {
        if !self.table.contains_key(&canon_instr) {
            // new table entry
            let new_ordinal = self.get_current_ordinal();
            self.table.insert(canon_instr, new_ordinal);

            let canonical_name = instr.get_dest().unwrap().to_string();
            self.env.insert(canonical_name.clone(), new_ordinal);
            self.names.insert(new_ordinal, canonical_name);
        } else {
            // already exists. just add to env
            let ordinal = self.table.get(&canon_instr).unwrap();
            self.env
                .insert(instr.get_dest().unwrap().to_string(), *ordinal);
        }
    }

    fn canonicalize_instruction(&mut self, instr: &Rc<Instruction>) {
        if !instr.is_instr() {
            // a label. nothing to canonicalize
            return;
        }

        if instr.is_const() {
            // if const, add to the table if it doesn't already exist.
            let canon_instr = canonicalize_const_instr(instr);
            self.register_canonicalized_instr(instr, canon_instr);
        } else if instr.is_value() {
            let canon_instr = canonicalize_value_instr(&self.env, instr);
            if let Err(_) = canon_instr {
                // failed to canonicalize an instr. bail
                return;
            }

            self.register_canonicalized_instr(instr, canon_instr.unwrap());
        }
    }

    fn reconstruct_instruction(&self, instr: &mut Rc<Instruction>) {
        if !instr.is_value() {
            return; // only things we need to reconstruct are value instrs
        }

        
    }
}

fn canonicalize_const_instr(instr: &Rc<Instruction>) -> LVNCanonicalExpression {
    LVNCanonicalExpression {
        op: format!("const_{}", instr.get_const_value().unwrap()),
        args: vec![],
    }
}

fn canonicalize_value_instr(
    env: &HashMap<String, usize>,
    instr: &Rc<Instruction>,
) -> Result<LVNCanonicalExpression, LVNError> {
    let mut arg_ordinals: Vec<usize> = Vec::with_capacity(instr.get_args().unwrap().len());
    for arg in instr.get_args().unwrap() {
        if !env.contains_key(arg) {
            // TODO: bad
            return Err(LVNError::UndeclaredArgName);
        }

        let ordinal = env.get(arg).unwrap();
        arg_ordinals.push(*ordinal);
    }

    Ok(LVNCanonicalExpression {
        op: instr.get_op_code().unwrap().to_string(),
        args: arg_ordinals,
    })
}
