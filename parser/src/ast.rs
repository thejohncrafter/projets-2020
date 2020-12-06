use std::fmt;
use automata::line_counter::Span;

#[derive(Debug)]
pub struct LocatedIdent<'a> {
    pub span: Span<'a>,
    pub name: String,
}

impl<'a> LocatedIdent<'a> {
    pub fn new(span: Span<'a>, name: String) -> Self {
        LocatedIdent {span, name}
    }
}

#[derive(Debug)]
pub struct Param<'a> {
    pub span: Span<'a>,
    pub name: LocatedIdent<'a>,
    pub ty: Option<LocatedIdent<'a>>,
}

impl<'a> Param<'a> {
    pub fn new(span: Span<'a>, name: LocatedIdent<'a>, ty: Option<LocatedIdent<'a>>) -> Self {
        Param {span, name, ty}
    }
}

#[derive(Debug)]
pub struct LValue<'a> {
    pub span: Span<'a>,
    pub in_exp: Option<Exp<'a>>,
    pub name: String,
}

impl<'a> LValue<'a> {
    pub fn new(span: Span<'a>, in_exp: Option<Exp<'a>>, name: String) -> Self {
        LValue {
            span,
            in_exp,
            name,
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
pub struct Range<'a> {
    pub span: Span<'a>,
    pub start: Exp<'a>,
    pub end: Exp<'a>,
}

impl<'a> Range<'a> {
    pub fn new(span: Span<'a>, start: Exp<'a>, end: Exp<'a>) -> Self {
        Range {span, start, end}
    }
}

#[derive(Debug)]
pub enum ElseVal<'a> {
    End,
    Else(Block<'a>),
    ElseIf(Exp<'a>, Block<'a>, Else<'a>),
}

#[derive(Debug)]
pub struct Else<'a> {
    pub span: Span<'a>,
    pub val: Box<ElseVal<'a>>,
}

impl<'a> Else<'a> {
    pub fn new(span: Span<'a>, val: ElseVal<'a>) -> Self {
        Else {span, val: Box::new(val)}
    }
}

#[derive(Debug)]
pub enum ExpVal<'a> {
    Return(Option<Exp<'a>>),
    Assign(LValue<'a>, Exp<'a>),
    BinOp(BinOp, Exp<'a>, Exp<'a>),
    UnaryOp(UnaryOp, Exp<'a>),
    Call(String, Vec<Exp<'a>>),
    Int(i64),
    Str(String),
    Bool(bool),
    LValue(LValue<'a>),

    Block(Block<'a>),

    Mul(i64, String),
    LMul(i64, Block<'a>),
    RMul(Exp<'a>, String),

    If(Exp<'a>, Block<'a>, Else<'a>),
    For(LocatedIdent<'a>, Range<'a>, Block<'a>),
    While(Exp<'a>, Block<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StaticType {
    Any,
    Nothing,
    Int64,
    Bool,
    Str,
    Struct(String)
}

impl fmt::Display for StaticType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StaticType::Any => write!(f, "Any"),
            StaticType::Nothing => write!(f, "Nothing"),
            StaticType::Int64 => write!(f, "Int64"),
            StaticType::Bool => write!(f, "Bool"),
            StaticType::Str => write!(f, "String"),
            StaticType::Struct(s) => write!(f, "Structure '{}'", s)
        }
    }
}

#[derive(Debug)]
pub struct Exp<'a> {
    pub span: Span<'a>,
    pub val: Box<ExpVal<'a>>,
    pub static_ty: Option<StaticType> // It should be populated as part of the static typing phase. If a type is unknown, then it must be Any.
}

impl<'a> Exp<'a> {
    pub fn new(span: Span<'a>, val: ExpVal<'a>) -> Self {
        Exp {
            span,
            val: Box::new(val),
            static_ty: None
        }
    }
}

#[derive(Debug)]
pub struct Block<'a> {
    pub span: Span<'a>,
    pub val: Vec<Exp<'a>>,
    pub trailing_semicolon: bool,
    pub static_ty: Option<StaticType>
}

impl<'a> Block<'a> {
    pub fn new(span: Span<'a>, val: Vec<Exp<'a>>, trailing_semicolon: bool) -> Self {
        Block {span, val, trailing_semicolon, static_ty: None}
    }
}

#[derive(Debug)]
pub struct Structure<'a> {
    pub span: Span<'a>,
    pub mutable: bool,
    pub name: LocatedIdent<'a>,
    pub fields: Vec<Param<'a>>,
}

impl<'a> Structure<'a> {
    pub fn new(span: Span<'a>, mutable: bool, name: LocatedIdent<'a>, fields: Vec<Param<'a>>) -> Self {
        Structure {span, mutable, name, fields}
    }
}

#[derive(Debug)]
pub struct Function<'a> {
    pub span: Span<'a>,
    pub name: String,
    pub params: Vec<Param<'a>>,
    pub ret_ty: Option<LocatedIdent<'a>>,
    pub body: Block<'a>,
}

impl<'a> Function<'a> {
    pub fn new(
        span: Span<'a>,
        name: String,
        params: Vec<Param<'a>>,
        ret_ty: Option<LocatedIdent<'a>>,
        body: Block<'a>
    ) -> Self {
        Function {span, name, params, ret_ty, body}
    }
}

#[derive(Debug)]
pub enum DeclVal<'a> {
    Structure(Structure<'a>),
    Function(Function<'a>),
    Exp(Exp<'a>),
}

#[derive(Debug)]
pub struct Decl<'a> {
    pub span: Span<'a>,
    pub val: DeclVal<'a>,
}

impl<'a> Decl<'a> {
    pub fn new(span: Span<'a>, val: DeclVal<'a>) -> Self {
        Decl {span, val}
    }
}

