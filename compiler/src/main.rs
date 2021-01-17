use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{Arg, App};

use ir::ast_to_hir::typed_ast_to_hir;
use ir::hir_to_lir::hir_to_lir;
use ir::lir_to_asm::lir_to_asm;
use parser::parse_and_type_file;

fn read_file(name: &str) -> Result<String, String> {
    let path = Path::new(name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {} : {}", display, why),
        Ok(file) => file,
    };
    
    let mut s = String::new();
    file.read_to_string(&mut s).map_err(|e| e.to_string())?;

    Ok(s)
}

fn write_file(name: &str, contents: &str) -> Result<(), String> {
    let path = Path::new(name);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => Err(format!("Couldn't open {} : {}", display, why)),
        Ok(file) => Ok(file),
    }?;

    match file.write_fmt(format_args!("{}", contents)) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    }?;

    Ok(())
}

fn compile_to_asm(file_name: &str, output: &str, debug_hir: bool, debug_lir: bool) -> Result<String, String> {
    let s = read_file(file_name)?;

    let res = match parse_and_type_file(file_name, &s) {
        Ok(res) => res,
        Err(e) => return Err(e.to_string())
    };

    let hir_repr = typed_ast_to_hir(res).map_err(|e| format!("{}", e))?;
    if debug_hir {
        write_file(&format!("{}.hir", output), &hir_repr.to_string())?;
    }
    let lir_repr = hir_to_lir(&hir_repr).map_err(|e| format!("{}", e))?;
    if debug_lir {
        write_file(&format!("{}.lir", output), &lir_repr.to_string())?;
    }
    let asm_repr = lir_to_asm(&lir_repr).map_err(|e| format!("{}", e))?;
    
    Ok(asm_repr)
}

fn compile(input: &str, output: &str, _parse_only: bool, _type_only: bool, debug_hir: bool, debug_lir: bool) -> Result<(), String> {
    compile_to_asm(input, output, debug_hir, debug_lir).and_then(|asm| write_file(&format!("{}.asm", output), &asm))
}

fn main() {
    let matches = App::new("pjulia")
        .version("1.0")
        .author("Julien Marquet, Ryan Lahfa")
        .arg(Arg::with_name("input")
            .help("The source file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("output")
            .short("o")
            .takes_value(true))
        .arg(Arg::with_name("parse-only")
            .short("p")
            .help("Only parse the input as an AST"))
        .arg(Arg::with_name("type-only")
            .short("t")
            .help("Parse the input and type it"))
        .arg(Arg::with_name("debug-hir")
            .short("h")
            .help("Create a file based on the output filename with the HIR representation"))
        .arg(Arg::with_name("debug-lir")
            .short("l")
            .help("Create a file based on the output filename with the LIR representation"))
        .get_matches();

    let success = {
        let input_filename = matches.value_of("input").unwrap();
        let output_filename = matches.value_of("output").unwrap_or("a");

        let parse_only = matches.is_present("parse-only");
        let type_only = matches.is_present("type-only");
        let debug_hir = matches.is_present("debug-hir");
        let debug_lir = matches.is_present("debug_lir");

        let res = compile(input_filename, output_filename,
            parse_only,
            type_only,
            debug_hir,
            debug_lir);

        match res {
            Ok(()) => true,
            Err(e) => {
                println!("{}", e);
                false
            }
        }
    };

    std::process::exit(if success {0} else {1});
}
