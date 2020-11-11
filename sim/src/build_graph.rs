
use std::collections::HashMap;

use super::parsing::*;
use super::types::*;

pub fn build_graph(netlist: Netlist) -> Result<OpsGraph, String> {
    // Maps the variable names to :
    //  * their identifier
    //  * their address
    //  * their length
    let mut addresses: HashMap<&str, (usize, usize, u32)> = HashMap::new();
    let mut mem_size: usize = 0;

    netlist.vars.iter().enumerate().try_for_each(|(id, (name, ty))| {
        let len = match ty {
            ValueType::Bit => 1,
            ValueType::BitArray(k) => *k
                // We checked during parsing that k != 0.
        };
        
        let prev = addresses.insert(name, (id, mem_size, len));
        mem_size += len as usize;

        if prev.is_some() {
            Err(format!("Variable \"{}\" already declared.", name))
        } else {
            Ok(())
        }
    })?;

    let mut edges = netlist.vars.iter()
        .map(|(name, _)| {
            let address = addresses.get(name.as_str()).unwrap().1;
            PartialVarInfo::new(name.clone(), address)
        })
        .collect::<Vec<_>>();

    let mut mem_ops = Vec::new();
    let mut mems = Vec::new();

    let find_var = |name: &str| -> Result<(usize, usize, u32), String> {
        match addresses.get(name) {
            Some(k) => Ok(*k),
            None => Err(format!("Can't find variable {}.", name))
        }
    };

    let find_arg = |arg: &Arg| -> Result<(u32, Option<usize>, OpArg), String> {
        // Returns the length, the id (for variables) and the OpArg.
        Ok(match arg {
            Arg::Const(Value::Bit(b)) => (1, None, OpArg::Cst(vec!(*b))),
            Arg::Const(Value::BitArray(a)) => (a.len() as u32, None, OpArg::Cst(a.clone())),
            Arg::Var(name) => {
                let var = find_var(&name)?;
                (var.2, Some(var.0), OpArg::Var(var.1))
            },
        })
    };

    netlist.inputs.iter().try_for_each(|name| -> Result<(), String> {
        let (id, _, len) = find_var(name)?;
        let info = &mut edges[id];
        
        info.check_empty()?;
        info.def = Some(VarDef::Input);
        info.deps = Some(vec!());
        info.len = Some(len);

        Ok(())
    })?;

    netlist.outputs.iter().try_for_each(|name| -> Result<(), String> {
        find_var(name)?;
        Ok(())
    })?;

    let inputs = netlist.inputs.into_iter().map(|name| {
            let (_, address, len) = find_var(&name).unwrap();
            (len, address, name)
        }).collect();
    let outputs = netlist.outputs.into_iter().map(|name| {
            let (_, address, len) = find_var(&name).unwrap();
            (len, address, name)
        }).collect();

    fn extract_deps(ids: &[&Option<usize>]) -> Vec<usize> {
        ids.iter()
            .filter_map(|id| **id)
            .collect()
    }

    netlist.defs.iter().try_for_each(|(name, def)| -> Result<(), String> {
        let (id, address, len) = find_var(name)?;
        let info = &mut edges[id];
        info.check_empty()?;

        match def {
            Def::Fwd(arg) => {
                let (arg_len, arg_id, arg_arg) = find_arg(arg)?;

                if !(len == arg_len) {
                    return Err("Mismatched lengths.".to_string())
                }

                let def = VarDef::Fwd(len, arg_arg.clone());

                info.def = Some(def);
                info.deps = Some(extract_deps(&[&arg_id]));
                info.len = Some(len);
            },
            Def::Not(arg) => {
                let (arg_len, arg_id, arg_arg) = find_arg(arg)?;

                if !(len == arg_len) {
                    return Err("Mismatched lengths.".to_string())
                }

                let def = VarDef::Not(len, arg_arg.clone());

                info.def = Some(def);
                info.deps = Some(extract_deps(&[&arg_id]));
                info.len = Some(len);
            },
            Def::Bin(op, l, r) => {
                let (l_len, l_id, l_arg) = find_arg(l)?;
                let (r_len, r_id, r_arg) = find_arg(r)?;

                if !(len == l_len && len == r_len) {
                    return Err("Mismatched lengths.".to_string())
                }

                let op = match op {
                    BinOpType::Or => BinOp::Or,
                    BinOpType::Xor => BinOp::Xor,
                    BinOpType::And => BinOp::And,
                    BinOpType::Nand => BinOp::Nand,
                };

                let def = VarDef::BinOp(op, len, l_arg.clone(), r_arg.clone());

                info.def = Some(def);
                info.deps = Some(extract_deps(&[&l_id, &r_id]));
                info.len = Some(len);
            },
            Def::Mux(sel, l, r) => {
                let (sel_len, sel_id, sel_arg) = find_arg(sel)?;
                let (l_len, l_id, l_arg) = find_arg(l)?;
                let (r_len, r_id, r_arg) = find_arg(r)?;

                if !(1 == sel_len && 1 == l_len && 1 == r_len) {
                    return Err("Mismatched lengths (expected 1).".to_string())
                }

                let def = VarDef::Mux(sel_arg.clone(), l_arg.clone(), r_arg.clone());

                info.def = Some(def);
                info.deps = Some(extract_deps(&[&sel_id, &l_id, &r_id]));
                info.len = Some(1);
            },
            Def::Reg(source) => {
                let (_, source_address, source_len) = find_var(source)?;
                
                if !(len == source_len) {
                    return Err("Mismatched lengths.".to_string())
                }

                info.def = Some(VarDef::Mem);
                info.deps = Some(vec!());
                info.len = Some(len);

                mem_ops.push(MemOp::Reg(RegOp {
                    data_len: len,
                    input: source_address,
                    output: address,
                }));
            },
            Def::Ram(data) => {
                let (ra_len, _ra_id, ra_arg) = find_arg(&data.read_address)?;
                let (we_len, _we_id, we_arg) = find_arg(&data.write_enable)?;
                let (wa_len, _wa_id, wa_arg) = find_arg(&data.write_address)?;
                let (da_len, _da_id, da_arg) = find_arg(&data.data)?;

                let addr_len = data.address_size;
                let word_len = data.word_size;

                if !(
                    addr_len == ra_len && addr_len == wa_len &&
                    1 == we_len && word_len == da_len &&
                    word_len == len
                ) {
                    return Err("Mismatched lengths.".to_string())
                }

                info.def = Some(VarDef::Mem);
                info.deps = Some(vec!());
                info.len = Some(len);

                mem_ops.push(MemOp::Ram(RamOp {
                    mem_id: mems.len(), // We will insert the corrsponding
                                        // MemInfo just after.
                    address_size: addr_len,
                    word_size: word_len,
                    read_address: ra_arg,
                    write_enable: we_arg,
                    write_address: wa_arg,
                    data: da_arg,
                    output: address,
                }));

                mems.push(MemInfo {
                    size: (2 as usize).pow(addr_len),
                    word_length: word_len,
                });
            },
            Def::Rom(data) => {
                let (ra_len, _ra_id, ra_arg) = find_arg(&data.read_address)?;

                let addr_len = data.address_size;
                let word_len = data.word_size;

                if !(
                    addr_len == ra_len &&
                    word_len == len
                ) {
                    return Err("Mismatched lengths.".to_string())
                }

                info.def = Some(VarDef::Mem);
                info.deps = Some(vec!());
                info.len = Some(len);

                mem_ops.push(MemOp::Rom(RomOp {
                    mem_id: mems.len(), // We will insert the corrsponding
                                        // MemInfo just after.
                    address_size: addr_len,
                    word_size: word_len,
                    read_address: ra_arg,
                    output: address,
                }));

                mems.push(MemInfo {
                    size: (2 as usize).pow(addr_len),
                    word_length: word_len,
                });
            },
            Def::Select(index, source) => {
                let (src_len, src_id, src_arg) = find_arg(source)?;

                if !(/*0 <= *index &&*/ *index < src_len) {
                    return Err(format!(
                            "Index ({}) out of bounds (length is {}).",
                            index, src_len
                    ))
                }

                if len != 1 {
                    return Err("Expected the output's length to be 1".to_string())
                }

                info.def = Some(VarDef::Select(*index, src_arg.clone()));
                info.deps = Some(extract_deps(&[&src_id]));
                info.len = Some(len);
            },
            Def::Slice(start, end, source) => {
                let (src_len, src_id, src_arg) = find_arg(source)?;

                if !(/*0 <= *start &&*/ start <= end && *end < src_len) {
                    return Err("Invalid slice bounds.".to_string())
                }

                if len != (end - start + 1) { // Range is inclusive.
                    return Err("Expected the output's length to be 1".to_string())
                }

                info.def = Some(VarDef::Slice(*start, *end, src_arg.clone()));
                info.deps = Some(extract_deps(&[&src_id]));
                info.len = Some(len);
            },
            Def::Concat(l, r) => {
                let (l_len, l_id, l_arg) = find_arg(l)?;
                let (r_len, r_id, r_arg) = find_arg(r)?;

                if !(len == l_len + r_len) {
                    return Err("Mismatched lengths.".to_string())
                }

                info.def = Some(VarDef::Concat(l_len, l_arg.clone(), r_len, r_arg.clone()));
                info.deps = Some(extract_deps(&[&l_id, &r_id]));
                info.len = Some(len);
            },
        }

        Ok(())
    })?;

    let mut validated_edges = Vec::with_capacity(edges.len());
    edges.into_iter().try_for_each(|info| -> Result<(), String> {
            validated_edges.push(info.validate()?);
            Ok(())
        })?;

    Ok(OpsGraph {
        mem_size,
        inputs,
        outputs,
        mems,
        edges: validated_edges,
        mem_ops,
    })
}

