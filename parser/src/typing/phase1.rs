use std::collections::HashSet;
use crate::ast::{Structure, Function, Exp, StaticType};
use super::data::*;
use super::visit::IntoVisitor;
use super::assign::collect_all_assign;
use super::func_signatures::{build_signature, is_callable_with_exactly, format_signature};

fn is_reserved_name(n: &String) -> bool {
    match n.as_str() {
        "div" | "print" | "println" => true,
        _ => false
    }
}

impl<'a> IntoVisitor<'a, InternalTypingResult<'a>> for GlobalEnvironmentState<'a> {
    fn visit_structure(&mut self, s: Structure<'a>) -> InternalTypingResult<'a> {
        //  If it's a structure, check:
        //      (a) does it exist already?
        //      (b) check its fields names: unique across all the files.
        //      (c) check its types.

        if self.structures.contains_key(&s.name.name) {
            return Err(
                (s.span, format!("The ident '{}' is already taken by another structure", s.name.name).to_string()).into());
        }
        self.known_types.insert(StaticType::Struct(s.name.name.clone()));

        for field in &s.fields {
            let fname = &field.name.name;
            if self.all_structure_fields.contains_key(fname) {
                return Err(
                    (field.span, format!("The field name '{}' is already taken by this structure or another one", fname).to_string()).into()
                );
            }
            if !self.known_types.contains(&field.ty) {
                return Err(
                    (field.span, format!("This type is malformed, either it is not a primitive, or it's not this structure itself or another structure declared before").to_string()).into()
                );
            }

            self.all_structure_fields.insert(
                fname.to_string().clone(),
                field.ty.clone()
            );

            if s.mutable {
                self.all_mutable_fields.insert(fname.to_string().clone());
            }
        }

        self.structures.insert(s.name.name.clone(), s);
        Ok(())
    }

    fn visit_function(&mut self, f: Function<'a>) -> InternalTypingResult<'a> {
        //  If it's a function, check:
        //      (a) is it a reserved name?
        //      (b) check its arguments names.
        //      (c) check if its own type and its arguments types are well formed.

        if is_reserved_name(&f.name) {
            return Err(
                (f.span, format!("The ident '{}' is a reserved name, it cannot be used as a function name", f.name).to_string()).into()
            );
        }
        
        if !self.known_types.contains(&f.ret_ty) {
            return Err((f.span, format!("The return type '{}' of '{}' is malformed, either it's not a primitive or a declared structure", f.ret_ty, f.name).to_string()).into());
        }

        let mut names: HashSet<String> = HashSet::new();

        for param in &f.params {
            if names.contains(&param.name.name) {
                return Err((param.span, format!("The ident '{}' is already taken by another argument", param.name.name).to_string()).into());
            }

            names.insert(param.name.name.clone());

            if !self.known_types.contains(&param.ty) {
                return Err(
                    (param.span, format!("This type is malformed, either it is not a primitive or it's not a declared before structure").to_string()).into()
                );
            }
        }

        // Iterate over all signatures to see whether there is already such a signature,
        // either ambiguously (None, Int64 vs Int64, None) or exact match.
        // FIXME: this is ugly.
        for sig in self.function_sigs.entry(f.name.clone()).or_default() {
            if is_callable_with_exactly(f.params.iter().map(|arg| arg.ty.clone()).collect(), &sig) {
                return Err(
                    (f.span, format!(
                            "The function '{}' has already been defined with the exact same signature ({}), add type annotations to disambiguate or remove duplicates",
                            f.name,
                            format_signature(f.params.into_iter().map(|arg| arg.ty).collect())
                    ).to_string()).into()
                );
            }
        }

        self.function_sigs.entry(f.name.clone()).or_default().push(build_signature(&f));
        self.functions.entry(f.name.clone()).or_default().push(f);

        Ok(())
    }

    fn visit_expression(&mut self, ge: Exp<'a>) -> InternalTypingResult<'a> {
        //  If it's a global expression, check all Assign nodes and add them.
        self.global_variables.extend(collect_all_assign(&ge).into_iter().map(|l_ident| l_ident.name));
        self.global_expressions.push(ge);

        Ok(())
    }
}


