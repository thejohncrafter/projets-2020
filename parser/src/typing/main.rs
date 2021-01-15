use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::*;
use super::data::*;
use super::fill::{type_expression, type_block};
//use super::fill;
use super::visit::IntoVisitor;
// use super::returns::verify_returns;

fn to_env(known: &HashSet<String>) -> HashMap<String, Vec<StaticType>> {
    known.into_iter().map(|t| (t.clone(), vec![StaticType::Any])).collect()
}

pub fn static_type<'a>(decls: Vec<Decl<'a>>) -> TypingResult<'a> {
    // Step 1. Build the global environment.
    let mut global_state: GlobalEnvironmentState<'a> = GlobalEnvironmentState::init();
    // Walk over declarations for phase 1-typing.
    for decl in decls {
        global_state.visit_decl(&decl)?;
    }

    // Prepare for the global environment.
    let mut environment = to_env(&global_state.global_variables);

    // Add nothing: Nothing in the future environment.
    environment
        .entry("nothing".to_string())
        .or_default()
        .push(StaticType::Nothing);

    // Step 2.
    // Iterate over all declarations.
    // Looks like déjà vu. :>
    let mut global_ctx = TypingContext {
        functions: global_state.function_sigs,
        structures: global_state.structures,
        known_types: global_state.known_types,
        mutable_fields: global_state.all_mutable_fields,
        all_fields: global_state.all_structure_fields,
        environment
    };

    let mut global_expressions = global_state.global_expressions;

    //  If it's an expression, type the expr in the global environment.
    for expr in &mut global_expressions {
        type_expression(&mut global_ctx, expr)?;
    }

    //  If it's a function, build a Γ environment shadowing the global one.
    //      Then, add all local variables inside of the block.
    //      Then, type the block.
    //      Then, if possible, check all the returns.
    for funcs in global_state.functions.values_mut() {
        for func in funcs {
            for arg in &func.params {
                global_ctx.push_to_env(&arg.name, arg.ty.clone());
            }

            let extra_local_vars: Vec<LocatedIdent> = vec![];//collect_all_assign_in_array(&func.body.val);
            for var in &extra_local_vars {
                // No need to pollute and get inferior types.
                if !global_ctx.environment.contains_key(&var.name) || var.name == "nothing" {
                    global_ctx.push_local_to_env(var);
                }
            }

            type_block(&mut global_ctx, &mut func.body)?;
            // global_ctx.verify_returns(&func);

            /*if let Some(ret_ty) = convert_to_static_type(func.ret_ty.as_ref()) {
                // Implicit return.
                if !func.body.trailing_semicolon && func.body.val.len() > 0 && !is_compatible(func.body.val.last().unwrap().static_ty.as_ref(), Some(&ret_ty)) {
                    return Err(
                        (func.body.val.last().unwrap().span, format!("Invalid implicit return type, expected '{:?}', found '{:?}'",
                                                                     func.body.val.last().unwrap().static_ty,
                                                                     ret_ty).to_string()).into()
                    );
                }
                // Explicit returns.
                verify_returns(&func.body, ret_ty)?;
            }*/


            for arg in &func.params {
                global_ctx.pop_from_env(&arg.name);
            }

            for var in extra_local_vars {
                if global_ctx.is_alive_in_env(&var) {
                    global_ctx.pop_from_env(&var);
                } else {
                    println!("WARNING: '{:?}' was already deleted!", &var);
                }
            }
        }
    }


    // Returns the enriched declarations.
    Ok(TypedDecls::from_global_environment(global_state))
}
