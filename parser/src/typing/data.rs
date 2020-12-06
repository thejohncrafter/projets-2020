use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::*;

use automata::read_error::ReadError;

pub fn is_compatible(alpha: Option<&StaticType>, beta: Option<&StaticType>) -> bool {
    match (alpha, beta) {
        (None, _) | (_, None) => true,
        (Some(a), Some(b)) => *a == StaticType::Any || *b == StaticType::Any || *a == *b
    }
}

pub type ReturnVerification<'a> = Result<(), ReadError<'a>>;
pub type FuncSignature = (Option<StaticType>, Vec<Option<StaticType>>);
pub fn build_signature(f: &Function) -> FuncSignature {
    let (ret, mut params): FuncSignature = (
        convert_to_static_type(f.ret_ty.as_ref()), vec![]);

    for param in &f.params {
        params.push(convert_to_static_type(param.ty.as_ref()));
    }

    (ret, params)
}

#[derive(Debug)]
pub struct TypedDecls<'a> {
    pub functions: HashMap<String, Vec<Function<'a>>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub global_expressions: Vec<Exp<'a>>
}

#[derive(Debug)]
pub struct TypingContext<'a> {
    pub functions: HashMap<String, Vec<FuncSignature>>,
    pub structures: HashMap<String, Structure<'a>>,
    pub known_types: HashSet<String>,
    pub mutable_fields: HashSet<String>,
    pub all_fields: HashMap<String, Option<StaticType>>,
    pub environment: HashMap<String, Vec<Option<StaticType>>>
}

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
        ExpVal::Return(e) => collect_all_assign(&e),
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
        ExpVal::For(_lident, _range, b) => collect_all_assign_in_array(&b.val),
        ExpVal::While(e, b) => collect_all_assign(&e)
            .into_iter()
            .chain(collect_all_assign_in_array(&b.val).into_iter())
            .collect(),
        ExpVal::Int(_) | ExpVal::Str(_) | ExpVal::Bool(_) | ExpVal::Mul(_, _) => vec![],
        ExpVal::LValue(lv) => {
            match &lv.in_exp {
                None => vec![],
                Some(e) => collect_all_assign(e)
            }
        }
    }
}

