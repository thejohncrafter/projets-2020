use super::data::*;
use crate::ast::*;
use automata::line_counter::Span;

fn verify_return_type<'a>(span: Span<'a>, found: Option<&Exp<'a>>, expected: &StaticType) -> InternalTypingResult<'a> {
    match found {
        None => {
            if expected != &StaticType::Any && expected != &StaticType::Nothing {
                Err(
                    (span, format!("Mismatching return types, found nothing, expected: '{}'", expected).to_string()).into()
                )
            } else { Ok(()) }
        },
        Some(expr) => {
            if !is_compatible(expr.static_ty.as_ref(), expected) {
                Err(
                    (expr.span, format!("Mismatching return types, found: '{}', expected: '{}'", expr.static_ty, expected).to_string()).into()
                )
            } else {
                Ok(())
            }
        }
    }
}

fn walk_returns<'a>(

fn visit_returns<'a>(e: &Exp<'a>, expected: &StaticType) -> ReturnVerification<'a> {
    
    fn visit_else_returns<'a>(else_: &Else<'a>, expected: &StaticType) -> ReturnVerification<'a> {
        match else_.val.as_ref() {
            ElseVal::End => {},
            ElseVal::Else(b) => {
                for e in &b.val {
                    visit_returns(e, expected)?;
                }
            },
            ElseVal::ElseIf(e, b, rest_) => {
                visit_returns(e, expected)?;
                for x in &b.val {
                    visit_returns(x, expected)?;
                }
                visit_else_returns(&rest_, expected)?;
            }
        }

        Ok(())
    }

    match e.val.as_ref() {
        ExpVal::Return(r) => verify_return_type(e.span, r.as_ref(), expected),
        ExpVal::Assign(_, e) => visit_returns(e, expected),
        ExpVal::BinOp(_, a, b) => {
            visit_returns(&a, expected)?;
            visit_returns(&b, expected)?;

            Ok(())
        },
        ExpVal::UnaryOp(_, e) => visit_returns(e, expected),
        ExpVal::Call(_, e_s) => {
            for e in e_s {
                visit_returns(e, expected)?;
            }
            Ok(())
        },
        ExpVal::Block(b) | ExpVal::LMul(_, b) | ExpVal::For(_, _, b) => {
            for e in &b.val {
                visit_returns(e, expected)?;
            }
            Ok(())
        },
        ExpVal::RMul(e, _) => visit_returns(e, expected),
        ExpVal::If(e, b, else_branch) => {
            visit_returns(e, expected)?;
            for x in &b.val {
                visit_returns(x, expected)?;
            }
            visit_else_returns(else_branch, expected)?;
            Ok(())
        },
        ExpVal::While(e, b) => {
            visit_returns(e, expected)?;
            for x in &b.val {
                visit_returns(x, expected)?;
            }
            Ok(())
        },
        _ => Ok(())
    }
}

pub fn verify_returns<'a>(block: &Block<'a>, expected: StaticType) -> ReturnVerification<'a> {
    for e in &block.val {
        visit_returns(e, &expected)?;
    }

    Ok(())
}
