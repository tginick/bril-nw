use std::{collections::HashSet, rc::Rc};

use json::JsonValue;

use super::types::{Function, FunctionArg, Instruction, OpCode, Program, Type, Value};

lazy_static! {
    static ref VALUE_INSTS: HashSet<OpCode> =
        HashSet::from([OpCode::Id, OpCode::Add, OpCode::Mul, OpCode::Phi]);
    static ref EFFECT_INSTS: HashSet<OpCode> =
        HashSet::from([OpCode::Print, OpCode::Ret, OpCode::Branch, OpCode::Jump]);
    static ref CONST_INSTS: HashSet<OpCode> = HashSet::from([OpCode::Const]);
}

#[derive(Debug)]
pub enum BrilLoadError {
    JSONParse,
    InvalidFunctionsBlock,
    FunctionInvalidName,
    FunctionInvalidArgs,
    InvalidTypeString,
    FunctionInvalidInstrs,
    FunctionArgInvalidSpec,
    UnrecognizedInstr(String),
    MalformedInstr,
    TypeMismatch,
    NotAStringArray,
    Unimplemented,
}

pub fn load_bril(loaded_str: &str) -> Result<Program, BrilLoadError> {
    let parsed = json::parse(loaded_str).map_err(|_e| BrilLoadError::JSONParse)?;
    Ok(load_bril_from_obj(parsed)?)
}

fn load_bril_from_obj(obj: JsonValue) -> Result<Program, BrilLoadError> {
    let functions = &obj["functions"];
    if functions.is_null() || !functions.is_array() {
        return Err(BrilLoadError::InvalidFunctionsBlock);
    }

    let mut loaded_functions: Vec<Rc<Function>> = Vec::new();

    for i in 0..functions.len() {
        loaded_functions.push(load_bril_function(&functions[i])?);
    }

    Ok(Program::new(loaded_functions))
}

fn load_bril_function(fn_obj: &JsonValue) -> Result<Rc<Function>, BrilLoadError> {
    let name = &fn_obj["name"];
    let args = &fn_obj["args"];
    let return_type_str = &fn_obj["type"];
    let instrs = &fn_obj["instrs"];

    if name.is_null() || !name.is_string() {
        return Err(BrilLoadError::FunctionInvalidName);
    }

    if !args.is_array() && !args.is_null() {
        return Err(BrilLoadError::FunctionInvalidArgs);
    }

    let mut loaded_args: Vec<Rc<FunctionArg>> = Vec::new();
    for i in 0..args.len() {
        loaded_args.push(load_bril_function_arg(&args[i])?);
    }

    let return_type = load_bril_type(return_type_str)?;

    if !instrs.is_array() {
        return Err(BrilLoadError::FunctionInvalidInstrs);
    }

    let mut loaded_instrs: Vec<Rc<Instruction>> = Vec::new();
    for i in 0..instrs.len() {
        loaded_instrs.push(load_bril_instr(&instrs[i])?);
    }

    Ok(Function::new(
        name.as_str().unwrap().to_string(),
        return_type,
        loaded_args,
        loaded_instrs,
    ))
}

fn load_bril_type(type_v: &JsonValue) -> Result<Type, BrilLoadError> {
    if type_v.is_null() {
        return Ok(Type::Unit);
    }

    if !type_v.is_string() {
        return Err(BrilLoadError::InvalidTypeString);
    }

    let type_v_str = type_v.as_str().unwrap();
    match type_v_str {
        "int" => Ok(Type::Int),
        "bool" => Ok(Type::Bool),
        _ => Err(BrilLoadError::InvalidTypeString),
    }
}

fn load_bril_function_arg(arg_v: &JsonValue) -> Result<Rc<FunctionArg>, BrilLoadError> {
    let name = &arg_v["name"];
    let arg_type = &arg_v["type"];

    if name.is_null() || arg_type.is_null() {
        return Err(BrilLoadError::FunctionArgInvalidSpec);
    }

    let loaded_arg_type = load_bril_type(arg_type)?;

    Ok(FunctionArg::new(
        name.as_str().unwrap().to_string(),
        loaded_arg_type,
    ))
}

