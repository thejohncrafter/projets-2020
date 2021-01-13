
pub enum Type {
    Int64,
    Bool,
    Str,
    Struct(String),
}

pub enum Val {
    Var(String),
    Const(u64, u64),
}

pub enum Callable {
    And(Val, Val),
    Or(Val, Val),

    Equ(Val, Val),
    Neq(Val, Val),
    Lt(Val, Val),
    Leq(Val, Val),
    Gt(Val, Val),
    Geq(Val, Val),

    Add(Val, Val),
    Sub(Val, Val),
    Mul(Val, Val),
    Div(Val, Val),

    Assign(Val),

    Cast(Val, Type),
    Access(Val, u64),
}

pub enum Statement {
    // Destination variable and called function
    Call(String, Callable),
    Return(Val),
    
    If(Val, Block, Block),
    While(Val, Block),
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
        Function {name, args, body}
    }
}

