
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
}

pub enum Instruction {
    And(String, Val, Val),
    Or(String, Val, Val),

    Equ(String, Val, Val),
    Neq(String, Val, Val),
    Lt(String, Val, Val),
    Leq(String, Val, Val),
    Gt(String, Val, Val),
    Geq(String, Val, Val),

    Add(String, Val, Val),
    Sub(String, Val, Val),
    Mul(String, Val, Val),
    Div(String, Val, Val),

    Mov(String, Val),

    Access(String, Val, u64),

    Jump(Label),
    Jumpif(Val, Label),

    Call(String, String, Vec<Val>),
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

