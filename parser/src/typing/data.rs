use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::*;

use automata::read_error::ReadError;

pub type ReturnVerification<'a> = Result<(), ReadError<'a>>;
pub type FuncSignature = (StaticType, Vec<StaticType>);

#[derive(Debug)]
pub struct TypedDecls<'a> {
    pub functions: HashMap<String, Vec<Function<'a>>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub global_expressions: Vec<Exp<'a>>
}

impl<'a> TypedDecls<'a> {
    pub fn from_global_environment(ges: GlobalEnvironmentState<'a>) -> Self {
        TypedDecls {
            functions: ges.functions,
            structures: ges.structures,
            global_expressions: ges.global_expressions
        }
    }
}

#[derive(Debug)]
pub struct GlobalEnvironmentState<'a> {
    pub structures: HashMap<String, Structure<'a>>,
    pub functions: HashMap<String, Vec<Function<'a>>>,
    pub function_sigs: HashMap<String, Vec<FuncSignature>>,
    pub all_structure_fields: HashMap<String, StaticType>,
    pub all_mutable_fields: HashSet<String>,
    pub global_variables: HashSet<String>,
    pub global_expressions: Vec<Exp<'a>>,
    pub known_types: HashSet<StaticType>,
}

impl<'a> GlobalEnvironmentState<'a> {
    pub fn init() -> Self {
        GlobalEnvironmentState {
            structures: HashMap::new(),
            functions: HashMap::new(),
            function_sigs: HashMap::new(),
            all_structure_fields: HashMap::new(),
            all_mutable_fields: HashSet::new(),
            global_variables: HashSet::new(),
            global_expressions: vec![],
            known_types: HashSet::new()
        }
    }
}

#[derive(Debug)]
pub struct TypingContext<'a> {
    pub functions: HashMap<String, Vec<FuncSignature>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub known_types: HashSet<StaticType>,
    pub mutable_fields: HashSet<String>,
    pub all_fields: HashMap<String, StaticType>,
    pub environment: HashMap<String, Vec<StaticType>>
}

impl<'a> TypingContext<'a> {
    pub fn push_to_env(&mut self, ident: &LocatedIdent<'a>, ty: StaticType) {
        self.environment
            .entry(ident.name.clone())
            .or_default()
            .push(ty);
    }

    pub fn push_local_to_env(&mut self, ident: &LocatedIdent<'a>) {
        self.push_to_env(&ident, StaticType::Any);
    }

    pub fn pop_from_env(&mut self, ident: &LocatedIdent<'a>) {
        let types = self.environment.get_mut(&ident.name).unwrap();
        types.pop();

        if types.len() == 1 {
            self.environment.remove(&ident.name);
        }
    }

    pub fn is_alive_in_env(&self, ident: &LocatedIdent<'a>) -> bool {
        self.environment.get(&ident.name).is_some()
    }
}

pub type InternalTypingResult<'a> = Result<(), ReadError<'a>>;

pub type TypingResult<'a> = Result<TypedDecls<'a>, ReadError<'a>>;

pub type ElseTypingResult<'a> = Result<Option<StaticType>, ReadError<'a>>;
pub type ExprTypingResult<'a> = Result<(), ReadError<'a>>;
pub type BlockTypingResult<'a> = Result<(), ReadError<'a>>;

pub fn convert_to_static_type(p: Option<&LocatedIdent>) -> Option<StaticType> {
    match p {
        None => None,
        Some(lident) => {
            Some(match lident.name.as_str() {
                "Any" => StaticType::Any,
                "Nothing" => StaticType::Nothing,
                "Int64" => StaticType::Int64,
                "Bool" => StaticType::Bool,
                "String" => StaticType::Str,
                _ => StaticType::Struct(lident.name.clone())
            })
        }
    }
}

pub fn collect_all_assign_in_array<'a>(a: &Vec<Exp<'a>>) -> Vec<String> {
    a.iter().flat_map(collect_all_assign).collect()
}

pub fn collect_all_assign<'a>(e: &Exp<'a>) -> Vec<String> {
    fn collect_else<'a>(u: &Else<'a>) -> Vec<String> {
        match u.val.as_ref() {
            ElseVal::End => vec![],
            ElseVal::Else(b) => collect_all_assign_in_array(&b.val),
            ElseVal::ElseIf(e, b, rest_) => collect_all_assign(&e)
                .into_iter()
                .chain(collect_all_assign_in_array(&b.val).into_iter())
                .chain(collect_else(&rest_).into_iter())
                .collect()
        }
    }

    // Perform a DFS on e to smoke out all Assign
    match e.val.as_ref() {
        ExpVal::Return(e) => match e {
            None => vec![],
            Some(e) => collect_all_assign(&e)
        },
        ExpVal::Assign(lv, e) => {
            let mut assigns = collect_all_assign(&e);
            match lv.in_exp {
                None => assigns.push(lv.name.clone()),
                _ => {}
            };
            assigns
        },
        ExpVal::BinOp(_, alpha, beta) => collect_all_assign(&alpha)
            .into_iter()
            .chain(collect_all_assign(&beta).into_iter())
            .collect(),
        ExpVal::UnaryOp(_, e) => collect_all_assign(&e),
        ExpVal::Call(_, e_s) => collect_all_assign_in_array(&e_s),
        ExpVal::Block(b) | ExpVal::LMul(_, b) => collect_all_assign_in_array(&b.val),
        ExpVal::RMul(e, _) => collect_all_assign(&e),
        ExpVal::If(e, b, else_branch) => collect_all_assign(&e)
            .into_iter()
            .chain(collect_all_assign_in_array(&b.val).into_iter())
            .chain(collect_else(&else_branch).into_iter())
            .collect(),
        ExpVal::For(_, _, _) | ExpVal::While(_, _) => vec![], 
        ExpVal::Int(_) | ExpVal::Str(_) | ExpVal::Bool(_) | ExpVal::Mul(_, _) => vec![],
        ExpVal::LValue(lv) => {
            match &lv.in_exp {
                None => vec![],
                Some(e) => collect_all_assign(e)
            }
        }
    }
}

