use super::visit::*;
use crate::ast::*;

/*fn convert_elseif_into_block<'a>(elseif: &Else<'a>) -> Exp<'a> {
    match elseif.val.as_ref() {
        ElseVal::End => panic!("Expected elseif, found END during conversion into block"),
        ElseVal::Else(b) => panic!("Expected elseif, found else during conversion into block"),
        ElseVal::ElseIf(eif_cond, eif_block, eif_else) => Exp::new(
            elseif.span,
            ExpVal::Block(Block::new(elseif.span, vec![Exp::new(
                elseif.span,
                // FIXME(Ryan): this looks like expensive.
                ExpVal::If(eif_cond.clone(), eif_block.clone(), eif_else.clone())
            )], false))
        )
    }
}*/

pub struct GenericResultLeafExpressionVisitor<'a, T1, T2>
{
    pub leaf_visitor: Box<dyn LeafVisitor<'a, Result<T1, T2>> + 'a>,
}

impl<'a, T1, T2> ExpressionVisitor<'a, Result<T1, T2>> for GenericResultLeafExpressionVisitor<'a, T1, T2>
{
    fn visit_expression(&mut self, p: &'a mut Exp<'a>) -> Result<T1, T2> {
        match p.val.as_mut() {
            ExpVal::Return(me) => {
                if let Some(e) = me {
                    self.visit_expression(e)?;
                }

                self.leaf_visitor.visit_return(p)
            },
            ExpVal::Assign(lv, e) => {
                self.visit_expression(e)?;

                self.leaf_visitor.visit_assign(p)
            },
            ExpVal::BinOp(bop, a, b) => {
                self.visit_expression(a)?;
                self.visit_expression(b)?;

                self.leaf_visitor.visit_bin_op(p)
            },
            ExpVal::UnaryOp(uop, e) => {
                self.visit_expression(e)?;

                self.leaf_visitor.visit_unary_op(p)
            },
            ExpVal::Call(name, e_s) => {
                for e in e_s {
                    self.visit_expression(e)?;
                }

                self.leaf_visitor.visit_call(p)
            },
            ExpVal::Block(b) => {
                for e in &mut b.val {
                    self.visit_expression(e)?;
                }

                self.leaf_visitor.visit_block(p)
            },
            ExpVal::Int(_) => self.leaf_visitor.visit_int(p),
            ExpVal::Str(_) => self.leaf_visitor.visit_str(p),
            ExpVal::Bool(_) => self.leaf_visitor.visit_bool(p),
            ExpVal::LValue(lv) => {
                if let Some(in_exp) = &mut lv.in_exp {
                    self.visit_expression(in_exp)?;
                }

                self.leaf_visitor.visit_lvalue(p)
            },
            ExpVal::Mul(c, v) => self.leaf_visitor.visit_left_arith_var(p),
            ExpVal::LMul(c, b) => {
                for e in &mut b.val {
                    self.visit_expression(e);
                }

                self.leaf_visitor.visit_left_arith_block(p)
            },
            ExpVal::RMul(e, v) => {
                self.visit_expression(e)?;

                self.leaf_visitor.visit_right_arith_expr(p)
            },
            ExpVal::For(counter, range, b) => {
                for e in &mut b.val {
                    self.visit_expression(e);
                }

                self.leaf_visitor.visit_for(p)
            },
            ExpVal::If(e, b, e_branch) => {
                self.visit_expression(e)?;

                for e_ in &mut b.val {
                    self.visit_expression(e_)?;
                }

                let ret = self.leaf_visitor.visit_if_branch(p)?;

                if let ElseVal::Else(e_block) = e_branch.val.as_mut() {
                    for e_ in &mut e_block.val {
                        self.visit_expression(e_)?;
                    }

                    self.leaf_visitor.visit_else_branch(p)
                } else if let ElseVal::ElseIf(eif_cond, eif_block, eif_else) = e_branch.val.as_mut() {
                    // else if is only else { if (…) }
                    // we build an ad-hoc expr for that.
                    // FIXME: that is really incorrect.
                    //let adhoc_expr: Exp = convert_elseif_into_block(e_branch);
                    //self.visit_expression(&mut adhoc_expr)?;
                    //self.leaf_visitor.visit_else_branch(&mut adhoc_expr)
                    //Ok(T1::default())
                    Ok(ret)
                } else {
                    Ok(ret)
                }
            },
            ExpVal::While(e, b) => {
                self.visit_expression(e)?;

                for e_ in &mut b.val {
                    self.visit_expression(e_)?;
                }

                self.leaf_visitor.visit_while(p)
            },
        }
    }
}
