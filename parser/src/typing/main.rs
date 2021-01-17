use std::collections::HashSet;
use crate::ast::*;
use super::data::*;
use super::fill::{type_expression, type_block};
//use super::fill;
use super::visit::IntoVisitor;
use super::returns::{verify_explicit_returns, verify_implicit_return};
use super::assign::collect_all_assign_in_array;

fn to_env(known: &HashSet<String>) -> EnvironmentMap {
    known.into_iter().map(|t| (t.clone(), vec![EnvVariable::init()])).collect()
}

pub fn static_type<'a>(decls: Vec<Decl<'a>>) -> TypingResult<'a> {
    // Step 1. Build the global environment.
    let mut global_state: GlobalEnvironmentState<'a> = GlobalEnvironmentState::init();
    // Walk over declarations for phase 1-typing.
    for decl in decls {
        global_state.visit_decl(decl)?; // It will consume decl forever.
    }

    // Prepare for the global environment.
    let mut environment = to_env(&global_state.global_variables);

    // Add nothing: Nothing in the future environment.
    environment
        .entry("nothing".to_string())
        .or_default()
        .push(EnvVariable::typed(StaticType::Nothing));

    // Step 2.
    // Iterate over all declarations.
    // Looks like déjà vu. :>
    let mut global_ctx = TypingContext {
        functions: global_state.function_sigs,
        structures: global_state.structures,
        known_types: global_state.known_types,
        mutable_fields: global_state.all_mutable_fields,
        all_fields: global_state.all_structure_fields,
        structure_name_by_fields: global_state.structure_name_by_fields,
        current_scope: Scope::Global,
        previous_scope: Scope::Global,
        environment
    };

    //  If it's an expression, type the expr in the global environment.
    for expr in &mut global_state.global_expressions {
        type_expression(&mut global_ctx, expr)?;
    }

    //  If it's a function, build a Γ environment shadowing the global one.
    //      Then, add all local variables inside of the block.
    //      Then, type the block.
    //      Then, if possible, check all the returns.
    for funcs in global_state.functions.values_mut() {
        for func in funcs {
            for arg in &func.params {
                global_ctx.push_to_env(&arg.name, arg.ty.clone(), Scope::Local);
            }

            let extra_local_vars = global_ctx.extend_local_env(collect_all_assign_in_array(&func.body.val));
            
            type_block(&mut global_ctx, &mut func.body)?;
            verify_implicit_return(&func)?;
            verify_explicit_returns(&func.body, func.ret_ty.clone())?;

            for arg in &func.params {
                global_ctx.pop_from_env(&arg.name);
            }

            global_ctx.unextend_env(extra_local_vars);
        }
    }


    // Returns the enriched declarations.
    Ok(TypedDecls {
        functions: global_state.functions,
        structures: global_ctx.structures,
        global_expressions: global_state.global_expressions
    })
}
