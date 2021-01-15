use crate::ast::*;

// This visitor will consume and destroy its victim internally.
pub trait IntoVisitor<'a, T> {
    fn visit_decl(&mut self, d: &Decl<'a>) -> T {
        match d.val {
            DeclVal::Structure(s) => self.visit_structure(s),
            DeclVal::Function(f) => self.visit_function(f),
            DeclVal::Exp(e) => self.visit_expression(e)
        }
    }

    fn visit_function(&mut self, f: Function<'a>) -> T;
    fn visit_structure(&mut self, s: Structure<'a>) -> T;
    fn visit_expression(&mut self, e: Exp<'a>) -> T;
}

pub trait MutableExpressionVisitor<'a, T> {
    fn visit_expression<'b>(&mut self, e: &'b mut Exp<'a>) -> T;
}

pub trait MutableLeafVisitor<'a, 'b, T> {
    fn set_current_node(&mut self, c: &'b mut Exp<'a>);

    fn visit_return(&mut self, r: &mut Option<Exp>) -> T;
    fn visit_assign(&mut self, lv: &'a mut LValue<'a>, e: &mut Exp) -> T;
    fn visit_bin_op(&mut self, bop: &mut BinOp, a: &mut Exp, b: &mut Exp) -> T;
    fn visit_unary_op(&mut self, u: &mut UnaryOp, e: &mut Exp) -> T;
    fn visit_call(&mut self, name: &mut String, block: &mut Vec<Exp>) -> T;
    fn visit_int(&mut self, cst: &mut i64) -> T;
    fn visit_str(&mut self, cst: &mut String) -> T;
    fn visit_bool(&mut self, cst: &mut bool) -> T;
    fn visit_lvalue(&mut self, lv: &mut LValue) -> T;
    fn visit_block(&mut self, block: &mut Block) -> T;
    fn visit_left_arith_var(&mut self, cst: &mut i64, v: &mut String) -> T;
    fn visit_left_arith_block(&mut self, cst: &mut i64, v: &mut Block) -> T;
    fn visit_right_arith_expr(&mut self, e: &mut Exp, v: &mut String) -> T;
    fn visit_if_branch(&mut self, condition: &mut Exp, then: &mut Block, other: &mut Else) -> T;
    fn visit_else_branch(&mut self, body: &mut Block) -> T;
    fn visit_for(&mut self, counter: &mut LocatedIdent, range: &mut Range, body: &mut Block) -> T;
    fn visit_while(&mut self, condition: &mut Exp, body: &mut Block) -> T;
}
