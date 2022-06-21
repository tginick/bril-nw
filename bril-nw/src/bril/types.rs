use std::rc::Rc;

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Rc<Function>>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub args: Vec<Rc<FunctionArg>>,
    pub instrs: Vec<Rc<Instruction>>,
}

#[derive(Debug)]
pub struct FunctionArg {
    pub name: String,
    pub arg_type: Type,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Unit,
}

#[derive(Debug)]
pub enum Value {
    Int(i32),
    Bool(bool),
}

#[derive(Debug)]
pub struct ConstInstruction {
    pub op: String,
    pub dest: String,
    pub instr_type: Type,
    pub value: Value,
}

#[derive(Debug)]
pub struct ValueInstruction {
    pub op: String,
    pub dest: String,
    pub instr_type: Type,
    pub args: Vec<String>,
    pub funcs: Vec<String>,
    pub labels: Vec<String>,
}

#[derive(Debug)]

pub struct EffectInstruction {
    pub op: String,
    pub args: Vec<String>,
    pub funcs: Vec<String>,
    pub labels: Vec<String>,
}

#[derive(Debug)]
pub enum Instruction {
    Const(ConstInstruction),
    Value(ValueInstruction),
    Effect(EffectInstruction),
    Label(String),
}

impl Program {
    pub fn new(functions: Vec<Rc<Function>>) -> Self {
        Program { functions }
    }
}

impl Function {
    pub fn new(
        name: String,
        return_type: Type,
        args: Vec<Rc<FunctionArg>>,
        instrs: Vec<Rc<Instruction>>,
    ) -> Rc<Self> {
        Rc::new(Function {
            name,
            return_type,
            args,
            instrs,
        })
    }
}

impl FunctionArg {
    pub fn new(name: String, arg_type: Type) -> Rc<Self> {
        Rc::new(FunctionArg { name, arg_type })
    }
}

impl Instruction {
    pub fn new_const(op: &str, dest: String, instr_type: Type, value: Value) -> Rc<Self> {
        Rc::new(Instruction::Const(ConstInstruction {
            op: op.to_string(),
            dest,
            instr_type,
            value,
        }))
    }

    pub fn new_value(
        op: &str,
        dest: String,
        instr_type: Type,
        args: Vec<String>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) -> Rc<Self> {
        Rc::new(Instruction::Value(ValueInstruction {
            op: op.to_string(),
            dest,
            instr_type,
            args,
            funcs,
            labels,
        }))
    }

    pub fn new_effect(
        op: &str,
        args: Vec<String>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) -> Rc<Self> {
        Rc::new(Instruction::Effect(EffectInstruction {
            op: op.to_string(),
            args,
            funcs,
            labels,
        }))
    }

    pub fn new_label(label_name: &str) -> Rc<Self> {
        Rc::new(Instruction::Label(label_name.to_string()))
    }

    pub fn is_instr(&self) -> bool {
        match self {
            Instruction::Const(_) => true,
            Instruction::Value(_) => true,
            Instruction::Effect(_) => true,
            Instruction::Label(_) => false,
        }
    }

    pub fn is_label(&self) -> bool {
        match self {
            Instruction::Const(_) => false,
            Instruction::Value(_) => false,
            Instruction::Effect(_) => false,
            Instruction::Label(_) => true,
        }
    }

    pub fn is_jump(&self) -> bool {
        if !self.is_instr() {
            return false;
        }

        let op = self.get_op_code().unwrap();
        return op == "br" || op == "jmp";
    }

    pub fn is_ret(&self) -> bool {
        if !self.is_instr() {
            return false;
        }

        let op = self.get_op_code().unwrap();
        return op == "ret";
    }

    pub fn get_op_code(&self) -> Option<&str> {
        match self {
            Instruction::Const(c) => Some(&c.op),
            Instruction::Value(v) => Some(&v.op),
            Instruction::Effect(e) => Some(&e.op),
            _ => None,
        }
    }

    pub fn get_jump_target(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Effect(e) => Some(get_jump_target_from_effect(e)),
            _ => None,
        }
    }

    pub fn get_label(&self) -> Option<&str> {
        match self {
            Instruction::Label(l) => Some(l),
            _ => None,
        }
    }

    pub fn get_dest(&self) -> Option<&str> {
        match self {
            Instruction::Const(c) => Some(&c.dest),
            Instruction::Value(v) => Some(&v.dest),
            _ => None,
        }
    }

    pub fn get_args_copy(&self) -> Vec<String> {
        match self {
            Instruction::Value(v) => v.args.clone(),
            _ => vec![],
        }
    }

    pub fn get_args(&self) -> Option<&Vec<String>> {
        match self {
            Instruction::Value(v) => Some(&v.args),
            _ => None,
        }
    }
}

fn get_jump_target_from_effect(e: &EffectInstruction) -> Vec<String> {
    e.labels.clone()
}
