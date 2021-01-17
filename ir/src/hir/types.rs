
use parser::ast as ast;
use std::iter::FromIterator;

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

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Callable {
    //  * Called function name;
    //  * Is is a native function ?;
    //  * Arguments for the call.
    Call(String, bool, Vec<Val>),
    Bin(BinOp, Val, Val),
    Unary(UnaryOp, Val),

    Assign(Val),

    // Allocates memory space for a structure
    Alloc(String),
    IsType(Val, Type),
    Access(Val, String, String),
}

#[derive(Debug, Clone)]
pub enum LValue {
    Var(String),
    Access(Val, String, String),
}

#[derive(Debug, Clone)]
pub enum Statement {
    // Destination variable and called function
    Call(LValue, Callable),
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

    pub fn merge(mut self, block: Block) -> Self {
        self.stmts.extend(block.stmts);
        self
    }
}

impl IntoIterator for Block {
    type Item = Statement;
    type IntoIter = std::vec::IntoIter<Statement>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.into_iter()
    }
}

impl FromIterator<Statement> for Block {
    fn from_iter<I: IntoIterator<Item=Statement>>(iter: I) -> Self {
        let mut c = vec![];

        for i in iter {
            c.push(i);
        }

        Block::new(c)
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

pub struct Source {
    pub globals: Vec<String>,
    pub decls: Vec<Decl>,
}

impl Source {
    pub fn new(globals: Vec<String>, decls: Vec<Decl>) -> Self {
        Source {globals, decls}
    }
}

