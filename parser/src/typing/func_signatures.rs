use crate::ast::{StaticType, Function};
use super::data::{FuncSignature, is_compatible};

pub fn match_score(a: &StaticType, b: &StaticType) -> i32 {
    if *a == StaticType::Any || *b == StaticType::Any {
        0
    } else if *a == *b {
        1
    } else {
        -1
    }
}

pub fn is_this_call_ambiguous(args: Vec<StaticType>, functions: &Vec<FuncSignature>) -> bool {
    // Functions cannot be empty.
    let weights: Vec<i32> = functions.iter()
        .filter(|sig| is_callable_with(&args, sig))
        .map(|sig| compute_selectivity_weight(&args, sig))
        .collect();
    let optimal_call_weight = weights.iter().max().unwrap();

    //FIXME: clean me.
    //println!("[DEBUG] Ambiguous detection, here's the weights: {:?}, optimal call weight: {}", weights, optimal_call_weight);
    //println!("[DEBUG] Ambiguous detection, here's the args: {:?}, here's the functions: {:?}", args, functions);

    optimal_call_weight > &0 && weights.iter().filter(|w| w == &optimal_call_weight).count() > 1
}

// Assumes that s1 is compatible with s2.
fn most_precise_type(s1: StaticType, s2: StaticType) -> StaticType {
    assert!(is_compatible(s1.clone(), s2.clone()), "Cannot compute most precise type on non-compatible types!");

    match s1 {
        StaticType::Any => s2,
        _ => s1
    }
}

fn compute_enriched_signature(sig_1: Vec<StaticType>, sig_2: Vec<StaticType>, sig_3: Vec<StaticType>) -> Vec<StaticType> {
    sig_1.into_iter().zip(sig_2.into_iter()).zip(sig_3.into_iter()).map(|((s1, s2), s3)| most_precise_type(most_precise_type(s1, s2), s3)).collect()
}

pub fn format_signature(ts: Vec<StaticType>) -> String {
    ts.into_iter().map(|t| format!("::{}", t)).collect::<Vec<String>>().join(", ")
}

pub fn compute_ambiguous_signature(args: Vec<StaticType>, functions: &Vec<FuncSignature>) -> Option<Vec<StaticType>> {
    // The idea is simple
    // If an ambiguous signature exist, then it must be a (f_i, f_j) pair such that for all type in
    // the signature t_k^(i) is compatible with t_k^(j) *and* such that f_i is callable with args
    // and f_j callable with args.
    // we return the signature enriched from the args, that is, each time there is Any and we have
    // a more valuable type, we replace it.
    // So we just do O(N^2 S) to find such a sig and compute it.

    let n = functions.len();

    for i in 0..n {
        for j in i + 1..n {
            let f_i = &functions[i];
            let f_j = &functions[j];

            if args
                .iter()
                .zip(f_i.1.iter()).zip(f_j.1.iter())
                .all(|((arg, param_i), param_j)| is_compatible(param_i.clone(), param_j.clone()) && is_compatible(arg.clone(), param_i.clone()) && is_compatible(arg.clone(), param_j.clone())) {
                return Some(compute_enriched_signature(args, f_i.1.iter().cloned().collect(), f_j.1.iter().cloned().collect()));
            }
        }
    }

    None
}

pub fn compute_selectivity_weight(params: &Vec<StaticType>, target_sig: &FuncSignature) -> i32 {
    params
        .iter()
        .zip(target_sig.1.iter())
        .map(|(param_ty, target_ty)| match_score(param_ty, target_ty))
        .sum()
}

// Test if a certain set of StaticType match another signature.
// Useful for ambiguity and duplication detection.
pub fn is_callable_with(params: &Vec<StaticType>, target_sig: &FuncSignature) -> bool {
    params
        .into_iter()
        .zip(target_sig.1.iter())
        .all(|(param_ty, target_type)| is_compatible(param_ty.clone(), target_type.clone()))
}

pub fn is_callable_with_exactly(params: Vec<StaticType>, target_sig: &FuncSignature) -> bool {
    params
        .iter()
        .zip(target_sig.1.iter())
        .all(|(param_ty, target_type)| param_ty == target_type)
}

pub fn build_signature(f: &Function) -> FuncSignature {
    let (ret, mut params): FuncSignature = (f.ret_ty.clone(), vec![]);

    for param in &f.params {
        params.push(param.ty.clone());
    }

    (ret, params)
}

