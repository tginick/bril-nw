extern crate bril_nw;
extern crate clap;

use bril_nw::{basicblock, bril, cfg, ssa};
use std::{fs, path::Path, process};

use clap::{arg, command};

struct CompilerConfig {
    file_name: String,
    display_blocks: bool,
    display_cfg: bool,
    convert_to_ssa: bool,
}

fn main() {
    let cmd_line = parse_cmd_line();

    let in_file_path = Path::new(cmd_line.file_name.as_str());
    if !in_file_path.exists() {
        println!(
            "bril-runner: error: Input file {} not found.",
            cmd_line.file_name
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
        let mut bb = basicblock::load_function_blocks(func.clone());

        let mut cfg = cfg::ControlFlowGraph::create_from_basic_blocks(&mut bb);
        if cmd_line.display_cfg {
            println!("// cfg: {}", cfg);
        }

        if cmd_line.convert_to_ssa {
            let dom_tree = cfg.create_dominator_tree(cfg.find_dominators());
            if cmd_line.display_cfg {
                println!("// domtree: {:?}", dom_tree);
            }

            ssa::convert_to_ssa_form(&mut cfg, &dom_tree);
        }

        if cmd_line.display_blocks {
            println!("{}", bb);
        }
    }
}

fn parse_cmd_line() -> CompilerConfig {
    let m = command!()
        .arg(arg!(-b --"blocks" "Display loaded blocks in BRIL notation"))
        .arg(arg!(-g --"graphs" "Display Control Flow Graph and related structures"))
        .arg(arg!(-s --"ssa" "Convert loaded blocks into SSA form before displaying"))
        .arg(arg!([NAME] "File to compile").required(true))
        .get_matches();

    let file_name = m.value_of("NAME").unwrap().to_string();

    CompilerConfig {
        file_name,
        display_blocks: m.is_present("blocks"),
        display_cfg: m.is_present("graphs"),
        convert_to_ssa: m.is_present("ssa"),
    }
}
