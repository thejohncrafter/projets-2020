
#[derive(Clone)]
pub enum OpArg {
    Var(usize), // Stores the address.
    Cst(Vec<bool>),
}

pub enum BinOp {
    Or, Xor, And, Nand,
}

pub enum VarDef {
    Input, // This variable will be written before each tick.
    Fwd(u32, OpArg),
    Not(u32, OpArg),
    BinOp(BinOp, u32, OpArg, OpArg),
    Mux(OpArg, OpArg, OpArg),
    Mem, // No need to store the dependencies here,
         // the variable will be rewritten
         // at the end of each tick.
    Select(u32, OpArg),
    Slice(u32, u32, OpArg),
    Concat(u32, OpArg, u32, OpArg),
}

pub struct RegOp {
    pub data_len: u32,
    pub input: usize,
    pub output: usize,
}

pub struct RamOp {
    pub mem_id: usize,
    pub address_size: u32,
    pub word_size: u32,
    pub read_address: OpArg,
    pub write_enable: OpArg,
    pub write_address: OpArg,
    pub data: OpArg,
    pub output: usize,
}

pub struct RomOp {
    pub mem_id: usize,
    pub address_size: u32,
    pub word_size: u32,
    pub read_address: OpArg,
    pub output: usize,
}

pub enum MemOp {
    Reg(RegOp),
    Ram(RamOp),
    Rom(RomOp),
}

pub struct MemInfo {
    pub size: usize, // Will be 2^addr_len.
    pub word_length: u32,
}

pub struct VarInfo {
    pub name: String,
    pub address: usize,
    pub def: VarDef,
    pub deps: Vec<usize>,
    pub len: u32,
}

pub struct PartialVarInfo {
    pub name: String,
    pub address: usize,
    pub def: Option<VarDef>,
    pub deps: Option<Vec<usize>>,
    pub len: Option<u32>,
}

impl PartialVarInfo {
    pub fn new(name: String, address: usize) -> Self {
        PartialVarInfo {
            name,
            address,
            def: None,
            deps: None,
            len: None,
        }
    }

    pub fn check_empty(&self) -> Result<(), String> {
        if !(self.def.is_none() && self.deps.is_none() && self.len.is_none()) {
            Err(format!("Variable {} is defined twice.", self.name))
        } else {
            Ok(())
        }
    }

    pub fn validate(self) -> Result<VarInfo, String> {
        let name = &self.name;
        let err = || format!("Variable {} hasn't been defined.", name);
 
        let def = self.def.ok_or_else(err)?;
        let deps = self.deps.ok_or_else(err)?;
        let len = self.len.ok_or_else(err)?;

        Ok(VarInfo {
            name: self.name,
            address: self.address,
            def, deps, len,
        })
    }
}

pub struct OpsGraph {
    pub mem_size: usize,
    pub inputs: Vec<(u32, usize, String)>,
    pub outputs: Vec<(u32, usize, String)>,
    pub mems: Vec<MemInfo>,
    pub edges: Vec<VarInfo>,
    pub mem_ops: Vec<MemOp>, 
}

pub struct OpsList {
    pub mem_size: usize,
    pub inputs: Vec<(u32, usize, String)>,
    pub outputs: Vec<(u32, usize, String)>,
    pub mems: Vec<MemInfo>,
    pub ops: Vec<(usize, VarDef)>, // The address of the variable and its definition.
    pub mem_ops: Vec<MemOp>,
}

