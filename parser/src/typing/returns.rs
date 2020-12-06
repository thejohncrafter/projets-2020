use super::data::*;
use crate::ast::*;

fn verify_return_type<'a>(found: &Exp<'a>, expected: &StaticType) -> ReturnVerification<'a> {
    if !is_compatible(found.static_ty.as_ref(), Some(expected)) {
        Err(
            (found.span, format!("Mismatching return types, found: '{:?}', expected: '{}'", found.static_ty, expected).to_string()).into()
        )
    } else {
        Ok(())
    }
}

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
        ExpVal::Return(e) => verify_return_type(e, expected),
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
