use std::fs::File;
use std::fs::read_dir;
use std::io::prelude::*;
use std::path::Path;
use std::env;

use std::process::{Command, Stdio};

use clap::{Arg, App};

use ir::ast_to_hir::typed_ast_to_hir;
use ir::hir_to_lir::hir_to_lir;
use ir::lir_to_asm::lir_to_asm;
use parser::parse_and_type_file;

fn read_file(name: &str) -> Result<String, String> {
    let path = Path::new(name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open for read {} : {}", display, why),
        Ok(file) => file,
    };
    
    let mut s = String::new();
    file.read_to_string(&mut s).map_err(|e| e.to_string())?;

    Ok(s)
}

fn write_file(name: &str, contents: &str) -> Result<(), String> {
    let path = Path::new(name);
    let display = path.display();
    let cwd = env::current_dir().map_err(|err| err.to_string())?;
    let cwd_display = cwd.display();

    let mut file = match File::create(&path) {
        Err(why) => Err(format!("Couldn't open for write {} : {} (cwd: {})", display, why, cwd_display)),
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

fn compile(input: &str,
    output: &str,
    _parse_only: bool,
    _type_only: bool,
    asm_only: bool, 
    debug_hir: bool, debug_lir: bool,
    runtime_object_filename: &str) -> Result<(), String> {
    let asm = compile_to_asm(input, output, debug_hir, debug_lir)?;
    let asm_filename = format!("{}.s", output);
    write_file(&asm_filename, &asm)?;

    if !asm_only {
        // run `as` on asm file to provide object file.
        // use runtime object file.
        // compile test to binary.

        let user_object_filename = format!("{}.o", output);

        let assembling = Command::new("as")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg(asm_filename)
            .arg("-o")
            .arg(user_object_filename.clone())
            .output()
            .expect("Failed to transform asm into object file!");

        if !assembling.status.success() {
            return Err("Fatal error: Failed to transform asm into object file!".into());
        }

        let mixing = Command::new("gcc")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg("-no-pie")
            .arg("-lm")
            .arg(user_object_filename)
            .arg(runtime_object_filename)
            .arg("-o")
            .arg(output)
            .output()
            .expect("Fatal error: Failed to transform object files into ELF binaries!");

        if !mixing.status.success() {
            return Err("Failed to transform object files into ELF binaires!".into());
        }
    }

    Ok(())
}

fn remove_suffix<'a>(s: &'a str, p: &str) -> &'a str {
    if s.ends_with(p) {
        &s[..s.len() - p.len()]
    } else {
        s
    }
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
            .long("parse-only")
            .help("Only parse the input as an AST"))
        .arg(Arg::with_name("type-only")
            .short("t")
            .long("type-only")
            .help("Parse the input and type it"))
        .arg(Arg::with_name("asm-only")
            .short("a")
            .long("asm-only")
            .help("Parse the input, type it and output the ASM by traversing all IR"))
        .arg(Arg::with_name("runtime-object-filename")
            .short("r")
            .long("runtime-object-filename")
            .help("Pass a runtime object filename for the native pJulia API")
            .takes_value(true))
        .arg(Arg::with_name("debug-hir")
            .short("h")
            .long("debug-hir")
            .help("Create a file based on the output filename with the HIR representation"))
        .arg(Arg::with_name("debug-lir")
            .short("l")
            .long("debug-lir")
            .help("Create a file based on the output filename with the LIR representation"))
        .get_matches();

    let success = {
        let input_filename = matches.value_of("input").unwrap();
        let output_filename = matches.value_of("output").unwrap_or(remove_suffix(input_filename, ".jl"));
        let runtime_object_filename = matches.value_of("runtime-object-filename").unwrap_or("runtime.o");

        let parse_only = matches.is_present("parse-only");
        let type_only = matches.is_present("type-only");
        let asm_only = matches.is_present("asm-only");
        let debug_hir = matches.is_present("debug-hir");
        let debug_lir = matches.is_present("debug_lir");

        let res = compile(input_filename, output_filename,
            parse_only,
            type_only,
            asm_only,
            debug_hir,
            debug_lir,
            runtime_object_filename);

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
