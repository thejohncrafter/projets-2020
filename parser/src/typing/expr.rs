use super::data::*;
use crate::ast::*;

fn is_any_or<'a>(alpha: &'a Exp<'a>, t: StaticType) -> bool {
    return alpha.static_ty == Some(StaticType::Any) || alpha.static_ty == Some(t);
}

fn is_one_of_or_any<'a>(alpha: &'a Exp<'a>, ts: &[StaticType]) -> bool {
    if alpha.static_ty == Some(StaticType::Any) {
        return true;
    }

    if let Some(static_ty) = &alpha.static_ty {
        return ts.into_iter().any(|t| *t == *static_ty)
    } else {
        return false;
    }
}

fn is_builtin_function(name: &String) -> bool {
    match name.as_str() {
        "println" | "div" | "print" => true,
        _ => false
    }
}

fn field_exist_in<'a>(structure_type: &StaticType, field_name: &String, ctx: &'a TypingContext<'a>) -> bool {
    println!("DEBUG: Checking if field: '{:?}' exist for '{:?}'", structure_type, field_name);
    match structure_type {
        StaticType::Any => true,
        StaticType::Nothing | StaticType::Int64 | StaticType::Str | StaticType::Bool  => false,
        StaticType::Struct(s) => ctx.structures[s].fields.iter().any(|p| &p.name.name == field_name)
    }
}

fn get_unique_function_ret_type<'a>(name: &String, ctx: &'a TypingContext<'a>) -> Option<StaticType> {
    match ctx.functions.get(name) {
        None => None,
        Some(same_functions) => match same_functions.len() > 1 {
            true => None,
            false => same_functions.first().unwrap().0.clone()
        }
    }
}

fn type_else<'a>(else_: &mut Else<'a>, ctx: &mut TypingContext<'a>) -> ElseTypingResult<'a> {
    match else_.val.as_mut() {
        ElseVal::End => Ok(None),
        ElseVal::Else(block) => {
            type_block(block, ctx)?;
            Ok(block.static_ty.clone())
        },
        ElseVal::ElseIf(e, block, else_) => {
            type_expression(e, ctx)?;
            type_block(block, ctx)?;
            let ret = type_else(else_, ctx)?;
            match ret {
                None => Ok(block.static_ty.clone()),
                Some(t) => {
                    if Some(t) == block.static_ty {
                        Ok(block.static_ty.clone())
                    } else {
                        Ok(Some(StaticType::Any))
                    }
                }
            }
        }
    }
}

pub fn type_block<'a>(block: &mut Block<'a>, context: &mut TypingContext<'a>) -> BlockTypingResult<'a> {
    for exp in &mut block.val {
        type_expression(exp, context)?;
    }

    if block.trailing_semicolon {
        block.static_ty = match block.val.last() {
            None => Some(StaticType::Nothing),
            Some(ret_exp) => ret_exp.static_ty.clone()
        };
    }

    Ok(())
}

