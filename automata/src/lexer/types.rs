
use std::collections::{BTreeMap, BTreeSet};

pub fn is_behaved(c: char) -> bool {
    (c != '\\') && (c != '"') && (c != '\n')
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Character {
    Char(char),
    Alpha,
    Num,
    Behaved,
    Any,
}

#[derive(Clone)]
pub enum Regexp {
    Epsilon,
    Character(Character),
    Union(Box<Regexp>, Box<Regexp>),
    Concat(Box<Regexp>, Box<Regexp>),
    Star(Box<Regexp>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum IChar {
    Char(Character, usize),
    // Also stores the id of the recognized token.
    Hash(usize),
}

pub type CSet = BTreeSet<IChar>;
pub type State = CSet;

pub type TransMap = BTreeMap<Character, State>;

#[derive(Clone)]
pub enum IRegexp {
    Epsilon,
    Character(IChar),
    Union(Box<IRegexp>, Box<IRegexp>),
    Concat(Box<IRegexp>, Box<IRegexp>),
    Star(Box<IRegexp>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CharOrEof {
    Char(Character),
    Eof(usize),
}

