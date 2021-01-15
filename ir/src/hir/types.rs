
pub enum Type {
    Int64,
    Bool,
    Str,
    Struct(String),
}

pub enum Val {
    Var(String),
    Const(Type, u64),
    Str(String),
}

pub enum BinOp {
    And, Or,
    Equ, Neq, Lt, Leq, Gt, Geq,
    Add, Sub, Mul, Div
}

pub enum Callable {
    Call(String, Vec<Val>),
    Bin(BinOp, Val, Val),

    Assign(Val),

    IsType(Val, Type),
    Access(Val, String, String),
}

pub enum Statement {
    FnCall(String, Vec<Val>),
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
    pub vars: Vec<String>,
    pub body: Block,
}

impl Function {
    pub fn new(name: String, args: Vec<String>, vars: Vec<String>, body: Block) -> Self {
        Function {name, args, vars, body}
    }
}

pub struct StructDecl {
    pub name: String,
    pub fields: Vec<String>,
}

impl StructDecl {
    pub fn new(name: String, fields: Vec<String>) -> StructDecl {
        StructDecl {name, fields}
    }
}

pub enum Decl {
    Function(Function),
    Struct(StructDecl),
}

