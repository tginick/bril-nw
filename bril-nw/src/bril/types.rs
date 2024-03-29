use std::{cell::RefCell, fmt, rc::Rc};

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

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Unit,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OpCode {
    Id,
    Const,
    Add,
    Mul,
    LessThan,
    Print,
    Jump,
    Branch,
    Ret,
    Phi,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstInstruction {
    pub op: OpCode,
    pub dest: String,
    pub instr_type: Type,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValueInstruction {
    pub op: OpCode,
    pub dest: String,
    pub instr_type: Type,
    pub args: Vec<String>,
    pub funcs: Vec<String>,
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]

pub struct EffectInstruction {
    pub op: OpCode,
    pub args: Vec<String>,
    pub funcs: Vec<String>,
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Const(ConstInstruction),
    Value(ValueInstruction),
    Effect(EffectInstruction),
    Label(String),
}

#[derive(Debug)]
pub struct InstructionScaffold(pub Rc<RefCell<Instruction>>);

impl TryFrom<&str> for OpCode {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "id" => Ok(OpCode::Id),
            "const" => Ok(OpCode::Const),
            "add" => Ok(OpCode::Add),
            "mul" => Ok(OpCode::Mul),
            "lt" => Ok(OpCode::LessThan),
            "jmp" => Ok(OpCode::Jump),
            "br" => Ok(OpCode::Branch),
            "ret" => Ok(OpCode::Ret),
            "print" => Ok(OpCode::Print),
            "phi" => Ok(OpCode::Phi),
            _ => Err(()),
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Id => write!(f, "id"),
            OpCode::Const => write!(f, "const"),
            OpCode::Add => write!(f, "add"),
            OpCode::Mul => write!(f, "mul"),
            OpCode::LessThan => write!(f, "lt"),
            OpCode::Jump => write!(f, "jmp"),
            OpCode::Branch => write!(f, "br"),
            OpCode::Ret => write!(f, "ret"),
            OpCode::Print => write!(f, "print"),
            OpCode::Phi => write!(f, "phi"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "int"),
            Type::Unit => write!(f, "()"),
        }
    }
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

impl From<&Rc<Instruction>> for InstructionScaffold {
    fn from(instr: &Rc<Instruction>) -> Self {
        let r = instr.as_ref().clone();
        InstructionScaffold(Rc::new(RefCell::new(r)))
    }
}

impl From<&InstructionScaffold> for Rc<Instruction> {
    fn from(i: &InstructionScaffold) -> Self {
        let instr = (*i.0).borrow().clone();
        Rc::new(instr)
    }
}

impl AsRef<Rc<RefCell<Instruction>>> for InstructionScaffold {
    fn as_ref(&self) -> &Rc<RefCell<Instruction>> {
        &self.0
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Const(c) => writeln!(
                f,
                "    {}: {} = {} {}",
                &c.dest, c.instr_type, c.op, c.value
            ),
            Instruction::Effect(e) => writeln!(
                f,
                "    {} {} {} {}",
                e.op,
                e.args.join(" "),
                e.labels
                    .iter()
                    .map(|s| format!(".{}", s))
                    .collect::<Vec<String>>()
                    .join(" "),
                e.funcs.join(" ")
            ),
            Instruction::Value(v) => writeln!(
                f,
                "    {}: {} = {} {} {} {}",
                &v.dest,
                v.instr_type,
                v.op,
                v.args.join(" "),
                v.labels
                    .iter()
                    .map(|s| format!(".{}", s))
                    .collect::<Vec<String>>()
                    .join(" "),
                v.funcs.join(" ")
            ),
            Instruction::Label(l) => writeln!(f, ".{}:", l),
        }
    }
}

impl Instruction {
    pub fn new_const(op: OpCode, dest: String, instr_type: Type, value: Value) -> Rc<Self> {
        Rc::new(Instruction::Const(ConstInstruction {
            op,
            dest,
            instr_type,
            value,
        }))
    }

