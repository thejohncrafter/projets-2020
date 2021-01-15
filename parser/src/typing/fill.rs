use super::data::*;
use crate::ast::*;

use super::visit::{MutableLeafVisitor, MutableExpressionVisitor};
use super::collect::GenericResultLeafMutableExpressionVisitor;
use automata::read_error::ReadError;

fn build_fill_visitor<'a>(tcx: &'a mut TypingContext<'a>) -> GenericResultLeafMutableExpressionVisitor<'a, (), ReadError<'a>> {
    GenericResultLeafMutableExpressionVisitor {
        leaf_visitor: Box::new(FillVisitor {
            tcx,
            cur: None
        })
    }
}

pub fn type_expression<'a>(tcx: &'a mut TypingContext<'a>, expr: &'a mut Exp<'a>) -> InternalTypingResult<'a> {
    build_fill_visitor(tcx).visit_expression(expr)
}

pub fn type_block<'a>(tcx: &'a mut TypingContext<'a>, block: &'a mut Block<'a>) -> InternalTypingResult<'a> {
    let mut visitor = build_fill_visitor(tcx);

    for expr in &mut block.val {
        visitor.visit_expression(expr)?;
    }

    Ok(())
}

struct FillVisitor<'a, 'b> {
    tcx: &'b mut TypingContext<'a>,
    cur: Option<&'a mut Exp<'a>>,
}


impl<'a, 'b> MutableLeafVisitor<'a, InternalTypingResult<'a>> for FillVisitor<'a, 'b> {
    fn set_current_node(&mut self, c: &'a mut Exp<'a>) {
        self.cur = Some(c);
    }

    fn visit_return(&mut self, r: &mut Option<Exp>) -> InternalTypingResult<'a> { Ok(()) }

    fn visit_assign(&mut self, lv: &'a mut LValue<'a>, e: &mut Exp) -> InternalTypingResult<'a> {
        match &lv.in_exp {
            None => {
                if !self.tcx.is_alive_in_env(&LocatedIdent::new(lv.span, lv.name.clone())) {
                    return Err(
                        (lv.span, format!("Compiler error, '{}' was not found in the global typing context, unreachable variable. Environment was {:?}", &lv.name, self.tcx.environment).to_string()).into());
                }

                Ok(())
            },
            Some(prefix_e) => {
                Ok(())
            }
        }
    }

    fn visit_bin_op(&mut self, bop: &mut BinOp, a: &mut Exp, b: &mut Exp) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_unary_op(&mut self, u: &mut UnaryOp, e: &mut Exp) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_call(&mut self, name: &mut String, block: &mut Vec<Exp>) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_int(&mut self, cst: &mut i64) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_str(&mut self, cst: &mut String) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_bool(&mut self, cst: &mut bool) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_lvalue(&mut self, lv: &mut LValue) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_block(&mut self, block: &mut Block) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_left_arith_var(&mut self, cst: &mut i64, v: &mut String) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_left_arith_block(&mut self, cst: &mut i64, v: &mut Block) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_right_arith_expr(&mut self, e: &mut Exp, v: &mut String) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_if_branch(&mut self, condition: &mut Exp, then: &mut Block, other: &mut Else) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_else_branch(&mut self, body: &mut Block) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_for(&mut self, counter: &mut LocatedIdent, range: &mut Range, body: &mut Block) -> InternalTypingResult<'a> { Ok(()) }
    fn visit_while(&mut self, condition: &mut Exp, body: &mut Block) -> InternalTypingResult<'a> { Ok(()) }
}
