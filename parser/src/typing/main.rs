use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::*;
use super::data::*;
use super::expr::{type_expression, type_block};
use super::returns::verify_returns;

fn is_well_formed(t: Option<&LocatedIdent>, known_types: &HashSet<String>) -> bool {
    match t {
        None => true,
        Some(lident) => known_types.contains(&lident.name)
    }
}

fn is_reserved_name(n: &String) -> bool {
    match n.as_str() {
        "div" | "print" | "println" => true,
        _ => false
    }
}

fn to_env(known: &HashSet<String>) -> HashMap<String, Vec<Option<StaticType>>> {
    known.into_iter().map(|t| (t.clone(), vec![None])).collect()
}

pub fn static_type<'a>(decls: Vec<Decl<'a>>) -> TypingResult<'a> {
    // Step 1. Build the global environment.
    let mut structures: HashMap<String, Structure> = HashMap::new();
    let mut functions: HashMap<String, Vec<Function>> = HashMap::new();
    let mut function_sigs: HashMap<String, Vec<FuncSignature>> = HashMap::new();
    let mut all_fields: HashMap<String, Option<StaticType>> = HashMap::new();
    let mut mutable_fields: HashSet<String> = HashSet::new();
    let mut known_variables: HashSet<String> = HashSet::new();
    let mut known_types: HashSet<String> = ["Any", "Nothing", "Int64", "Bool", "String"].iter().cloned().map(|s| s.to_string()).collect();
    let mut global_exprs: Vec<Exp<'a>> = vec![];
    // Iterate over all declaration.
    for decl in decls {
        match decl.val {
            DeclVal::Structure(s) => {
                //  If it's a structure, check:
                //      (a) does it exist already?
                //      (b) check its fields names: unique across all the files.
                //      (c) check its types.

                if structures.contains_key(&s.name.name) {
                    return Err(
                        (s.span, format!("The ident '{}' is already taken by another structure", s.name.name).to_string()).into());
                }


                known_types.insert(s.name.name.clone());

                for field in &s.fields {
                    let fname = &field.name.name;
                    if all_fields.contains_key(fname) {
                        return Err(
                            (field.span, format!("The field name '{}' is already taken by this structure or another one", fname).to_string()).into()
                        );
                    }
                    if !is_well_formed(field.ty.as_ref(), &known_types) {
                        return Err(
                            (field.span, format!("This type is malformed, either it is not a primitive, or it's not this structure itself or another structure declared before").to_string()).into()
                        );
                    }

                    all_fields.insert(
                        fname.to_string().clone(),
                        convert_to_static_type(field.ty.as_ref())
                    );

                    if s.mutable {
                        mutable_fields.insert(fname.to_string().clone());
                    }
                }

                structures.insert(s.name.name.clone(), s);
            },
            DeclVal::Function(f) => {
                //  If it's a function, check:
                //      (a) is it a reserved name?
                //      (b) check its arguments names.
                //      (c) check if its own type and its arguments types are well formed.

                if is_reserved_name(&f.name) {
                    return Err(
                        (f.span, format!("Ident '{}' is a reserved name, it cannot be used as a function name", f.name).to_string()).into()
                    );
                }
                
                if !is_well_formed(f.ret_ty.as_ref(), &known_types) {
                    return Err((f.span, format!("The return type of '{}' is malformed, either it's not a primitive or a declared structure", f.name).to_string()).into());
                }

                let mut names: HashSet<String> = HashSet::new();

                for param in &f.params {
                    if names.contains(&param.name.name) {
                        return Err((param.span, format!("The ident '{}' is already taken by another argument", param.name.name).to_string()).into());
                    }

                    names.insert(param.name.name.clone());

                    if !is_well_formed(param.ty.as_ref(), &known_types) {
                        return Err(
                            (param.span, format!("This type is malformed, either it is not a primitive or it's not a declared before structure").to_string()).into()
                        );
                    }
                }

                // Iterate over all signatures to see whether there is already such a signature,
                // either ambiguously (None, Int64 vs Int64, None) or exact match.

                for sig in function_sigs.entry(f.name.clone()).or_default() {
                    if is_callable_with(&f, &sig) {
                        return Err(
                            (f.span, "This function is already defined or has ambiguous types which cannot be resolved at runtime".to_string()).into()
                        );
                    }
                }

                function_sigs.entry(f.name.clone()).or_default().push(build_signature(&f));
                functions.entry(f.name.clone()).or_default().push(f);
            },
            DeclVal::Exp(ge) => {
                //  If it's a global expression, check all Assign nodes and add them.
                known_variables.extend(collect_all_assign(&ge).into_iter());
                global_exprs.push(ge);
            }
        }
    }

    // Add nothing: Nothing in the future environment.
    let mut environment = to_env(&known_variables);
    environment
        .entry("nothing".to_string())
        .or_default()
        .push(Some(StaticType::Nothing));

    // Step 2.
    // Iterate over all declarations.
    // Looks like déjà vu. :>
    let mut global_ctx = TypingContext {
        functions: function_sigs,
        structures,
        known_types,
        mutable_fields,
        all_fields,
        environment
    };

    //  If it's an expression, type the expr in the global environment.
    for expr in &mut global_exprs {
        type_expression(expr, &mut global_ctx)?;
    }

    //  If it's a function, build a Γ environment shadowing the global one.
    //      Then, add all local variables inside of the block.
    //      Then, type the block.
    //      Then, if possible, check all the returns.
    for funcs in functions.values_mut() {
        for func in funcs {
            for arg in &func.params {
                global_ctx.environment
                    .entry(arg.name.name.clone())
                    .or_default()
                    .push(convert_to_static_type(arg.ty.as_ref()));
            }

            let extra_local_vars = collect_all_assign_in_array(&func.body.val);
            for var in &extra_local_vars {
                global_ctx.environment
                    .entry(var.clone())
                    .or_default()
                    .push(None);
            }

            type_block(&mut func.body, &mut global_ctx)?;

            if let Some(ret_ty) = convert_to_static_type(func.ret_ty.as_ref()) {
                // Implicit return.
                if !func.body.trailing_semicolon && func.body.val.len() > 0 && !is_compatible(func.body.val.last().unwrap().static_ty.as_ref(), Some(&ret_ty)) {
                    return Err(
                        (func.body.val.last().unwrap().span, format!("Invalid implicit return type, expected '{:?}', found '{:?}'",
                                                                     func.body.val.last().unwrap().static_ty,
                                                                     ret_ty).to_string()).into()
                    );
                }
                // Explicit returns.
                verify_returns(&func.body, ret_ty)?;
            }


            for arg in &func.params {
                global_ctx.environment.get_mut(&arg.name.name).unwrap().pop();
            }

            for var in extra_local_vars {
                global_ctx.environment.get_mut(&var).unwrap().pop();
            }
        }
    }


    // Returns the enriched declarations.
    Ok(TypedDecls {
        functions,
        structures: global_ctx.structures,
        global_expressions: global_exprs
    })
}
