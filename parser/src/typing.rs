use std::collections::HashMap;
use std::collections::HashSet;
use super::ast::*;

use automata::read_error::ReadError;

type TypingResult<'a> = Result<Vec<Decl<'a>>, ReadError<'a>>;

fn is_compatible(alpha: StaticType, beta: StaticType) -> bool {
    return alpha == StaticType::Any || beta == StaticType::Any || alpha == beta;
}

fn is_any_or<'a>(alpha: Exp<'a>, t: StaticType) -> bool {
    return alpha.static_ty == Some(StaticType::Any) || alpha.static_ty == Some(t);
}

fn is_one_of_or_any<'a>(alpha: Exp<'a>, ts: Vec<StaticType>) -> bool {
    if alpha.static_ty == Some(StaticType::Any) {
        return true;
    }

    for t in ts {
        if alpha.static_ty == Some(t) {
            return true;
        }
    }
    
    return false;
}

fn is_well_formed(t: Option<&LocatedIdent>, known_types: &HashSet<String>) -> bool {
    match t {
        None => true,
        Some(lident) => known_types.contains(&lident.name)
    }
}

fn is_reserved_name(n: String) -> bool {
    match n.as_str() {
        "div" | "print" | "println" => true,
        _ => false
    }
}

fn collect_all_assign<'a>(e: &Exp<'a>) -> Vec<String> {
    fn collect_else<'a>(u: &Else<'a>) -> Vec<String> {
        match &*u.val {
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
    match &*e.val {
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
        ExpVal::For(lident, range, b) => collect_array(&b.val),
        ExpVal::While(e, b) => collect_all_assign(&e)
            .into_iter()
            .chain(collect_array(&b.val).into_iter())
            .collect(),
        _ => vec![] // Default case: no assignations can be hidden here.
    }
}

pub fn static_type<'a>(decls: Vec<Decl<'a>>) -> TypingResult<'a> {
    // Step 1. Build the global environment.
    let mut structures: HashMap<String, &Structure> = HashMap::new();
    let mut functions: HashMap<String, &Function> = HashMap::new();
    let mut overall_fields: HashSet<String> = HashSet::new();
    let mut known_types: HashSet<String> = ["Any", "Nothing", "Int64", "Bool", "String"].iter().cloned().map(|s| s.to_string()).collect();
    // Iterate over all declaration.
    for decl in &decls {
        match &decl.val {
            DeclVal::Structure(s) => {
                //  If it's a structure, check:
                //      (a) does it exist already?
                //      (b) check its fields names: unique across all the files.
                //      (c) check its types.

                if structures.contains_key(&s.name.name) {
                    return Err(
                        (s.span, format!("The ident '{}' is already taken by another structure", s.name.name).to_string()).into());
                }


                structures.insert(s.name.name.clone(), s);
                known_types.insert(s.name.name.clone());

                for field in &s.fields {
                    let fname = &field.name.name;
                    if overall_fields.contains(fname) {
                        return Err(
                            (field.span, format!("The field name '{}' is already taken by this structure or another one", fname).to_string()).into()
                        );
                    }
                    if !is_well_formed(field.ty.as_ref(), &known_types) {
                        return Err(
                            (field.span, format!("This type is malformed, either it is not a primitive, or it's not this structure itself or another structure declared before").to_string()).into()
                        );
                    }

                    overall_fields.insert(fname.to_string().clone());
                }
            },
            DeclVal::Function(f) => {
                //  If it's a function, check:
                //      (a) is it a reserved name?
                //      (b) check its arguments names.
                //      (c) check if its own type and its arguments types are well formed.
                
                if functions.contains_key(&f.name) {
                    return Err((f.span, format!("The ident '{}' is already taken by another function", f.name).to_string()).into());
                }

                functions.insert(f.name.clone(), f);

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
            },
            DeclVal::Exp(ge) => {
                //  If it's a global expression, check all Assign nodes and add them.
                let assigns = collect_all_assign(ge);
                println!("Assignations: {:?}", assigns);
                println!("---");
            }
        }
    }

    // Step 2.
    // Iterate over all declarations.
    // Looks like déjà vu. :>
    for decl in &decls {
        //  If it's a function, build a Γ environment shadowing the global one.
        //      Then, add all local variables inside of the block.
        //      Then, type the block.
        //  If it's an expression, type the expr in the global environment.
        match &decl.val {
            DeclVal::Function(f) => {
                //type_block(f.block, build_context(global_ctx, f.params));
            },
            DeclVal::Exp(ge) => {
                //type_expr(ge, global_ctx);
            }
            _ => {}
        }
    }

    // Returns the enriched declarations.
    Ok(decls)
}
