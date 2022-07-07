use std::{collections::HashMap, rc::Rc};

use crate::{
    basicblock::BasicBlock,
    bril::types::{Instruction, OpCode},
    opt::LocalOptimizationPass,
};

#[derive(Debug)]
pub enum LVNError {
    UndeclaredArgName,
}

pub struct LocalValueNumbering {
    env: HashMap<String, usize>,
    table: HashMap<LVNCanonicalExpression, usize>,
    names: HashMap<usize, String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct LVNCanonicalExpression {
    op: String,
    args: Vec<usize>,
}

impl LocalOptimizationPass for LocalValueNumbering {
    fn run(&mut self, block: &mut BasicBlock) {
        for instr in &mut block.instrs {
            let canon_instr = self.canonicalize_instruction(instr);
            if let None = canon_instr {
                continue;
            }

            let canon_instr = canon_instr.unwrap();
            let (is_new_entry, ordinal) =
                self.register_canonicalized_instr(instr, canon_instr.clone());

            self.reconstruct_instruction(instr, canon_instr, is_new_entry, ordinal);
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
    ) -> (bool, usize) {
        if !self.table.contains_key(&canon_instr) {
            // new table entry
            let new_ordinal = self.get_current_ordinal();
            self.table.insert(canon_instr, new_ordinal);

            let canonical_name = instr.get_dest().unwrap().to_string();
            self.env.insert(canonical_name.clone(), new_ordinal);
            self.names.insert(new_ordinal, canonical_name);

            (true, new_ordinal)
        } else {
            // already exists. just add to env
            let ordinal = self.table.get(&canon_instr).unwrap();
            self.env
                .insert(instr.get_dest().unwrap().to_string(), *ordinal);

            (false, *ordinal)
        }
    }

    fn canonicalize_instruction(
        &mut self,
        instr: &Rc<Instruction>,
    ) -> Option<LVNCanonicalExpression> {
        if !instr.is_instr() {
            // a label. nothing to canonicalize
            return None;
        }

        if instr.is_const() {
            // if const, add to the table if it doesn't already exist.
            let canon_instr = canonicalize_const_instr(instr);
            return Some(canon_instr);
        } else if instr.is_value() {
            let canon_instr = canonicalize_value_instr(&self.env, instr);
            if let Err(_) = canon_instr {
                // failed to canonicalize an instr. bail
                return None;
            }

            return Some(canon_instr.unwrap());
        }

        return None;
    }

    fn reconstruct_instruction(
        &self,
        instr: &mut Rc<Instruction>,
        canon_instr: LVNCanonicalExpression,
        is_new_entry: bool,
        ordinal: usize,
    ) {
        if !instr.is_value() {
            return; // only things we need to reconstruct are value instrs
        }

        if !is_new_entry {
            // rewrite instruction to an id
            let existing_canonical_name = self.names.get(&ordinal);
            if let None = existing_canonical_name {
                // TODO: bad
                return;
            }

            let existing_canonical_name = existing_canonical_name.unwrap();
            let new_instr = Instruction::new_value(
                OpCode::Id,
                instr.get_dest().unwrap().to_string(),
                instr.get_type().unwrap(),
                vec![existing_canonical_name.clone()],
                vec![],
                vec![],
            );

            *instr = new_instr;
        } else {
            // new computed value. don't change op code but rewrite args
            let updated_args: Vec<String> = canon_instr
                .args
                .iter()
                .map(|arg_ordinal| {
                    let existing_canonical_name = self.names.get(&arg_ordinal).unwrap();

                    existing_canonical_name.clone()
                })
                .collect();

            let new_instr = Instruction::new_value(
                instr.get_op_code().unwrap(),
                instr.get_dest().unwrap().to_string(),
                instr.get_type().unwrap(),
                updated_args,
                instr.get_funcs_copy().unwrap(),
                instr.get_labels_copy().unwrap(),
            );

            *instr = new_instr;
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
