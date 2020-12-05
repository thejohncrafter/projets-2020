use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::*;

use automata::read_error::ReadError;

#[derive(Debug)]
pub struct TypedDecls<'a> {
    pub functions: HashMap<String, Function<'a>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub global_expressions: Vec<Exp<'a>>
}

#[derive(Debug)]
pub struct TypingContext<'a> {
    pub functions: HashMap<String, Function<'a>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub known_types: HashSet<String>,
    pub mutable_fields: HashSet<String>,
    pub all_fields: HashMap<String, Option<StaticType>>,
    pub environment: HashMap<String, Option<StaticType>>
}

pub type TypingResult<'a> = Result<TypedDecls<'a>, ReadError<'a>>;
pub type ExprTypingResult<'a> = Result<(), ReadError<'a>>;
pub type BlockTypingResult<'a> = Result<(Block<'a>, TypingContext<'a>), ReadError<'a>>;
