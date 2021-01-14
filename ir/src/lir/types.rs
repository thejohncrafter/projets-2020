
pub struct Label {
    pub name: String,
}

impl Label {
    pub fn new(name: String) -> Self {
        Label {name}
    }
}

pub enum Val {
    Var(String),
    Const(u64),
    Str(String),
}

pub enum BinOp {
    And, Or,
    Equ, Neq, Lt, Leq, Gt, Geq,
    Add, Sub, Mul, Div,
}

pub enum Instruction {
    Bin(String, BinOp, Val, Val),

    Mov(String, Val),

    Access(String, Val, u64),

    Jump(Label),
    Jumpif(Val, Label),

    Call(Option<(String, String)>, String, Vec<Val>),
    Return(Val, Val),
}

pub enum Statement {
    Label(Label),
    Inst(Instruction),
}

pub struct Block {
    pub stmts: Vec<Statement>,
}

impl Block {
    pub fn new(stmts: Vec<Statement>) -> Self {
        Block {stmts}
    }
}

pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: Block,
}

impl Function {
    pub fn new(name: String, args: Vec<String>, body: Block) -> Self {
        Function {
            name, args, body
        }
    }
}

