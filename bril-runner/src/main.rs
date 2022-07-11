extern crate bril_nw;
extern crate clap;

use bril_nw::{basicblock, bril, cfg};
use std::{fs, path::Path, process};

use clap::{arg, command};

struct CompilerConfig {
    file_name: String,
}

fn main() {
    let cfg = parse_cmd_line();

    let in_file_path = Path::new(cfg.file_name.as_str());
    if !in_file_path.exists() {
        println!(
            "bril-runner: error: Input file {} not found.",
            cfg.file_name
        );

        process::exit(1);
    }

    let contents = fs::read_to_string(in_file_path);
    if let Err(e) = contents {
        println!("Error reading file: {:?}", e);
        process::exit(1);
    }

    let contents = contents.unwrap();

    let loaded_bril = bril::loader::load_bril(&contents);
    drop(contents);

    if let Err(e) = loaded_bril {
        println!("Error occurred parsing BRIL {:?}", e);
        process::exit(1);
    }

    let loaded_bril = loaded_bril.unwrap();

    for func in loaded_bril.functions {
        let bb = basicblock::load_function_blocks(func.clone());
        println!("{:?}", bb);

        let cfg = cfg::ControlFlowGraph::create_from_basic_blocks(bb.get_blocks());
        println!("cfg: {:?}", cfg.successors);
    }
}

fn parse_cmd_line() -> CompilerConfig {
    let m = command!()
        .arg(arg!([NAME] "File to compile").required(true))
        .get_matches();

    let file_name = m.value_of("NAME").unwrap().to_string();

    CompilerConfig { file_name }
}
