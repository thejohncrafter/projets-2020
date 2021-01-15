use crate::ast::*;

// This visitor will consume and destroy its victim internally.
pub trait IntoVisitor<'a, T> {
    fn visit_decl(&mut self, d: Decl<'a>) -> T {
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

pub trait ExpressionVisitor<'a, T> {
    fn visit_expression(&mut self, e: &'a Exp<'a>) -> T;
}
