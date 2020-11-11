
mod parsing;
mod types;
mod build_graph;
mod sort_graph;
mod runner;

use clap::{Arg, App, SubCommand};

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

fn run(file_name: &str) -> Result<(), String> {
    let s = read_file(file_name)?;
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

fn test(test_name: &str) -> Result<(), String> {
    let netlist_name = &format!("{}.net", test_name);
    let input_name = &format!("{}.in", test_name);
    let output_name = &format!("{}.out", test_name);

    let netlist = read_file(netlist_name)?;
    let input = read_file(input_name)?;
    let output = read_file(output_name)?;

    fn read_bits_sequence(src: &str) -> Result<Vec<Vec<Vec<bool>>>, String> {
        let mut frames = Vec::new();
        let mut lists = Vec::new();
        let mut curr_list = Vec::new();
        let mut empty_line = true;

        src.chars().try_for_each(|c| -> Result<(), String> {
            let mut flush_curr = |curr_list: &mut Vec<_>, lists: &mut Vec<_>| {
                if !empty_line {
                    let mut v = Vec::new();
                    std::mem::swap(curr_list, &mut v);
                    lists.push(v);
                    empty_line = true;
                }
            };
            let flush_lists = |lists: &mut Vec<_>, frames: &mut Vec<_>| {
                let mut v = Vec::new();
                std::mem::swap(lists, &mut v);
                frames.push(v);
            };

            match c {
                '\n' => flush_curr(&mut curr_list, &mut lists),
                '0' => {empty_line = false; curr_list.push(false)},
                '1' => {empty_line = false; curr_list.push(true)},
                ';' => {
                    flush_curr(&mut curr_list, &mut lists);
                    flush_lists(&mut lists, &mut frames);
                },
                _ => Err("Unexpected character in input.".to_string())?
            }

            Ok(())
        })?;

        Ok(frames)
    }

    let input_frames = read_bits_sequence(&input)?;
    let output_frames = read_bits_sequence(&output)?;

    if input_frames.len() != output_frames.len() {
        Err("Mismatched number of sub-stests.")?
    }

    let res = parse_netlist(netlist_name, &netlist);

    let res = match res {
        Ok(res) => res,
        Err(e) => return Err(e.to_string())
    };

    let graph = build_graph(res)?;
    let list = sort_graph(graph)?;

    for (test_id, (input_vecs, output_vecs)) in
        Iterator::zip(input_frames.iter(), output_frames.iter()).enumerate()
    {
        let (mut runner, inputs, outputs) = Runner::new(&list);

        let mut i = 0;

        while i < input_vecs.len() {
            for input in inputs.iter() {
                if i < input_vecs.len() {
                    runner.write(input, &input_vecs[i]);
                    i += 1;
                } else {
                    Err("Missing inputs to complete a cycle.".to_string())?
                }
            }

            runner.tick()?;
        }

        let mut i = 0;
        let mut matches = true;
        let mut got = Vec::new();

        for output in outputs.iter() {
            if i < output_vecs.len() {
                let v = runner.read(output);
                matches = matches && v == output_vecs[i];
                got.push(v);
                i += 1;
            } else {
                Err("Missing output templates.".to_string())?
            }
        }

        if matches {
            println!("    #{} passed.", test_id + 1);
        } else {
            println!("    #{} failed.", test_id + 1);
            
            fn format_bits(v: &[bool]) -> String {
                v.iter().map(|b| if *b {'1'} else {'0'}).collect()
            }

            for i in 0..output_vecs.len() {
                println!(
                    "    expected {}, got {}",
                    format_bits(&output_vecs[i]),
                    format_bits(&got[i])
                )
            }
        }
    } 

    Ok(())
}

fn main() -> Result<(), String> {
    let matches = App::new("sysnum-2020")
        .version("1.0")
        .author("Julien Marquet")
        .subcommand(SubCommand::with_name("run")
            .about("Interactively simulates the given netlist")
            .arg(Arg::with_name("input")
                .help("The netlist to simulate")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("test")
            .about("Runs the given test (see folder tests/)")
            .arg(Arg::with_name("input")
                .help("The netlist to simulate")
                .required(true)
                .index(1)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let file_name = matches.value_of("input").unwrap();
        run(file_name)?;
    } else if let Some(matches) = matches.subcommand_matches("test") {
        let test_name = matches.value_of("input").unwrap();
        test(test_name)?;
    }

    Ok(())
}

