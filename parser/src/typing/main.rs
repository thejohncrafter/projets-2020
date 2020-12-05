use std::collections::HashMap;
use std::collections::HashSet;
use crate::ast::*;
use super::data::*;
use super::expr::type_expression;

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

fn convert_to_static_type(p: Option<&LocatedIdent>) -> Option<StaticType> {
    match p {
        None => None,
        Some(lident) => {
            Some(match lident.name.as_str() {
                "Any" => StaticType::Any,
                "Nothing" => StaticType::Nothing,
                "Int64" => StaticType::Int64,
                "Bool" => StaticType::Bool,
                "String" => StaticType::Str,
                _ => StaticType::Struct(lident.name.clone())
            })
        }
    }
}

fn collect_all_assign<'a>(e: &Exp<'a>) -> Vec<String> {
    fn collect_else<'a>(u: &Else<'a>) -> Vec<String> {
        match u.val.as_ref() {
            ElseVal::End => vec![],
            ElseVal::Else(b) => collect_array(&b.val),
            ElseVal::ElseIf(e, b, rest_) => collect_all_assign(&e)
                .into_iter()
                .chain(collect_array(&b.val).into_iter())
                .chain(collect_else(&rest_).into_iter())
                .collect()
        }
    }

    fn collect_array<'a>(a: &Vec<Exp<'a>>) -> Vec<String> {
        a.iter().flat_map(collect_all_assign).collect()
    }

    // Perform a DFS on e to smoke out all Assign
    match e.val.as_ref() {
        ExpVal::Return(e) => collect_all_assign(&e),
        ExpVal::Assign(lv, e) => {
            let mut assigns = collect_all_assign(&e);
            match lv.in_exp {
                None => assigns.push(lv.name.clone()),
                _ => {}
            };
            assigns
        },
        ExpVal::BinOp(_, alpha, beta) => collect_all_assign(&alpha)
            .into_iter()
            .chain(collect_all_assign(&beta).into_iter())
            .collect(),
        ExpVal::UnaryOp(_, e) => collect_all_assign(&e),
        ExpVal::Call(_, e_s) => collect_array(&e_s),
        ExpVal::Block(b) | ExpVal::LMul(_, b) => collect_array(&b.val),
        ExpVal::RMul(e, _) => collect_all_assign(&e),
        ExpVal::If(e, b, else_branch) => collect_all_assign(&e)
            .into_iter()
            .chain(collect_array(&b.val).into_iter())
            .chain(collect_else(&else_branch).into_iter())
            .collect(),
        ExpVal::For(_lident, _range, b) => collect_array(&b.val),
        ExpVal::While(e, b) => collect_all_assign(&e)
            .into_iter()
            .chain(collect_array(&b.val).into_iter())
            .collect(),
        _ => vec![] // Default case: no assignations can be hidden here.
    }
}

fn to_env(known: &HashSet<String>) -> HashMap<String, Option<StaticType>> {
    known.into_iter().map(|t| (t.clone(), None)).collect()
}

pub fn static_type<'a>(decls: Vec<Decl<'a>>) -> TypingResult<'a> {
    // Step 1. Build the global environment.
    let mut structures: HashMap<String, Structure> = HashMap::new();
    let mut functions: HashMap<String, Function> = HashMap::new();
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
                
                if functions.contains_key(&f.name) {
                    return Err((f.span, format!("The ident '{}' is already taken by another function", f.name).to_string()).into());
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

                functions.insert(f.name.clone(), f);
            },
            DeclVal::Exp(ge) => {
                //  If it's a global expression, check all Assign nodes and add them.
                known_variables.extend(collect_all_assign(&ge).into_iter());
                global_exprs.push(ge);
            }
        }
    }

    println!("Assignations: {:?}", known_variables);
    println!("---");
    // Step 2.
    // Iterate over all declarations.
    // Looks like déjà vu. :>
    let mut global_ctx = TypingContext {
        functions,
        structures,
        known_types,
        mutable_fields,
        all_fields,
        environment: to_env(&known_variables)
    };
        //  If it's a function, build a Γ environment shadowing the global one.
        //      Then, add all local variables inside of the block.
        //      Then, type the block.

    //  If it's an expression, type the expr in the global environment.
    for expr in &mut global_exprs {
        type_expression(expr, &mut global_ctx)?;
    }

    // Returns the enriched declarations.
    Ok(TypedDecls {
        functions: global_ctx.functions,
        structures: global_ctx.structures,
        global_expressions: global_exprs
    })
}
