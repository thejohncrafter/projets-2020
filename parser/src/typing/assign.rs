use crate::ast::*;

pub type AssignationList<'a> = Vec<LocatedIdent<'a>>;

pub fn collect_all_assign_in_array<'a>(a: &Vec<Exp<'a>>) -> AssignationList<'a> {
    a.iter().flat_map(collect_all_assign).collect()
}

pub fn collect_all_assign<'a>(e: &Exp<'a>) -> AssignationList<'a> {
    fn collect_else<'a>(u: &Else<'a>) -> AssignationList<'a> {
        match u.val.as_ref() {
            ElseVal::End => vec![],
            ElseVal::Else(b) => collect_all_assign_in_array(&b.val),
            ElseVal::ElseIf(e, b, rest_) => collect_all_assign(&e)
                .into_iter()
                .chain(collect_all_assign_in_array(&b.val).into_iter())
                .chain(collect_else(&rest_).into_iter())
                .collect()
        }
    }

    // Perform a DFS on e to smoke out all Assign
    match e.val.as_ref() {
        ExpVal::Return(e) => match e {
            None => vec![],
            Some(e) => collect_all_assign(&e)
        },
        ExpVal::Assign(lv, e) => {
            let mut assigns = collect_all_assign(&e);
            match lv.in_exp {
                None => assigns.push(LocatedIdent::new(
                        lv.span,
                        lv.name.clone())),
                _ => {}
            };
            assigns
        },
        ExpVal::BinOp(_, alpha, beta) => collect_all_assign(&alpha)
            .into_iter()
            .chain(collect_all_assign(&beta).into_iter())
            .collect(),
        ExpVal::UnaryOp(_, e) => collect_all_assign(&e),
        ExpVal::Call(_, e_s) => collect_all_assign_in_array(&e_s),
        ExpVal::Block(b) | ExpVal::LMul(_, b) => collect_all_assign_in_array(&b.val),
        ExpVal::RMul(e, _) => collect_all_assign(&e),
        ExpVal::If(e, b, else_branch) => collect_all_assign(&e)
            .into_iter()
            .chain(collect_all_assign_in_array(&b.val).into_iter())
            .chain(collect_else(&else_branch).into_iter())
            .collect(),
        ExpVal::For(_, _, _) | ExpVal::While(_, _) => vec![], 
        ExpVal::Int(_) | ExpVal::Str(_) | ExpVal::Bool(_) | ExpVal::Mul(_, _) => vec![],
        ExpVal::LValue(lv) => {
            match &lv.in_exp {
                None => vec![],
                Some(e) => collect_all_assign(e)
            }
        }
    }
}