    pub fn new_value(
        op: OpCode,
        dest: String,
        instr_type: Type,
        args: Vec<String>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) -> Rc<Self> {
        Rc::new(Instruction::Value(ValueInstruction {
            op,
            dest,
            instr_type,
            args,
            funcs,
            labels,
        }))
    }

    pub fn new_effect(
        op: OpCode,
        args: Vec<String>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) -> Rc<Self> {
        Rc::new(Instruction::Effect(EffectInstruction {
            op,
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
            Instruction::Label(_) => true,
            _ => false,
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            Instruction::Const(_) => true,
            _ => false,
        }
    }

    pub fn is_value(&self) -> bool {
        match self {
            Instruction::Value(_) => true,
            _ => false,
        }
    }

    pub fn is_effect(&self) -> bool {
        match self {
            Instruction::Effect(_) => true,
            _ => false,
        }
    }

    pub fn is_jump(&self) -> bool {
        if !self.is_instr() {
            return false;
        }

        let op = self.get_op_code().unwrap();
        return op == OpCode::Branch || op == OpCode::Jump;
    }

    pub fn is_ret(&self) -> bool {
        if !self.is_instr() {
            return false;
        }

        let op = self.get_op_code().unwrap();
        return op == OpCode::Ret;
    }

    pub fn get_op_code(&self) -> Option<OpCode> {
        match self {
            Instruction::Const(c) => Some(c.op),
            Instruction::Value(v) => Some(v.op),
            Instruction::Effect(e) => Some(e.op),
            _ => None,
        }
    }

    pub fn change_op_code(&mut self, new_op: OpCode) {
        match self {
            Instruction::Const(c) => c.op = new_op,
            Instruction::Value(v) => v.op = new_op,
            Instruction::Effect(e) => e.op = new_op,
            _ => (),
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

    pub fn set_dest(&mut self, new_dest: String) {
        match self {
            Instruction::Const(c) => c.dest = new_dest,
            Instruction::Value(v) => v.dest = new_dest,
            _ => (),
        }
    }

    pub fn get_args_copy(&self) -> Vec<String> {
        match self {
            Instruction::Value(v) => v.args.clone(),
            Instruction::Effect(e) => e.args.clone(),
            _ => vec![],
        }
    }

    pub fn get_args(&self) -> Option<&Vec<String>> {
        match self {
            Instruction::Value(v) => Some(&v.args),
            Instruction::Effect(e) => Some(&e.args),
            _ => None,
        }
    }

    pub fn get_args_mut(&mut self) -> Option<&mut Vec<String>> {
        match self {
            Instruction::Value(v) => Some(&mut v.args),
            Instruction::Effect(e) => Some(&mut e.args),
            _ => None,
        }
    }

    pub fn get_const_value(&self) -> Option<Value> {
        match self {
            Instruction::Const(c) => Some(c.value),
            _ => None,
        }
    }

    pub fn get_type(&self) -> Option<Type> {
        match self {
            Instruction::Const(c) => Some(c.instr_type),
            Instruction::Value(v) => Some(v.instr_type),
            _ => None,
        }
    }

    pub fn get_funcs_copy(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Value(v) => Some(v.funcs.clone()),
            Instruction::Effect(e) => Some(e.funcs.clone()),
            _ => None,
        }
    }

    pub fn get_labels_copy(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Value(v) => Some(v.labels.clone()),
            Instruction::Effect(e) => Some(e.labels.clone()),
            _ => None,
        }
    }

    pub fn get_labels_mut(&mut self) -> Option<&mut Vec<String>> {
        match self {
            Instruction::Value(v) => Some(&mut v.labels),
            Instruction::Effect(e) => Some(&mut e.labels),
            _ => None,
        }
    }
}

fn get_jump_target_from_effect(e: &EffectInstruction) -> Vec<String> {
    e.labels.clone()
}
