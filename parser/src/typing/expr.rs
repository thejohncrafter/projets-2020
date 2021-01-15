use std::collections::HashSet;
use super::data::*;
use crate::ast::*;


// We want to know if target improves the precision of the original type.
fn is_valuable_type(origin: Option<&StaticType>, target: Option<&StaticType>) -> bool {
    match origin {
        None => true,
        Some(s) => match target {
            None => false,
            Some(t) => match (s, t) {
                (StaticType::Any, _) => true,
                (_, StaticType::Any) => false,
                (_, _) => true
            }
        }
    }
}

