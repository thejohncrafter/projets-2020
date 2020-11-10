
use automata::line_counter::SimpleSpan;

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

pub enum ExpVal {
    BinOp(BinOp, Exp, Exp)
}

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

