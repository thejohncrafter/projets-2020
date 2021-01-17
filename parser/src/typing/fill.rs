use super::data::*;
use super::func_signatures::{is_this_call_ambiguous, format_signature, compute_ambiguous_signature};
use super::assign::{collect_all_assign_in_array};
use crate::ast::*;
use std::fmt;

use automata::line_counter::Span;

fn is_any_or<'a>(alpha: &'a Exp<'a>, t: StaticType) -> bool {
    return alpha.static_ty == StaticType::Any || alpha.static_ty == t;
}

fn is_one_of_or_any<'a>(alpha: &'a Exp<'a>, ts: &[StaticType]) -> bool {
    if alpha.static_ty == StaticType::Any {
        return true;
    }

    return ts.into_iter().any(|t| *t == alpha.static_ty)
}

pub fn type_block<'a>(tcx: &mut TypingContext<'a>, block: &mut Block<'a>) -> InternalTypingResult<'a> {
    for exp in &mut block.val {
        type_expression(tcx, exp)?;
    }

    if block.trailing_semicolon {
        block.static_ty = match block.val.last() {
            None => StaticType::Nothing,
            Some(ret_exp) => ret_exp.static_ty.clone()
        };
    }

    Ok(())
}

fn type_else<'a>(tcx: &mut TypingContext<'a>, else_: &mut Else<'a>) -> PartialTypingResult<'a> {
    match else_.val.as_mut() {
        ElseVal::End => Ok(StaticType::Nothing),
        ElseVal::Else(block) => {
            tcx.enter_in_local_scope();
            type_block(tcx, block)?;
            tcx.restore_previous_scope();
            Ok(block.static_ty.clone())
        },
        ElseVal::ElseIf(e, block, else_) => {
            type_expression(tcx, e)?;
            tcx.enter_in_local_scope();
            type_block(tcx, block)?;
            tcx.restore_previous_scope();
            let ret = type_else(tcx, else_)?;
            if ret == block.static_ty {
                Ok(block.static_ty.clone())
            } else {
                Ok(StaticType::Any)
            }
        }
    }
}


fn type_user_function<'a>(tcx: &mut TypingContext<'a>, span: &Span<'a>, name: &String, args: &mut Vec<Exp<'a>>) -> PartialTypingResult<'a> {
    let entity_types: Vec<StaticType>;

    if tcx.structures.contains_key(name) {
        entity_types = tcx.structures[name].fields.iter().map(|field| field.ty.clone()).collect();
    } else if tcx.functions.contains_key(name) && tcx.functions[name].len() == 1 {
        entity_types = tcx.functions[name].first().unwrap().1.clone();
    } else {
        entity_types = vec![StaticType::Any; args.len()];
    }

    for (arg, expected_ty) in args.iter_mut().zip(entity_types.iter()) {
        type_expression(tcx, arg)?;

        if !is_compatible(arg.static_ty.clone(), expected_ty.clone()) {
            return Err(
                (arg.span, format!("Incompatible types. Expected '{}', found '{}'", expected_ty, arg.static_ty).to_string()).into()
            );
        }
    }

    if let Some(ty) = tcx.get_potentially_unique_return_type_for_function(&name) {
        Ok(ty)
    } else {
        let st_args: Vec<StaticType> = args.iter().map(|arg| arg.static_ty.clone()).collect();
        // Ambiguity detection for functions.
        if tcx.functions.contains_key(name) && is_this_call_ambiguous(st_args.clone(), &tcx.functions[name]) {
                return Err(
                    (span.clone(), format!("Ambiguous call to function '{} ({})', cannot be resolve at runtime through dynamic dispatch", &name,
                    compute_ambiguous_signature(st_args, &tcx.functions[name]).and_then(|s| Some(format_signature(s))).unwrap_or("no information on the signature".to_string())).to_string()).into()
                );
        }

        Ok(StaticType::Any)
    }
}


pub fn type_simple_assign<'a>(tcx: &mut TypingContext<'a>, lv: &mut LValue<'a>, e: &mut Exp<'a>) -> InternalTypingResult<'a> {
    type_expression(tcx, e)?;

    match tcx.type_from_env_name(&lv.name) {
        None => {
            return Err(
                (lv.span, format!("Compiler error, '{}' was not found in the global typing context, unreachable variable. Environment was {:?}", &lv.name, tcx.environment).to_string()).into()
            );
        },
        Some(st) => {
            if !is_compatible(st.clone(), e.static_ty.clone()) {
                return Err(
                    (e.span, format!("Expected on the lhs '{}' type, found: '{}' on the rhs", st, e.static_ty).to_string()).into()
                );
            }
        }
    }

    Ok(())
}

