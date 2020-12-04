
mod ast;
mod parse;
mod typing;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{Arg, App};

use parse::parse;
use typing::static_type;

fn run(file_name: &str, parse_only: bool, type_only: bool) -> Result<(), String> {
    let path = Path::new(file_name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {} : {}", display, why),
        Ok(file) => file,
    };
    
    let mut s = String::new();
    file.read_to_string(&mut s).map_err(|e| e.to_string())?;

    println!("Parsing...");
    let mut ast = parse(file_name, &s).map_err(|e| e.to_string())?;
    if !parse_only {
        println!("Typing...");
        ast = static_type(ast).map_err(|e| e.to_string())?;
    }

    println!("{:?}", ast);

    Ok(())
}

fn test(dir_name: &str) -> Result<(), String> {
    /*
     * Returns true if all the tests pass.
     */
    fn test<'a, I: Iterator<Item = std::path::PathBuf>>(tests: I, expect_success: bool)
        -> Result<bool, String>
    {
        let mut failed = false;

        for path in tests {
            let display = path.display();
            let mut file = match File::open(&path) {
                Err(why) => panic!("Couldn't open {} : {}", display, why),
                Ok(file) => file,
            };

            let mut s = String::new();
            file.read_to_string(&mut s).map_err(|e| e.to_string())?;

            let file_name = path.file_name().and_then(|n| n.to_str()).map(|n| n.to_string()).unwrap_or("test".to_string());
            match parse(&file_name, &s) {
                Ok(_) => {
                    if expect_success {
                        println!("    \u{2713} {}", file_name);
                    } else {
                        failed = true;
                        println!("    \u{2717} {} : expected a failure, got a success", file_name);
                    }
                },
                Err(_) => {
                    if expect_success {
                        failed = true;
                        println!("    \u{2717} {} : expected a success, got a failure", file_name);
                    } else {
                        // "Task failed successfully."
                        println!("    \u{2713} {}", file_name);
                    }
                }
            }
        }

        Ok(failed)
    }
 
    fn get_paths(path: &Path) -> Result<Vec<std::path::PathBuf>, String> {
        let mut members = Vec::new();

        fs::read_dir(&path)
            .map_err(|e| e.to_string())?
            .map(|e| e.map_err(|e| e.to_string()).map(|e| e.path()))
            .try_for_each(|e| -> Result<(), String> {members.push(e?); Ok(())})?;

        members.sort();
        Ok(members)
    }

    let good_name = format!("{}/good", dir_name);
    let good_path = Path::new(&good_name);
    let bad_name = format!("{}/bad", dir_name);
    let bad_path = Path::new(&bad_name);

    let good_tests = get_paths(good_path)?;

    let bad_tests = get_paths(bad_path)?;
    
    let mut failed = false;
    println!("Testing \"good\" inputs :");
    failed = failed || test(good_tests.into_iter(), true)?;
    println!("Testing \"bad\" inputs :");
    failed = failed || test(bad_tests.into_iter(), false)?;

    if failed {
        println!("*** FAILED ***");
    } else {
        println!("*** SUCCESS ***");
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
        .arg(Arg::with_name("test")
            .long("test")
            .help("Runs all the tests in the given directory, overrides the default behavior"))
        .arg(Arg::with_name("parse-only")
            .long("parse-only")
            .help("Only parse the input"))
        .arg(Arg::with_name("type-only")
            .long("type-only")
            .help("Only types the input"))
        .get_matches();

    let success = if !matches.is_present("test") {
        let file_name = matches.value_of("input").unwrap();
        let _parse_only = matches.is_present("parse_only");
        let _type_only = matches.is_present("type_only");

        let res = run(file_name, _parse_only, _type_only);

        match res {
            Ok(()) => true,
            Err(e) => {
                println!("{}", e);
                false
            }
        }
    } else {
        let dir_name = matches.value_of("input").unwrap();
        
        let res = test(dir_name);
        
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

