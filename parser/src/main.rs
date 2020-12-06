
mod ast;
mod parse;
mod typing;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{Arg, App};

use parse::parse;
use typing::main::static_type;

fn run(file_name: &str, parse_only: bool, type_only: bool) -> Result<(), String> {
    let path = Path::new(file_name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {} : {}", display, why),
        Ok(file) => file,
    };
    
    let mut s = String::new();
    file.read_to_string(&mut s).map_err(|e| e.to_string())?;

    let ast = parse(file_name, &s).map_err(|e| e.to_string())?;
    if !parse_only {
        let typed_decls = static_type(ast).map_err(|e| e.to_string())?;

        println!("{:?}", typed_decls);
    } else {
        println!("{:?}", ast);
    }

    Ok(())
}

fn main() {
    let matches = App::new("petit-julia")
        .version("1.0")
        .author("Julien Marquet")
        .arg(Arg::with_name("input")
            .help("The program to run")
            .required(true)
            .index(1))
        .arg(Arg::with_name("parse-only")
            .long("parse-only")
            .help("Only parse the input"))
        .arg(Arg::with_name("type-only")
            .long("type-only")
            .help("Only types the input"))
        .get_matches();

    let success = {
        let file_name = matches.value_of("input").unwrap();
        let _parse_only = matches.is_present("parse-only");
        let _type_only = matches.is_present("type-only");

        let res = run(file_name, _parse_only, _type_only);

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