fn type_complex_assign<'a>(tcx: &mut TypingContext<'a>, name: &String, span: &Span<'a>, prefix_e: &mut Exp<'a>, e: &mut Exp<'a>) -> InternalTypingResult<'a> {
    type_expression(tcx, prefix_e)?;

    // If prefix_e is known, we can check if the field exist.
    if !tcx.field_exist_in(&prefix_e.static_ty, name) {
        return Err(
            (span.clone(), format!("Field '{}' does not exist for the type '{}'", name, prefix_e.static_ty).to_string()).into()
        );
    }

    // If we do not know the type, we can just assume the static type to be the one which is
    // related to the unique structure containing this field if it exist at all.
    if prefix_e.static_ty == StaticType::Any {
        match tcx.structure_name_by_fields.get(name) {
            None => {
                return Err(
                (span.clone(), format!("Field '{}' is not declared anywhere in any structure", name).to_string()).into());
            },
            Some(s) => {
                prefix_e.static_ty = StaticType::Struct(s.clone());
            }
        }
    }

    if !tcx.mutable_fields.contains(name) {
        return Err(
            (span.clone(), format!("Field '{}' is not contained in a mutable structure, it cannot be assigned", name).to_string()).into()
        );
    }

    type_expression(tcx, e)?;

    if !is_compatible(tcx.all_fields[name].clone(),
        e.static_ty.clone()) {
        return Err(
            (e.span, format!("This expression has type '{}' but is incompatible with '{:?}' (declared in the structure)",
            e.static_ty,
            tcx.all_fields[name]).to_string()).into()
        );
    }

    Ok(())
}

fn raise_no_such_operation_err<'a, T: fmt::Display>(span: Span<'a>, op: T, ts: Vec<&StaticType>) -> InternalTypingResult<'a> {
    Err((span, format!(
                "No such operation '{}' for signature ({})", 
                op,
                format_signature(ts.into_iter().cloned().collect())
        )).into())
}

