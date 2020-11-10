
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Symbol {
    // Terminal
    T(usize),
    // Non-terminal
    N(usize),
}

pub struct Production {
   pub symbol: usize,
   pub expand: Vec<Symbol>
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Shift(usize),
    Reduce(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum Goto {
    Accept,
    Some(usize),
    None,
}

