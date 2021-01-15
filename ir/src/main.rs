
mod hir;
mod lir;
mod error;
mod hir_to_lir;
mod lir_to_asm;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{Arg, App, SubCommand};

use hir::parsing::*;
use lir::parsing::*;
use hir_to_lir::*;
use lir_to_asm::*;

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

fn compile_hir(file_name: &str) -> Result<String, String> {
    let s = read_file(file_name)?;
    let res = parse_hir(file_name, &s);

    let res = match res {
        Ok(res) => res,
        Err(e) => return Err(e.to_string())
    };

    println!("** HIR **");
    res.iter().for_each(|d| {
            println!("{}", d);
        });

    println!();
    println!("** LIR **");
    let compiled = hir_to_lir(&res).map_err(|e| format!("{}", e))?;
    compiled.iter().for_each(|f| println!("{}", f));
 
    println!();
    println!("** asm **");   
    let asm = lir_to_asm(&compiled).map_err(|e| format!("{}", e))?;
    println!("{}", asm);

    Ok(asm)
}

fn compile_lir(file_name: &str) -> Result<String, String> {
    let s = read_file(file_name)?;
    let res = parse_lir(file_name, &s);

    let res = match res {
        Ok(res) => res,
        Err(e) => return Err(e.to_string())
    };

    println!("** LIR **");
    res.iter().for_each(|f| {
            println!("{}", f);
        });

    println!();
    println!("** asm **");

    let compiled = lir_to_asm(&res).map_err(|e| format!("{}", e))?;
    println!("{}", compiled);

    Ok(compiled)
}

fn main() {
    let matches = App::new("petit-julia-ir")
        .version("1.0")
        .author("Julien Marquet, Ryan Lahfa")
        .subcommand(SubCommand::with_name("hir")
            .about("Compiles a HIR source file")
            .arg(Arg::with_name("input")
                .help("The source file")
                .required(true)
                .index(1))
            .arg(Arg::with_name("output")
                .short("o")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("lir")
            .about("Compiles a LIR source file")
            .arg(Arg::with_name("input")
                .help("The source file")
                .required(true)
                .index(1))
            .arg(Arg::with_name("output")
                .short("o")
                .takes_value(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("hir") {
        let file_name = matches.value_of("input").unwrap();

        let res = compile_hir(file_name);

        match res {
            Ok(asm) => {
                if let Some(file_name) = matches.value_of("output") {
                    match write_file(file_name, &asm) {
                        Ok(()) => (),
                        Err(e) => println!("{}", e)
                    }
                }
            },
            Err(e) => {
                println!("{}", e)
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("lir") {
        let file_name = matches.value_of("input").unwrap();

        let res = compile_lir(file_name);

        match res {
            Ok(asm) => {
                if let Some(file_name) = matches.value_of("output") {
                    match write_file(file_name, &asm) {
                        Ok(()) => (),
                        Err(e) => println!("{}", e)
                    }
                }
            },
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}

