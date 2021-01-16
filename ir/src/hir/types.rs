use parser::ast as ast;

#[derive(Debug, Clone)]
pub enum Type {
    Nothing,
    Int64,
    Bool,
    Str,
    Struct(String),
}

#[derive(Debug, Clone)]
pub enum Val {
    Nothing,
    Var(String),
    Const(Type, u64),
    Str(String),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    And, Or,
    Equ, Neq, Lt, Leq, Gt, Geq,
    Add, Sub, Mul, Div,
}

#[derive(Debug, Clone)]
pub enum Callable {
    Call(String, Vec<Val>),
    Bin(BinOp, Val, Val),

    Assign(Val),

    IsType(Val, Type),
    Access(Val, String, String),
}

#[derive(Debug, Clone)]
pub enum Statement {
    FnCall(String, Vec<Val>),
    // Destination variable and called function
    Call(String, Callable),
    Return(Val),
    
    If(Val, Block, Block),
    While(Val, Block),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Statement>,
}

impl Block {
    pub fn new(stmts: Vec<Statement>) -> Self {
        Block {stmts}
    }

    pub fn push(&mut self, stmt: Statement) {
        self.stmts.push(stmt);
    }

    pub fn extend(&mut self, stmts: Vec<Statement>) -> &mut Self {
        self.stmts.extend(stmts);
        self
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

