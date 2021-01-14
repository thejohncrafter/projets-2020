
mod hir;
mod lir;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{Arg, App, SubCommand};

use hir::parsing::*;
use lir::parsing::*;

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

fn compile_hir(file_name: &str) -> Result<(), String> {
    let s = read_file(file_name)?;
    let res = parse_hir(file_name, &s);

    let res = match res {
        Ok(res) => res,
        Err(e) => return Err(e.to_string())
    };

    res.iter().for_each(|f| {
            println!("{}", f);
        });

    Ok(())
}

fn compile_lir(file_name: &str) -> Result<(), String> {
    let s = read_file(file_name)?;
    let res = parse_lir(file_name, &s);

    let res = match res {
        Ok(res) => res,
        Err(e) => return Err(e.to_string())
    };

    res.iter().for_each(|f| {
            println!("{}", f);
        });

    Ok(())
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
                .index(1)))
        .subcommand(SubCommand::with_name("lir")
            .about("Compiles a LIR source file")
            .arg(Arg::with_name("input")
                .help("The source file")
                .required(true)
                .index(1)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("hir") {
        let file_name = matches.value_of("input").unwrap();

        let res = compile_hir(file_name);

        match res {
            Ok(()) => (),
            Err(e) => {
                println!("{}", e)
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("lir") {
        let file_name = matches.value_of("input").unwrap();

        let res = compile_lir(file_name);

        match res {
            Ok(()) => (),
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}