pub fn type_expression<'a>(tcx: &mut TypingContext<'a>, expr: &mut Exp<'a>) -> InternalTypingResult<'a> {
    match expr.val.as_mut() {
        ExpVal::Return(m_e) => {
            if let Some(e) = m_e {
                type_expression(tcx, e)?;
            }
            expr.static_ty = StaticType::Any;
        },
        ExpVal::Assign(lv, e) => {
            match lv.in_exp.as_mut() {
                None => {
                    type_simple_assign(tcx, lv, e)?;
                },
                Some(prefix_e) => {
                    type_complex_assign(tcx, &lv.name, &lv.span, prefix_e, e)?;
                }
            }
        },
        ExpVal::BinOp(op, a, b) => {
            type_expression(tcx, a)?;
            type_expression(tcx, b)?;

            match op {
                BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Mod | BinOp::Pow => {
                    if !is_any_or(&a, StaticType::Int64) {
                        return raise_no_such_operation_err(a.span, op, vec![&a.static_ty, &b.static_ty]);
                    }

                    if !is_any_or(&b, StaticType::Int64) {
                        return raise_no_such_operation_err(b.span, op, vec![&a.static_ty, &b.static_ty]);
                    }

                    expr.static_ty = StaticType::Int64;
                },
                BinOp::Equ | BinOp::Neq => {
                    expr.static_ty = StaticType::Bool;
                },
                BinOp::Lt | BinOp::Leq | BinOp::Gt | BinOp::Geq => {
                    let admissible_types = vec![StaticType::Int64, StaticType::Bool];

                    if !is_one_of_or_any(&a, &admissible_types) {
                        return raise_no_such_operation_err(a.span, op, vec![&a.static_ty, &b.static_ty]);
                    }
                    if !is_one_of_or_any(&b, &admissible_types) {
                        return raise_no_such_operation_err(b.span, op, vec![&a.static_ty, &b.static_ty]);
                    }

                    expr.static_ty = StaticType::Bool;
                },
                BinOp::And | BinOp::Or => {
                    if !is_any_or(&a, StaticType::Bool) {
                        return raise_no_such_operation_err(a.span, op, vec![&a.static_ty, &b.static_ty]);
                    }

                    if !is_any_or(&b, StaticType::Bool) {
                        return raise_no_such_operation_err(b.span, op, vec![&a.static_ty, &b.static_ty]);
                    }

                    expr.static_ty = StaticType::Bool;
                }
            }
        },
        ExpVal::UnaryOp(op, e) => {
            type_expression(tcx, e)?;

            match op {
                UnaryOp::Neg => {
                    if !is_any_or(&e, StaticType::Int64) {
                        raise_no_such_operation_err(e.span, op, vec![&e.static_ty])?;
                    }
                    expr.static_ty = StaticType::Int64;
                },
                UnaryOp::Not => {
                    if !is_any_or(&e, StaticType::Bool) {
                        raise_no_such_operation_err(e.span, op, vec![&e.static_ty])?;
                    }
                    expr.static_ty = StaticType::Bool;
                }
            }
        },
        ExpVal::Call(name, args) => {
            match name.as_str() {
                "div" => {
                    if args.len() != 2 {
                        return Err(
                            (expr.span, format!("`div` was called here with less or more than two arguments!").to_string()).into());
                    }

                    type_expression(tcx, &mut args[0])?;
                    type_expression(tcx, &mut args[1])?;

                    if is_any_or(&args[0], StaticType::Int64) && is_any_or(&args[1], StaticType::Int64) {
                        expr.static_ty = StaticType::Int64;
                    }
                },
                "print" => {
                    for arg in args {
                        type_expression(tcx, arg)?;
                    }

                    expr.static_ty = StaticType::Nothing;
                },
                _ => {
                    if !is_builtin_function(name) && !tcx.structures.contains_key(name) && !tcx.functions.contains_key(name) {
                        return Err(
                            (expr.span, format!("There is no such function or structure named '{}'", name).to_string()).into()
                        );
                    }

                    expr.static_ty = type_user_function(tcx, &expr.span, name, args)?;
                }
            }
        },
        ExpVal::Int(_) => expr.static_ty = StaticType::Int64,
        ExpVal::Str(_) => expr.static_ty = StaticType::Str,
        ExpVal::Bool(_) => expr.static_ty = StaticType::Bool,
        ExpVal::LValue(lv) => {
            match lv.in_exp.as_mut() {
                None => {
                    match tcx.type_from_env_name(&lv.name).zip(tcx.scope_from_env_name(&lv.name)) {
                        None => { 
                            return Err(
                                (lv.span, format!("No variable named '{}' is declared in this scope", &lv.name).to_string()).into()
                            );
                        },
                        Some((st, scope)) => {
                            lv.scope = scope;
                            expr.static_ty = st
                        }
                    }
                },
                Some(e) => {
                    type_expression(tcx, e)?;

                    if !tcx.field_exist_in(&e.static_ty, &lv.name) {
                        return Err(
                            (lv.span, format!("No field named '{}' in type '{}'", &lv.name, e.static_ty).to_string()).into()
                        );
                    }

                    if !tcx.all_fields.contains_key(&lv.name) {
                        return Err(
                            (lv.span, format!("No field named '{}' in any structure", &lv.name).to_string()).into()
                        );
                    }

                    if e.static_ty == StaticType::Any {
                        e.static_ty = StaticType::Struct(tcx.structure_name_by_fields.get(&lv.name).unwrap().clone());
                    }

                    expr.static_ty = tcx.all_fields[&lv.name].clone();
                    lv.scope = Scope::Local;
                }
            }
        },
        ExpVal::Block(block) => {
            type_block(tcx, block)?;
            expr.static_ty = StaticType::Any;
        },
        ExpVal::Mul(_, var) => {
            if !tcx.environment.contains_key(var) {
                return Err(
                    (expr.span, format!("No variable named '{}' is declared in this scope", var).to_string()).into());
            }
            // n*var: 3x
            expr.static_ty = StaticType::Int64;
        },
        ExpVal::LMul(_, block) => {
            // a(block)
            type_block(tcx, block)?;
            expr.static_ty = StaticType::Int64;
        },
        ExpVal::RMul(e, var) => {
            if !tcx.environment.contains_key(var) {
                return Err(
                    (expr.span, format!("No variable named '{}' is declared in this scope", var).to_string()).into());
            }
            // (expr)identfiant
            type_expression(tcx, e)?;
            expr.static_ty = StaticType::Int64;
        },
        ExpVal::If(e, block, else_) => {
            type_expression(tcx, e)?;

            if !is_any_or(e, StaticType::Bool) {
                return Err(
                    (e.span, format!("Non-boolean ({}) used in boolean context", e.static_ty).to_string()).into()
                );
            }

            tcx.enter_in_local_scope();
            type_block(tcx, block)?;
            tcx.restore_previous_scope();

            let ret_ty = type_else(tcx, else_)?;

            if block.static_ty != ret_ty {
                expr.static_ty = StaticType::Any;
            } else {
                expr.static_ty = block.static_ty.clone();
            }
        },
        ExpVal::For(ident, range, block) => {
            type_expression(tcx, &mut range.start)?;
            type_expression(tcx, &mut range.end)?;

            let local_extra_vars = tcx.extend_local_env(collect_all_assign_in_array(&block.val));
            tcx.push_to_env(&ident, StaticType::Int64, Scope::Local);
            tcx.enter_in_local_scope();

            type_block(tcx, block)?;

            tcx.restore_previous_scope();
            tcx.pop_from_env(&ident);
            tcx.unextend_env(local_extra_vars);
        },
        ExpVal::While(e, block) => {
            type_expression(tcx, e)?;

            if is_any_or(e, StaticType::Bool) {
                let local_extra_vars = tcx.extend_local_env(collect_all_assign_in_array(&block.val));
                tcx.enter_in_local_scope();

                type_block(tcx, block)?;

                tcx.restore_previous_scope();
                tcx.unextend_env(local_extra_vars);

                expr.static_ty = StaticType::Nothing;
            } else {
                return Err(
                    (e.span, format!("Non-boolean ({}) used in boolean context", e.static_ty).to_string()).into()
                );
            }
        },
    }
    Ok(())
}
