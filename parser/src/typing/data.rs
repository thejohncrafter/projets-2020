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

pub fn compatibility_value(alpha: Option<&StaticType>, beta: Option<&StaticType>) -> i32 {
    match (alpha, beta) {
        (None, None) => 1,
        (None, _) | (_, None) => 0,
        (Some(a), Some(b)) => {
            if *a == *b {
                1
            } else if *a == StaticType::Any {
                0
            } else if *b == StaticType::Any {
                0
            } else {
                -1
            }
        }
    }
}

pub type ReturnVerification<'a> = Result<(), ReadError<'a>>;
pub type FuncSignature = (Option<StaticType>, Vec<Option<StaticType>>);

pub fn is_this_call_ambiguous(args: Vec<Option<&StaticType>>, functions: &Vec<FuncSignature>) -> bool {
    // Functions cannot be empty.
    let weights: Vec<i32> = functions.iter()
        .filter(|sig| is_callable_with(&args, sig))
        .map(|sig| compute_selectivity_weight(&args, sig))
        .collect();
    let optimal_call_weight = weights.iter().max().unwrap();

    println!("Ambiguous detection, here's the weights: {:?}", weights);

    optimal_call_weight > &0 && weights.iter().filter(|w| w == &optimal_call_weight).count() > 1
}

pub fn compute_selectivity_weight(params: &Vec<Option<&StaticType>>, target_sig: &FuncSignature) -> i32 {
    params
        .iter()
        .zip(target_sig.1.iter())
        .map(|(param_ty, target_ty)| compatibility_value(*param_ty, target_ty.as_ref()))
        .sum()
}

// Test if a certain set of StaticType match another signature.
// Useful for ambiguity and duplication detection.
pub fn is_callable_with(params: &Vec<Option<&StaticType>>, target_sig: &FuncSignature) -> bool {
    params
        .into_iter()
        .zip(target_sig.1.iter())
        .all(|(param_ty, target_type)| is_compatible(*param_ty, target_type.as_ref()))
}

pub fn is_callable_with_exactly(params: Vec<Option<StaticType>>, target_sig: &FuncSignature) -> bool {
    params
        .iter()
        .zip(target_sig.1.iter())
        .all(|(param_ty, target_type)| compatibility_value(param_ty.as_ref(), target_type.as_ref()) == 1)
}

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

