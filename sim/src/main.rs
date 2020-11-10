
mod parsing;
mod types;
mod build_graph;
mod sort_graph;
mod runner;

use clap::{Arg, App};

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use parsing::*;
use build_graph::build_graph;
use sort_graph::sort_graph;
use runner::Runner;

/*
 * The vertices are the variable names.
 *
 * First pass :
 *   * give each variable an address,
 *     taking array lengths into account;
 *     build a map from names to addresses
 *     and vector lengths;
 *   * also give each constant an address.
 * 
 * Second pass :
 *   * check there is no definition conflict
 *     (inputs _count_ as definitions);
 *   * build the edges : a variable is
 *     linked to the variables it depends
 *     on at a given tick
 *     (-> memory definitions don't count);
 *   * build a list of all the memory
 *     operators, remember their inputs
 *     and outputs as well as their types
 *     (-> reg, ram, rom, _and_ inputs);
 *   * make sure everything is well-typed.
 *
 * Then perform a topological sort on the graph.
 *
 * To simulate the circuit :
 *   * perform each operation;
 *   * rewrite memory based on the memory operators.
 */

fn request_input(name: &str, len: u32) -> Vec<bool> {
    if len == 1 {
        print!("    {} (1 bit): ", name);
    } else {
        print!("    {} ({} bits): ", name, len);
    }

    loop {
        io::stdout().flush().expect("IO error.");
        
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("    Failed to read line.");
        let input = input.trim();

        if input.chars().all(|c| c == '0' || c == '1') {
            if input.len() != len as usize {
                if len == 1 {
                    print!("    Please enter 1 bit : ");
                } else {
                    print!("    Please enter {} bits : ", len); 
                }
            } else {
                break input.chars().map(|c| c == '1').collect()
            }
        } else {
            if len == 1 {
                print!("    Please enter \"0\" or \"1\" : ");
            } else {
                print!("    Please enter a sequence of \"0\"s and \"1\"s : ")
            }
        }
    }
}

fn main() -> Result<(), String> {
    let app = App::new("sysnum-2020")
        .version("1.0")
        .author("Julien Marquet")
        .arg(Arg::with_name("input")
            .help("The netlist to simulate.")
            .required(true)
            .index(1))
        .get_matches();

    let file_name = app.value_of("input").unwrap();
    let path = Path::new(file_name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {} : {}", display, why),
        Ok(file) => file,
    };
    
    let mut s = String::new();
    file.read_to_string(&mut s).map_err(|e| e.to_string())?;

    let res = parse_netlist(file_name, &s);

    let res = match res {
        Ok(res) => res,
        Err(e) => return Err(e.to_string())
    };

    let graph = build_graph(res)?;
    let list = sort_graph(graph)?;

    let (mut runner, inputs, outputs) = Runner::new(&list);
   
    for i in 0.. {
        println!();
        println!("Tick #{}", i);

        if outputs.len() == 0 {
            println!("No outputs.");
        } else if outputs.len() == 1 {
            println!("Output :");
        } else {
            println!("Outputs :");
        }

        outputs.iter().for_each(|output| {
            let val = runner.read(output);
            let formatted = val.iter().map(|b| if *b {'1'} else {'0'}).collect::<String>();
            println!("    {}: {}", output.name(), formatted)
        });

        if inputs.len() == 1 {
            println!("Input :");
        } else if inputs.len() != 0 {
            println!("Inputs :");
        }

        inputs.iter().for_each(|input| {
            let val = request_input(input.name(), input.len());
            runner.write(input, &val);
        });

        if inputs.len() == 0 {
            print!("No inputs, press enter to tick. ");
            io::stdout().flush().expect("IO error.");
            io::stdin().lock().read_until('\n' as u8, &mut vec!()).expect("IO error");
        }

        runner.tick()?;
    }

    Ok(())
}

