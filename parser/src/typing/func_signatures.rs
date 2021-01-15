use crate::ast::{StaticType, Function};
use super::data::FuncSignature;

pub fn is_compatible(a: StaticType, b: StaticType) -> bool {
    a == StaticType::Any || b == StaticType::Any || a == b
}

pub fn match_score(a: &StaticType, b: &StaticType) -> i32 {
    if *a == *b {
        1
    } else if *a == StaticType::Any || *b == StaticType::Any {
        0
    } else {
        -1
    }
}

/*pub fn is_this_call_ambiguous(args: Vec<StaticType>, functions: &Vec<FuncSignature>) -> bool {
    // Functions cannot be empty.
    let weights: Vec<i32> = functions.iter()
        .filter(|sig| is_callable_with(&args, sig))
        .map(|sig| compute_selectivity_weight(&args, sig))
        .collect();
    let optimal_call_weight = weights.iter().max().unwrap();

    println!("Ambiguous detection, here's the weights: {:?}", weights);

    optimal_call_weight > &0 && weights.iter().filter(|w| w == &optimal_call_weight).count() > 1
}

pub fn compute_selectivity_weight(params: &Vec<StaticType>, target_sig: &FuncSignature) -> i32 {
    params
        .iter()
        .zip(target_sig.1.iter())
        .map(|(param_ty, target_ty)| match_score(*param_ty, target_ty.as_ref()))
        .sum()
}

// Test if a certain set of StaticType match another signature.
// Useful for ambiguity and duplication detection.
pub fn is_callable_with(params: &Vec<&StaticType>, target_sig: &FuncSignature) -> bool {
    params
        .into_iter()
        .zip(target_sig.1.iter())
        .all(|(param_ty, target_type)| is_compatible(*param_ty, target_type.as_ref()))
}*/

pub fn is_callable_with_exactly(params: Vec<StaticType>, target_sig: &FuncSignature) -> bool {
    params
        .iter()
        .zip(target_sig.1.iter())
        .all(|(param_ty, target_type)| match_score(param_ty, target_type) == 1)
}

pub fn build_signature(f: &Function) -> FuncSignature {
    let (ret, mut params): FuncSignature = (f.ret_ty.clone(), vec![]);

    for param in &f.params {
        params.push(param.ty.clone());
    }

    (ret, params)
}

