use super::data::*;
use crate::ast::*;

fn is_compatible(alpha: Option<&StaticType>, beta: Option<&StaticType>) -> bool {
    match (alpha, beta) {
        (None, _) | (_, None) => true,
        (Some(a), Some(b)) => *a == StaticType::Any || *b == StaticType::Any || *a == *b
    }
}

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

pub fn type_expression<'a, 'b>(toplevel: &'b mut Exp<'a>, context: &'b mut TypingContext<'a>) -> ExprTypingResult<'a> {

    fn fill_types<'a, 'b>(expr: &'b mut Exp<'a>, ctx: &'b mut TypingContext<'a>) -> ExprTypingResult<'a> {
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
                                (lv.span, format!("Compiler error, '{}' was not found in the global typing context, unreachable variable.", &lv.name).to_string()).into()
                            );
                        }

                        if !is_compatible(ctx.environment[&lv.name].as_ref(),
                            e.static_ty.as_ref()) {
                            return Err(
                                (e.span, format!("This expression has type '{:?}' but is incompatible with '{:?}' (expected)", e.static_ty, ctx.environment[&lv.name]).to_string()).into()
                            );
                        }
                    },
                    Some(prefix_e) => {
                        fill_types(prefix_e, ctx)?;

                        // If prefix_e is known, we can check if the field exist.
                        if let Some(st) = &prefix_e.static_ty {
                        }

                        if !ctx.mutable_fields.contains(&lv.name) {
                            return Err(
                                (lv.span, format!("Field '{}' is not contained in a mutable structure, it cannot be assigned", &lv.name).to_string()).into()
                            );
                        }

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
                        if is_any_or(&a, StaticType::Int64) && is_any_or(&b, StaticType::Int64) {
                            expr.static_ty = Some(StaticType::Int64);
                        }
                    },
                    BinOp::Equ | BinOp::Neq => {
                        expr.static_ty = Some(StaticType::Bool);
                    },
                    BinOp::Lt | BinOp::Leq | BinOp::Gt | BinOp::Geq => {
                        let admissible_types = vec![StaticType::Int64, StaticType::Bool];

                        if is_one_of_or_any(&a, &admissible_types) && is_one_of_or_any(&b, &admissible_types) {
                            expr.static_ty = Some(StaticType::Bool);
                        }
                    },
                    BinOp::And | BinOp::Or => {
                        if is_any_or(&a, StaticType::Bool) && is_any_or(&b, StaticType::Bool) {
                            expr.static_ty = Some(StaticType::Bool);
                        }
                    }
                }
            },
            _ => expr.static_ty = Some(StaticType::Any)
        }

        Ok(())
    }

    fill_types(toplevel, context)
}