fn load_bril_instr(instr_v: &JsonValue) -> Result<Rc<Instruction>, BrilLoadError> {
    let maybe_label = &instr_v["label"];
    if maybe_label.is_string() {
        return Ok(Instruction::new_label(maybe_label.as_str().unwrap()));
    }

    let op = &instr_v["op"];

    if !op.is_string() {
        return Err(BrilLoadError::MalformedInstr);
    }

    let op_str = op.as_str().unwrap();

    let real_op: Result<OpCode, ()> = op_str.try_into();
    if let Err(_) = real_op {
        return Err(BrilLoadError::MalformedInstr);
    }

    let real_op = real_op.unwrap();

    return if CONST_INSTS.contains(&real_op) {
        load_bril_const_instr(real_op, instr_v)
    } else if EFFECT_INSTS.contains(&real_op) {
        load_bril_effect_instr(real_op, instr_v)
    } else if VALUE_INSTS.contains(&real_op) {
        load_bril_value_instr(real_op, instr_v)
    } else {
        Err(BrilLoadError::UnrecognizedInstr(op_str.to_string()))
    };
}

fn load_bril_const_instr(
    op: OpCode,
    instr_v: &JsonValue,
) -> Result<Rc<Instruction>, BrilLoadError> {
    let dest = &instr_v["dest"];
    let instr_type_str = &instr_v["type"];
    let value = &instr_v["value"];

    if !dest.is_string() {
        return Err(BrilLoadError::MalformedInstr);
    }

    let dest_str = dest.as_str().unwrap().to_string();
    let instr_type = load_bril_type(instr_type_str)?;
    let loaded_value = load_bril_value(value, instr_type)?;

    Ok(Instruction::new_const(
        op,
        dest_str,
        instr_type,
        loaded_value,
    ))
}

fn load_bril_value_instr(
    op: OpCode,
    instr_v: &JsonValue,
) -> Result<Rc<Instruction>, BrilLoadError> {
    let dest = &instr_v["dest"];
    let instr_type_str = &instr_v["type"];
    let args = &instr_v["args"];
    let funcs = &instr_v["funcs"];
    let labels = &instr_v["labels"];

    if !dest.is_string() {
        return Err(BrilLoadError::MalformedInstr);
    }

    let dest_str = dest.as_str().unwrap().to_string();
    let instr_type = load_bril_type(instr_type_str)?;

    Ok(Instruction::new_value(
        op,
        dest_str,
        instr_type,
        load_string_array(args)?,
        load_string_array(funcs)?,
        load_string_array(labels)?,
    ))
}

fn load_bril_effect_instr(
    op: OpCode,
    instr_v: &JsonValue,
) -> Result<Rc<Instruction>, BrilLoadError> {
    let args = &instr_v["args"];
    let funcs = &instr_v["funcs"];
    let labels = &instr_v["labels"];

    Ok(Instruction::new_effect(
        op,
        load_string_array(args)?,
        load_string_array(funcs)?,
        load_string_array(labels)?,
    ))
}

fn load_string_array(arr: &JsonValue) -> Result<Vec<String>, BrilLoadError> {
    if arr.is_null() {
        return Ok(Vec::new());
    }

    if !arr.is_array() {
        return Err(BrilLoadError::NotAStringArray);
    }

    let mut loaded_strs: Vec<String> = Vec::with_capacity(arr.len());
    for i in 0..arr.len() {
        let s = &arr[i];

        if !s.is_string() {
            return Err(BrilLoadError::NotAStringArray);
        }

        loaded_strs.push(s.as_str().unwrap().to_string());
    }

    Ok(loaded_strs)
}

fn load_bril_value(value_v: &JsonValue, expected_type: Type) -> Result<Value, BrilLoadError> {
    if expected_type == Type::Int {
        if !value_v.is_number() {
            return Err(BrilLoadError::TypeMismatch);
        }

        return Ok(Value::Int(value_v.as_i32().unwrap()));
    } else if expected_type == Type::Bool {
        if !value_v.is_boolean() {
            return Err(BrilLoadError::TypeMismatch);
        }

        return Ok(Value::Bool(value_v.as_bool().unwrap()));
    }

    Err(BrilLoadError::Unimplemented)
}
