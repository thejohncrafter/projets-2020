
use automata::line_counter::SimpleSpan;

#[derive(Debug)]
pub struct LocatedIdent {
    name: String,
}

impl LocatedIdent {
    pub fn new(name: String) -> Self {
        LocatedIdent {name}
    }
}

#[derive(Debug)]
pub struct Param {
    name: LocatedIdent,
    ty: Option<LocatedIdent>,
}

impl Param {
    pub fn new(name: LocatedIdent, ty: Option<LocatedIdent>) -> Self {
        Param {name, ty}
    }
}

#[derive(Debug)]
pub struct LValue {
    // The chain of accesses to the lvalue.
    pub val: Vec<String>,
}

impl LValue {
    pub fn new(val: Vec<String>) -> Self {
        LValue {
            val: val,
        }
    }
}

#[derive(Debug)]
pub enum BinOp {
    Or,
    And,

    Equ,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,

    Plus,
    Minus,

    Times,
    Div,

    Pow,
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub struct Range {
    start: Exp,
    end: Exp,
}

impl Range {
    pub fn new(start: Exp, end: Exp) -> Self {
        Range {start, end}
    }
}

#[derive(Debug)]
pub enum ElseVal {
    End,
    Else(Block),
    ElseIf(Exp, Block, Else),
}

#[derive(Debug)]
pub struct Else {
    val: Box<ElseVal>,
}

impl Else {
    pub fn new(val: ElseVal) -> Self {
        Else {val: Box::new(val)}
    }
}

#[derive(Debug)]
pub enum ExpVal {
    Return(Exp),
    Assign(LValue, Exp),
    BinOp(BinOp, Exp, Exp),
    UnaryOp(UnaryOp, Exp),
    Call(String, Vec<Exp>),
    Int(i64),
    Str(String),
    Bool(bool),
    LValue(LValue),

    Block(Block),

    If(Exp, Block, Else),
    For(LocatedIdent, Range, Block),
    While(Exp, Block),
}

#[derive(Debug)]
pub struct Exp {
    pub val: Box<ExpVal>,
}

impl Exp {
    pub fn new(val: ExpVal) -> Self {
        Exp {
            val: Box::new(val),
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub val: Vec<Exp>,
}

impl Block {
    pub fn new(val: Vec<Exp>) -> Self {
        Block {val}
    }
}

#[derive(Debug)]
pub struct Structure {
    pub mutable: bool,
    pub name: LocatedIdent,
    pub fields: Vec<Param>,
}

impl Structure {
    pub fn new(mutable: bool, name: LocatedIdent, fields: Vec<Param>) -> Self {
        Structure {mutable, name, fields}
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Option<LocatedIdent>,
    pub body: Block,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<Param>,
        ret_ty: Option<LocatedIdent>,
        body: Block
    ) -> Self {
        Function {name, params, ret_ty, body}
    }
}

#[derive(Debug)]
pub enum DeclVal {
    Structure(Structure),
    Function(Function),
    Exp(Exp),
}

#[derive(Debug)]
pub struct Decl {
    val: DeclVal,
}

impl Decl {
    pub fn new(val: DeclVal) -> Self {
        Decl {val}
    }
}

