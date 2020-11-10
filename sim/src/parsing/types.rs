
#[derive(Debug)]
pub enum ValueType {
    Bit,
    BitArray(u32),
}

#[derive(Debug)]
pub enum Value {
    Bit(bool),
    BitArray(Vec<bool>),
}

#[derive(Debug)]
pub enum Arg {
    Const(Value),
    Var(String),
}

#[derive(Debug)]
pub enum BinOpType {
    Or, Xor, And, Nand,
}

#[derive(Debug)]
pub struct RamData {
    pub address_size: u32,
    pub word_size: u32,
    pub read_address: Arg,
    pub write_enable: Arg,
    pub write_address: Arg,
    pub data: Arg,
}

#[derive(Debug)]
pub struct RomData {
    pub address_size: u32,
    pub word_size: u32,
    pub read_address: Arg,
}

#[derive(Debug)]
pub enum Def {
    Fwd(Arg),
    Not(Arg),
    Bin(BinOpType, Arg, Arg),
    Mux(Arg, Arg, Arg),
    Reg(String),
    Ram(RamData),
    Rom(RomData),
    Select(u32, Arg),
    Slice(u32, u32, Arg),
    Concat(Arg, Arg),
}

#[derive(Debug)]
pub struct Netlist {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub vars: Vec<(String, ValueType)>,
    pub defs: Vec<(String, Def)>,
}

