
use super::types::*;

pub struct InputInfo<'a> {
    name: &'a str,
    len: u32,
    address: usize,
}

impl<'a> InputInfo<'a> {
    fn new(name: &'a str, len: u32, address: usize) -> Self {
        InputInfo {name, len, address}
    }

    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }
}

pub struct OutputInfo<'a> {
    name: &'a str,
    len: u32,
    address: usize,
}

impl<'a> OutputInfo<'a> {
    fn new(name: &'a str, len: u32, address: usize) -> Self {
        OutputInfo {name, len, address}
    }

    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }
}

struct AuxMem {
    word_size: u32,
    mem: Vec<bool>,
}

impl AuxMem {
    fn new(size: usize, word_size: u32) -> Self {
        AuxMem {
            word_size,
            mem: vec![false; word_size as usize * size],
        }
    }

    fn read(&self, address: usize) -> Vec<bool> {
        let base = address * self.word_size as usize;
        (0..self.word_size).map(|i| self.mem[base + i as usize]).collect()
    }

    fn write(&mut self, address: usize, data: &[bool]) {
        let base = address * self.word_size as usize;
        data.iter().enumerate().for_each(|(i, b)| self.mem[base + i] = *b);
    }
}

pub struct Runner<'a> {
    mem: Vec<bool>,
    aux_mems: Vec<AuxMem>,
    ops: &'a OpsList
}

impl<'a> Runner<'a> {
    pub fn new(ops: &'a OpsList) -> (Self, Vec<InputInfo<'a>>, Vec<OutputInfo<'a>>) {
        let mem = vec![false; ops.mem_size];
        let aux_mems = ops.mems.iter().map(|info| AuxMem::new(info.size, info.word_length)).collect();

        let inputs = ops.inputs.iter()
            .map(|(len, address, name)| InputInfo::new(name, *len, *address))
            .collect();
        let outputs = ops.outputs.iter()
            .map(|(len, address, name)| OutputInfo::new(name, *len, *address))
            .collect();

        (Runner {mem, aux_mems, ops}, inputs, outputs)
    }

    pub fn read(&self, output: &OutputInfo) -> Vec<bool> {
        (0..output.len).map(|i| {
                self.mem[output.address + i as usize]
            }).collect()
    }

    pub fn write(&mut self, input: &InputInfo, val: &[bool]) {
        if val.len() != input.len as usize {
            panic!("Expected {} values for input {}.", input.len, input.name)
        }

        for i in 0..input.len {
            self.mem[input.address + i as usize] = val[i as usize];
        }
    }

    fn get_value(&self, arg: &OpArg, offset: u32) -> bool {
        match arg {
            OpArg::Var(addr) => self.mem[addr + offset as usize],
            OpArg::Cst(vals) => vals[offset as usize],
        }
    }

    fn get_address(&self, arg: &OpArg, len: u32) -> usize {
        let mut addr = 0;
        
        for _ in 0..len {
            let bit = self.get_value(arg, 0);
            addr = 2*addr + (if bit {1} else {0});
        }
        
        addr
    }

    fn tick_logic(&mut self) -> Result<(), String> {
        self.ops.ops.iter().try_for_each(|(output, def)| -> Result<(), String> {
            match def {
                VarDef::Input => (), // Nothing to do now, this has already
                                     // been updated.
                VarDef::Fwd(len, arg) => {
                    for i in 0..*len {
                        self.mem[*output + i as usize] = self.get_value(arg, i);
                    }
                },
                VarDef::Not(len, arg) => {
                    for i in 0..*len {
                        self.mem[*output + i as usize] = !self.get_value(arg, i);
                    }
                },
                VarDef::BinOp(op, len, l, r) => {
                    for i in 0..*len {
                        let (l, r) = (self.get_value(l, i), self.get_value(r, i));
                        
                        let res = match op {
                            BinOp::Or => l || r,
                            BinOp::Xor => l ^ r,
                            BinOp::And => l && r,
                            BinOp::Nand => !(l && r),
                        };

                        self.mem[*output + i as usize] = res;
                    }
                },
                VarDef::Mux(sel, l, r) => {
                    let res = if self.get_value(sel, 0) == false {
                        self.get_value(l, 0)
                    } else {
                        self.get_value(r, 0)
                    };

                    self.mem[*output] = res;
                },
                VarDef::Mem => (), // Nothing to do now, this has already
                                   // been updated.
                VarDef::Select(index, source) => {
                    self.mem[*output] = self.get_value(source, *index);
                },
                VarDef::Slice(start, end, source) => {
                    // Range is inclusive.
                    for i in 0..=(end-start) {
                        self.mem[*output + i as usize] = self.get_value(source, start + i);
                    }
                },
                VarDef::Concat(l_len, l, r_len, r) => {
                    for i in 0..*l_len {
                        self.mem[*output + i as usize] = self.get_value(l, i);
                    }

                    for i in 0..*r_len {
                        self.mem[*output + (l_len + i) as usize] = self.get_value(r, i);
                    }
                },
            };

            Ok(())
        })?;

        Ok(())
    }

    fn tick_mem(&mut self) -> Result<(), String> {
        let mut mem_updates = Vec::new();
        let mut aux_mem_updates: Vec<(usize, usize, Vec<bool>)> = Vec::new();

        self.ops.mem_ops.iter().try_for_each(|op| -> Result<(), String> {
            match op {
                MemOp::Reg(op) => {
                    for i in 0..op.data_len {
                        mem_updates.push((
                                op.output + i as usize,
                                self.mem[op.input + i as usize]
                            ));
                    }
                },
                MemOp::Ram(op) => {
                    let read_addr = self.get_address(&op.read_address, op.address_size);
                    let write_enable = self.get_value(&op.write_enable, 0);

                    if write_enable {
                        let write_addr = self.get_address(&op.write_address, op.address_size);

                        let data = (0..op.word_size).map(|i| self.get_value(&op.data, i)).collect();
                        aux_mem_updates.push((op.mem_id, write_addr, data));
                    }

                    for i in 0..op.word_size {
                        let data = self.aux_mems[op.mem_id].read(read_addr);
                        mem_updates.push((op.output + i as usize, data[i as usize]));
                    }
                },
                MemOp::Rom(op) => {
                    let read_addr = self.get_address(&op.read_address, op.address_size);

                    for i in 0..op.word_size {
                        let data = self.aux_mems[op.mem_id].read(read_addr);
                        mem_updates.push((op.output + i as usize, data[i as usize]));
                    }
                },
            };

            Ok(())
        })?;

        mem_updates.into_iter().for_each(|(addr, value)| self.mem[addr] = value);
        aux_mem_updates.into_iter().for_each(|(mem_id, addr, value)|
                self.aux_mems[mem_id].write(addr, &value)
            );

        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), String> {
        self.tick_mem()?;
        self.tick_logic()?;

        Ok(())
    }
}