pub fn type_expression<'a>(toplevel: &mut Exp<'a>, context: &mut TypingContext<'a>) -> ExprTypingResult<'a> {

    fn fill_types<'a>(expr: &mut Exp<'a>, ctx: &mut TypingContext<'a>) -> ExprTypingResult<'a> {
        match expr.val.as_mut() {
            ExpVal::Return(e) => {
                fill_types(e, ctx)?;
                expr.static_ty = Some(StaticType::Any);
            },
            ExpVal::Assign(lv, e) => {
                match &mut lv.in_exp {
                    None => {
                        fill_types(e, ctx)?;

                        if !ctx.environment.contains_key(&lv.name) {
                            return Err(
                                (lv.span, format!("Compiler error, '{}' was not found in the global typing context, unreachable variable. Environment was {:?}", &lv.name, ctx.environment).to_string()).into()
                            );
                        }

                        if !is_compatible(ctx.environment[&lv.name].last().and_then(|t| t.as_ref()),
                            e.static_ty.as_ref()) {
                            return Err(
                                (e.span, format!("This expression has type '{:?}' but is incompatible with '{:?}' (expected)", e.static_ty, ctx.environment[&lv.name].last().unwrap()).to_string()).into()
                            );
                        }
                    },
                    Some(prefix_e) => {
                        fill_types(prefix_e, ctx)?;

                        // If prefix_e is known, we can check if the field exist.
                        if let Some(st) = &prefix_e.static_ty {
                            if !field_exist_in(st, &lv.name, ctx) {
                                return Err(
                                    (lv.span, format!("Field '{}' does not exist for the type '{:?}'", &lv.name, st).to_string()).into()
                                );
                            }
                        }

                        if !ctx.mutable_fields.contains(&lv.name) {
                            return Err(
                                (lv.span, format!("Field '{}' is not contained in a mutable structure, it cannot be assigned", &lv.name).to_string()).into()
                            );
                        }

                        fill_types(e, ctx)?;

                        if !is_compatible(ctx.all_fields[&lv.name].as_ref(),
                            e.static_ty.as_ref()) {
                            return Err(
                                (e.span, format!("This expression has type '{:?}' but is incompatible with '{:?}' (declared in the structure)",
                                e.static_ty,
                                ctx.all_fields[&lv.name]).to_string()).into()
                            );
                        }
                    }
                }
            },
            ExpVal::BinOp(op, a, b) => {
                fill_types(a, ctx)?;
                fill_types(b, ctx)?;

                match op {
                    BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Div | BinOp::Pow => {
                        // FIXME(Ryan): implement Display for BinOp.
                        if !is_any_or(&a, StaticType::Int64) {
                            return Err(
                                 (a.span, format!("No such operation '{:?}' for types '{:?}' and '{:?}'", op, a.static_ty, b.static_ty).to_string()).into()
                            ); 
                        }

                        if !is_any_or(&b, StaticType::Int64) {
                            return Err(
                                 (b.span, format!("No such operation '{:?}' for types '{:?}' and '{:?}'", op, a.static_ty, b.static_ty).to_string()).into()
                            ); 
                        }

                        expr.static_ty = Some(StaticType::Int64);
                    },
                    BinOp::Equ | BinOp::Neq => {
                        expr.static_ty = Some(StaticType::Bool);
                    },
                    BinOp::Lt | BinOp::Leq | BinOp::Gt | BinOp::Geq => {
                        let admissible_types = vec![StaticType::Int64, StaticType::Bool];

                        if !is_one_of_or_any(&a, &admissible_types) {
                           return Err(
                                 (a.span, format!("No such operation '{:?}' for types '{:?}' and '{:?}'", op, a.static_ty, b.static_ty).to_string()).into()
                           );
                        }
                        if !is_one_of_or_any(&b, &admissible_types) {
                           return Err(
                                 (b.span, format!("No such operation '{:?}' for types '{:?}' and '{:?}'", op, a.static_ty, b.static_ty).to_string()).into()
                           );
                        }

                        expr.static_ty = Some(StaticType::Bool);
                    },
                    BinOp::And | BinOp::Or => {
                        if !is_any_or(&a, StaticType::Bool) {
                           return Err(
                                 (a.span, format!("No such operation '{:?}' for types '{:?}' and '{:?}'", op, a.static_ty, b.static_ty).to_string()).into()
                           );
                        }

                        if !is_any_or(&b, StaticType::Bool) {
                           return Err(
                                 (b.span, format!("No such operation '{:?}' for types '{:?}' and '{:?}'", op, a.static_ty, b.static_ty).to_string()).into()
                           );
                        }

                        expr.static_ty = Some(StaticType::Bool);
                    }
                }
            },
            ExpVal::UnaryOp(op, e) => {
                fill_types(e, ctx)?;

                match op {
                    UnaryOp::Neg => {
                        if !is_any_or(&e, StaticType::Int64) {
                            return Err(
                                (e.span, format!("No such operation '{:?}' for type '{:?}'", op, e.static_ty).to_string()).into()
                            );
                        }
                        expr.static_ty = Some(StaticType::Int64);
                    },
                    UnaryOp::Not => {
                        if !is_any_or(&e, StaticType::Bool) {
                            return Err(
                                (e.span, format!("No such operation '{:?}' for type '{:?}'", op, e.static_ty).to_string()).into()
                            );
                        }
                        expr.static_ty = Some(StaticType::Bool);
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

                        fill_types(&mut args[0], ctx)?;
                        fill_types(&mut args[1], ctx)?;

                        if is_any_or(&args[0], StaticType::Int64) && is_any_or(&args[1], StaticType::Int64) {
                            expr.static_ty = Some(StaticType::Int64);
                        }
                    },
                    "print" => {
                        for arg in args {
                            fill_types(arg, ctx)?;
                        }

                        expr.static_ty = Some(StaticType::Nothing);
                    },
                    _ => {
                        if !is_builtin_function(name) && !ctx.structures.contains_key(name) && !ctx.functions.contains_key(name) {
                            return Err(
                                (expr.span, format!("There is no such function or structure named '{}'", name).to_string()).into()
                            );
                        }

                        let entity_types: Vec<Option<StaticType>>;
                        if ctx.structures.contains_key(name) {
                            entity_types = ctx.structures[name].fields.iter()
                                .map(|field| convert_to_static_type(field.ty.as_ref())).collect();
                        } else if ctx.functions.contains_key(name) && ctx.functions[name].len() == 1 {
                            entity_types = ctx.functions[name].first().unwrap().1.clone();
                        } else {
                            entity_types = vec![None; args.len()];
                        }

                        for (arg, expected_ty) in args.iter_mut().zip(entity_types.iter()) {
                            fill_types(arg, ctx)?;

                            if !is_compatible(arg.static_ty.as_ref(), expected_ty.as_ref()) {
                                return Err(
                                    (arg.span, format!("Incompatible types. Expected '{:?}', found '{:?}'", expected_ty, arg.static_ty).to_string()).into()
                                );
                            }
                        }

                        if let Some(ty) = get_unique_function_ret_type(&name, ctx) {
                            expr.static_ty = Some(ty);
                        } else {
                            expr.static_ty = Some(StaticType::Any);
                        }
                    }
                }
            },
            ExpVal::Int(_) => expr.static_ty = Some(StaticType::Int64),
            ExpVal::Str(_) => expr.static_ty = Some(StaticType::Str),
            ExpVal::Bool(_) => expr.static_ty = Some(StaticType::Bool),
            ExpVal::LValue(lv) => {
                match lv.in_exp.as_mut() {
                    None => {
                        if !ctx.environment.contains_key(&lv.name) {
                            return Err(
                                (lv.span, format!("No variable named '{}' is declared in this scope", &lv.name).to_string()).into()
                            );
                        }

                        expr.static_ty = ctx.environment[&lv.name].last().unwrap_or(&Some(StaticType::Any)).clone();
                    },
                    Some(e) => {
                        fill_types(e, ctx)?;
                    }
                }
            },
            ExpVal::Block(block) => {
                type_block(block, ctx)?;
                expr.static_ty = Some(StaticType::Any);
            },
            ExpVal::Mul(_, var) => {
                if !ctx.environment.contains_key(var) {
                    return Err(
                        (expr.span, format!("Undefined variable '{}'", var).to_string()).into());
                }
                // n*var: 3x
                expr.static_ty = Some(StaticType::Int64);
            },
            ExpVal::LMul(_, block) => {
                // a(block)
                type_block(block, ctx)?;
                expr.static_ty = Some(StaticType::Int64);
            },
            ExpVal::RMul(e, var) => {
                if !ctx.environment.contains_key(var) {
                    return Err(
                        (expr.span, format!("Undefined variable '{}'", var).to_string()).into());
                }
                // (expr)identfiant
                fill_types(e, ctx)?;
                expr.static_ty = Some(StaticType::Int64);
            },
            ExpVal::If(e, block, else_) => {
                fill_types(e, ctx)?;

                if !is_any_or(e, StaticType::Bool) {
                    return Err(
                        (e.span, format!("Non-boolean ({:?}) used in boolean context", e.static_ty).to_string()).into()
                    );
                }

                type_block(block, ctx)?;
                let ret_ty = type_else(else_, ctx)?;

                if block.static_ty != ret_ty {
                    expr.static_ty = Some(StaticType::Any);
                } else {
                    expr.static_ty = block.static_ty.clone();
                }
            },
            ExpVal::For(ident, range, block) => {
                fill_types(&mut range.start, ctx)?;
                fill_types(&mut range.end, ctx)?;

                let local_extra_vars = collect_all_assign_in_array(&block.val);
                for var in &local_extra_vars {
                    ctx.environment.entry(var.to_string()).or_default().push(None); // Shadow or inject it into the environment.
                }
                ctx.environment.entry(ident.name.clone()).or_default().push(Some(StaticType::Int64));

                type_block(block, ctx)?;

                for var in local_extra_vars {
                    ctx.environment.get_mut(&var).unwrap().pop();
                }
            },
            ExpVal::While(e, block) => {
                fill_types(e, ctx)?;

                if is_any_or(e, StaticType::Bool) {
                    let local_extra_vars = collect_all_assign_in_array(&block.val);

                    for var in &local_extra_vars {
                        ctx.environment.entry(var.to_string()).or_default().push(None);
                    }

                    type_block(block, ctx)?;

                    for var in local_extra_vars {
                        ctx.environment.get_mut(&var).unwrap().pop();
                    }

                    expr.static_ty = Some(StaticType::Nothing);
                } else {
                    return Err(
                        (e.span, format!("Non-boolean ({:?}) used in boolean context", e.static_ty).to_string()).into()
                    );
                }
            },
        }

        Ok(())
    }

    fill_types(toplevel, context)
}
