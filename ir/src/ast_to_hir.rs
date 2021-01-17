use std::collections::HashSet;
use std::collections::HashMap;
use std::iter::once;

use super::hir::types as hir;
use super::error::*;

use parser::ast::*;
use parser::typing::data::TypedDecls;
use parser::typing::assign::collect_all_assign_in_array;

pub type HIRValueResult = Result<(Vec<hir::Statement>, hir::Val), Error>;
pub type HIRMultiValuesResult = Result<(Vec<hir::Statement>, Vec<hir::Val>), Error>;
pub type HIRStatementsResult = Result<Vec<hir::Statement>, Error>;
pub type HIRBlockResult = Result<hir::Block, Error>;
pub type HIRFunctionResult = Result<hir::Function, Error>;
pub type HIREntrypointResult = Result<(Vec<String>, hir::Function), Error>; // Pair (globals, function)
pub type HIRStructDeclResult = Result<hir::StructDecl, Error>;
pub type HIRDeclsResult = Result<Vec<hir::Decl>, Error>;
pub type HIRSourceResult = Result<hir::Source, Error>;

fn from_static_type(s: StaticType) -> Option<hir::Type> {
    match s {
        StaticType::Any => None,
        StaticType::Nothing => Some(hir::Type::Nothing),
        StaticType::Int64 => Some(hir::Type::Int64),
        StaticType::Bool => Some(hir::Type::Bool),
        StaticType::Str => Some(hir::Type::Str),
        StaticType::Struct(s) => Some(hir::Type::Struct(s))
    }
}

struct Emitter {
    pub next_intermediate_variable_id: u64,
    pub current_local_vars: HashSet<String>,
    pub current_params: HashSet<String>,
    pub old_global_vars: HashMap<String, String>, // original name → renamed name
    pub global_vars: HashSet<String>, // renamed names.
    pub structure_names: HashSet<String>
}

impl Emitter {
    fn init(st_names: HashSet<String>) -> Self {
        Emitter { next_intermediate_variable_id: 0,
            current_local_vars: HashSet::new(),
            global_vars: HashSet::new(),
            old_global_vars: HashMap::new(),
            current_params: HashSet::new(),
            structure_names: st_names
        }
    }

    fn mk_intermediate_var(&mut self) -> String {
        let mut out = format!("__intermediate_internal{}", self.next_intermediate_variable_id);
        while self.global_vars.contains(&out) {
            self.next_intermediate_variable_id += 1;
            out = format!("__intermediate_internal{}", self.next_intermediate_variable_id);
        }
        self.current_local_vars.insert(out.clone());
        self.next_intermediate_variable_id += 1;
        out
    }

    fn emit_unique_gvar_name(&mut self, gvar: &String) -> String {
        let mut out = format!("_g{}", gvar);
        let mut idx = 0;

        while self.global_vars.contains(&out) {
            out = format!("_g{}{}", gvar, idx);
            idx += 1;
        }

        out
    }

    fn emit_block_value(&mut self, b: &Block) -> HIRValueResult {
        // the value of a block is the value of its last statement.
        if let Some((last, head)) = b.val.split_last() {
            let mut stmts = self.emit_flattened_statements(head)?;
            let (last_stmts, last_val) = self.emit_value(last)?;
            stmts.extend(last_stmts);

            Ok((stmts, last_val))
        } else {
            Ok((vec![], hir::Val::Nothing))
        }
    }

    fn emit_value(&mut self, e: &Exp) -> HIRValueResult {
        match e.val.as_ref() {
            ExpVal::BinOp(op, a, b) => {
                let (stmts_a, val_a) = self.emit_value(a)?;
                let (stmts_b, val_b) = self.emit_value(b)?;
                let mut stmts = stmts_a.into_iter().chain(stmts_b).collect::<Vec<_>>();
                let out = self.mk_intermediate_var();

                enum NativeOrSoft {
                    Native(hir::BinOp),
                    Soft(String),
                }

                use NativeOrSoft::*;

                let action = match op {
                    BinOp::And => Native(hir::BinOp::And),
                    BinOp::Or => Native(hir::BinOp::Or),
                    BinOp::Equ => Native(hir::BinOp::Equ),
                    BinOp::Neq => Native(hir::BinOp::Neq),
                    BinOp::Lt => Native(hir::BinOp::Lt),
                    BinOp::Leq => Native(hir::BinOp::Leq),
                    BinOp::Gt => Native(hir::BinOp::Gt),
                    BinOp::Geq => Native(hir::BinOp::Geq),
                    BinOp::Plus => Native(hir::BinOp::Add),
                    BinOp::Minus => Native(hir::BinOp::Sub),
                    BinOp::Times => Native(hir::BinOp::Mul),
                    BinOp::Div => Native(hir::BinOp::Div),
                    BinOp::Pow => Soft("pow".to_string()),
                };
                
                let callable = match action {
                    Native(op) => {
                        hir::Callable::Bin(op, val_a, val_b)
                    },
                    Soft(fn_name) => {
                        hir::Callable::Call(fn_name, true, vec!(val_a, val_b))
                    },
                };

                stmts.push(hir::Statement::Call(hir::LValue::Var(out.clone()), callable));

                Ok((stmts, hir::Val::Var(out)))
            },
            ExpVal::Block(block) => self.emit_block_value(block),
            ExpVal::UnaryOp(op, e) => {
                let (mut stmts, val) = self.emit_value(e)?;
                let out = self.mk_intermediate_var();

                stmts.push(hir::Statement::Call(
                        hir::LValue::Var(out.clone()),
                        hir::Callable::Unary(
                            match op {
                                UnaryOp::Neg => hir::UnaryOp::Neg,
                                UnaryOp::Not => hir::UnaryOp::Not,
                            },
                            val
                        )
                    ));

                Ok((stmts, hir::Val::Var(out)))
            },
            ExpVal::Int(cst) => Ok((vec![], hir::Val::Const(hir::Type::Int64, *cst as u64))),
            ExpVal::Bool(cst) => Ok((vec![], hir::Val::Const(hir::Type::Bool, u64::from(*cst)))),
            ExpVal::Str(cst) => Ok((vec![], hir::Val::Str(cst.clone()))),
            ExpVal::LValue(lv) => {
                match lv.in_exp.as_ref() {
                    None => {
                        if lv.scope == Scope::Local {
                            Ok((vec![], hir::Val::Var(lv.name.clone())))
                        } else {
                            match self.old_global_vars.get(&lv.name) {
                                None => Err(
                                    format!("[T-AST] Unexpected error, lvalue of type '{}' is scoped globally but no global variables of name '{}' exist!", e.static_ty,
                                        lv.name).into()),
                                Some(renamed) => Ok((vec![], hir::Val::Var(renamed.clone())))
                            }
                        }
                    },
                    Some(p_exp) => {
                        match &p_exp.static_ty {
                            StaticType::Struct(s) => {
                                let (mut stmts, st_val) = self.emit_value(&p_exp)?;
                                let access_out = self.mk_intermediate_var();

                                stmts.push(hir::Statement::Call(hir::LValue::Var(access_out.clone()),
                                            hir::Callable::Access(st_val, s.clone(), lv.name.clone())));

                                Ok((stmts, hir::Val::Var(access_out)))
                            },
                            _ => Err(format!("[T-AST] Unexpected error, lvalue of type '{}' has no field '{}'!", p_exp.static_ty, lv.name).into())
                        }
                    }
                }
            },
            ExpVal::Mul(cst, var) => {
                let out = self.mk_intermediate_var();
                Ok((vec![
                    hir::Statement::Call(hir::LValue::Var(out.clone()),
                        hir::Callable::Bin(
                            hir::BinOp::Mul,
                            hir::Val::Const(hir::Type::Int64, *cst as u64),
                            hir::Val::Var(var.clone())
                        )
                    )
                ], hir::Val::Var(out)))
            },
            ExpVal::LMul(cst, block) => {
                let (mut stmts, b_val) = self.emit_block_value(block)?;
                let out = self.mk_intermediate_var();
                stmts.push(
                    hir::Statement::Call(hir::LValue::Var(out.clone()),
                        hir::Callable::Bin(hir::BinOp::Mul,
                            hir::Val::Const(hir::Type::Int64, *cst as u64),
                            b_val
                        )
                    )
                );

                Ok((stmts, hir::Val::Var(out)))
            },
            ExpVal::RMul(exp, var) => {
                let out = self.mk_intermediate_var();
                let (mut stmts, val) = self.emit_value(&exp)?;
                stmts.push(
                    hir::Statement::Call(hir::LValue::Var(out.clone()),
                        hir::Callable::Bin(
                            hir::BinOp::Mul,
                            val,
                            hir::Val::Var(var.clone())
                        )
                    )
                );

                Ok((stmts, hir::Val::Var(out)))
            },
            ExpVal::Return(internal_exp) => {
                // Evaluate internal_exp as a statement.
                // attribute nothing value.
                match internal_exp {
                    None => Ok((vec![], hir::Val::Nothing)),
                    Some(actual_exp) => {
                        self.emit_value(actual_exp)
                    }
                }
            },
            ExpVal::Call(name, args) => {
                let out = self.mk_intermediate_var();

                let (mut stmts, vals) = self.emit_values(args)?;

                // FIXME(Ryan): here, we are committing a sin.
                // we shall warn our user that he is doing something very bad
                // and we are going to do something even worse.
                // tl;dr: what you do if there is a struct S and a function S, same sig.
                if self.structure_names.contains(name) {
                    stmts.push(
                        hir::Statement::Call(
                            hir::LValue::Var(out.clone()),
                            hir::Callable::Alloc(name.clone())
                        )
                    );
                } else {
                    stmts.push(
                        hir::Statement::Call(hir::LValue::Var(out.clone()),
                        hir::Callable::Call(name.clone(), false, vals))
                    );
                }

                Ok((stmts, hir::Val::Var(out)))
            },
            ExpVal::If(cond, then, else_) => {
                // The value emitted by a if, is the value emitted by the then block or the else
                // block.

                let out = self.mk_intermediate_var();

                let (mut stmts, cond_val) = self.emit_value(cond)?;
                let (mut then_stmts, then_val) = self.emit_block_value(then)?;
                let (mut else_stmts, else_val) = self.emit_else_value(else_)?;

                then_stmts.push(
                    hir::Statement::Call(
                        hir::LValue::Var(out.clone()),
                        hir::Callable::Assign(then_val)
                    )
                );

                else_stmts.push(
                    hir::Statement::Call(
                        hir::LValue::Var(out.clone()),
                        hir::Callable::Assign(else_val)
                    )
                );

                // we want to write down if (cond) { then_stmts… out <- then_val; } else {
                // else_stmts… out <- else_val; }
                stmts.push(
                    hir::Statement::If(
                        cond_val,
                        hir::Block::new(then_stmts),
                        hir::Block::new(else_stmts)
                     )
                );

                Ok((stmts, hir::Val::Var(out)))
            }
            // FIXME(Ryan): verify we covered all cases.
            _ => Ok((vec![], hir::Val::Var(format!("I_AM_A_PLACEHOLDER_CHECK_ME_PLEASE: {:?}", e).to_string())))
        }
    }

    fn emit_statements(&mut self, e: &Exp) -> HIRStatementsResult {
        match e.val.as_ref() {
            ExpVal::Return(maybe_expr) => {
                // Here, we want to emit all statements required to eval e.
                // And then, emit a return with the value of e.
                match maybe_expr {
                    None => Ok(vec![hir::Statement::Return(hir::Val::Nothing)]),
                    Some(expr) => {
                        let (mut stmts, value) = self.emit_value(&expr)?;
                        stmts.push(hir::Statement::Return(value));
                        Ok(stmts)
                    }
                }
            },
            ExpVal::If(cond, then, else_) => {
                let (mut stmts, val_cond) = self.emit_value(cond)?;

                stmts.push(hir::Statement::If(
                    val_cond,
                    self.emit_block(&then, false)?,
                    self.emit_else_block(&else_)?
                ));

                Ok(stmts)
            },
            ExpVal::For(c, range, body) => {
                let (stmts_start, val_start) = self.emit_value(&range.start)?;
                let (stmts_end, val_end) = self.emit_value(&range.end)?;

                let mut stmts = stmts_start.into_iter().chain(stmts_end).collect::<Vec<_>>();
                stmts.push(hir::Statement::Call(hir::LValue::Var(c.name.clone()),
                        hir::Callable::Assign(val_start)));
                let boolean_val = self.mk_intermediate_var();

                let increment_counter_stmt = hir::Statement::Call(
                    hir::LValue::Var(c.name.clone()),
                    hir::Callable::Bin(hir::BinOp::Add,
                        hir::Val::Var(c.name.clone()),
                        hir::Val::Const(hir::Type::Int64, 1)));

                let boolean_update_stmt = hir::Statement::Call(
                    hir::LValue::Var(boolean_val.clone()),
                    hir::Callable::Bin(hir::BinOp::Leq,
                        hir::Val::Var(c.name.clone()),
                        val_end
                    ));

                let mut body_block = self.emit_block(&body, false)?;

                body_block.push(increment_counter_stmt);
                body_block.push(boolean_update_stmt);

                stmts.push(hir::Statement::While(hir::Val::Var(boolean_val), body_block));

                Ok(stmts)
            },
            ExpVal::While(cond, body) => {
                let (stmts_cond, val_cond) = self.emit_value(&cond)?;
                let mut while_body = self.emit_block(&body, false)?;

                let mut stmts = stmts_cond.clone();
                while_body.extend(stmts_cond);
                stmts.push(hir::Statement::While(
                        val_cond, while_body));

                Ok(stmts)
            },
            ExpVal::Call(f_name, args) => {
                let (mut stmts, vals) = self.emit_values(&args)?;

                stmts.push(hir::Statement::Call(
                    hir::LValue::Var(self.mk_intermediate_var()),
                    hir::Callable::Call(f_name.clone(), false, vals),
                ));

                Ok(stmts)
            },
            ExpVal::Assign(lv, e) => {
                match lv.in_exp.as_ref() {
                    None => self.emit_global_assign(&lv.name, &e),
                    Some(p_exp) => self.emit_complex_assign(p_exp, &lv.name, &e)
                }
            },
            ExpVal::Block(block) => {
                Ok(self.emit_block(block, false)?.stmts)
            },
            // FIXME: are LMul/RMul really dead code?
            ExpVal::BinOp(_, _, _) 
                | ExpVal::UnaryOp(_, _)
                | ExpVal::Int(_) | ExpVal::Str(_) | ExpVal::Bool(_)
                | ExpVal::LValue(_) 
                | ExpVal::Mul(_, _) | ExpVal::LMul(_, _) | ExpVal::RMul(_, _) 
                => Ok(vec![]), // Dead code.
        }
    }

    fn emit_global_assign(&mut self, var_name: &String, rhs_expr: &Exp) -> HIRStatementsResult {
        // lost thoughts: what if var_name is a global variable?
        // if there is at least one assignment in a function body of what is a global variable,
        // therefore, we have to assume that all lvalues in this body are local scoped and rely on
        // this local variable.
        // thus, we have to look for the current scope of var_name.
        // we shall rename the global variables rather than the local ones.
        if !self.current_params.contains(var_name) {
            self.current_local_vars.insert(var_name.clone());
        }

        // Here, we want to decompose rhs_expr as much as possible.
        // And finally, assign its value.
        let (mut stmts, val) = self.emit_value(rhs_expr)?;

        stmts.push(hir::Statement::Call(
                hir::LValue::Var(var_name.clone()),
                hir::Callable::Assign(val)
        ));

        Ok(stmts)
    }

    fn emit_complex_assign(&mut self, structure_exp: &Exp, field_name: &String, rhs_expr: &Exp) -> HIRStatementsResult {
        let (mut stmts, struct_val) = self.emit_value(structure_exp)?;
        let (rhs_stmts, rhs_val) = self.emit_value(rhs_expr)?;
        stmts.extend(rhs_stmts);

        // A structure must be allocated before to be used, right?
        if let hir::Val::Var(ref struct_val_name) = struct_val {
            if !self.current_params.contains(struct_val_name) && !self.current_local_vars.contains(struct_val_name) && !self.old_global_vars.contains_key(struct_val_name) {
                return Err(format!("[T-AST] Unbound structure variable, '{}' is not bound (params: {:?}, locals: {:?}, globals: {:?})!",
                struct_val_name.clone(), self.current_params, self.current_local_vars, self.global_vars).into());
            }
        } else {
            return Err(
                format!("[T-AST] Invalid assignment location, left hand is not a variable but a '{:?}'", struct_val).into());
        }

        match &structure_exp.static_ty {
            StaticType::Struct(struct_name) => {
                stmts.push(
                    hir::Statement::Call(
                        hir::LValue::Access(struct_val, struct_name.clone(), field_name.clone()),
                        hir::Callable::Assign(rhs_val)
                    )
                );

                Ok(stmts)
            },
            _ => Err(format!("[T-AST] Invalid assignment location, left hand is not a structure but a '{}'!", structure_exp.static_ty).into())
        }
    }

    fn emit_values(&mut self, exprs: &Vec<Exp>) -> HIRMultiValuesResult {
        let mut vals: Vec<hir::Val> = vec![];
        let mut stmts: Vec<hir::Statement> = vec![];

        for exp in exprs {
            let (exp_stmts, val) = self.emit_value(exp)?;
            stmts.extend(exp_stmts);
            vals.push(val);
        }

        Ok((stmts, vals))
    }

    fn emit_flattened_statements(&mut self, exps: &[Exp]) -> HIRStatementsResult {
        exps.iter().map(|e| self.emit_statements(&e)).flat_map(|result| match result {
            Ok(stmts) => stmts.into_iter().map(|item| Ok(item)).collect(),
            Err(err) => vec![Err(err)]
        }).collect::<HIRStatementsResult>()
    }

    fn emit_block(&mut self, b: &Block, allow_implicit_returns: bool) -> HIRBlockResult {
        if allow_implicit_returns && !b.trailing_semicolon {
            let (mut stmts, val) = self.emit_block_value(b)?;
            stmts.push(
                hir::Statement::Return(val)
            );

            Ok(hir::Block::new(stmts))
        } else {
            self.emit_flattened_statements(&b.val)
            .and_then(|stmts| Ok(hir::Block::new(stmts)))
        }
    }

    fn emit_else_block(&mut self, else_: &Else) -> HIRBlockResult {
        match else_.val.as_ref() {
            ElseVal::End => Ok(hir::Block::new(vec![])),
            ElseVal::Else(block) => self.emit_block(&block, false),
            ElseVal::ElseIf(cond, then, else__) => {
                let (mut stmts, cond_val) = self.emit_value(&cond)?;

                stmts.push(
                    hir::Statement::If(cond_val, self.emit_block(&then, false)?, self.emit_else_block(&else__)?)
                );

                Ok(hir::Block::new(stmts))
            }
        }
    }

    fn emit_else_value(&mut self, else_: &Else) -> HIRValueResult {
        match else_.val.as_ref() {
            ElseVal::End => Ok((vec![], hir::Val::Nothing)),
            ElseVal::Else(block) => self.emit_block_value(&block),
            ElseVal::ElseIf(cond, then, else__) => {
                let out = self.mk_intermediate_var();
                let (mut stmts, cond_val) = self.emit_value(&cond)?;
                let (mut then_stmts, then_val) = self.emit_block_value(then)?;
                let (mut else_stmts, else_val) = self.emit_else_value(else__)?;

                then_stmts.push(hir::Statement::Call(
                                hir::LValue::Var(out.clone()),
                                hir::Callable::Assign(then_val)));
                else_stmts.push(hir::Statement::Call(
                        hir::LValue::Var(out.clone()),
                        hir::Callable::Assign(else_val)));

                stmts.push(
                    hir::Statement::If(cond_val,
                        hir::Block::new(then_stmts),
                        hir::Block::new(else_stmts)
                    )
                );

                Ok((stmts, hir::Val::Var(out)))
            }
        }
    }

    fn emit_fn(&mut self, f: &Function, name: String) -> HIRFunctionResult {
        // Reset the counters and local state.
        self.current_local_vars.clear();
        self.current_params = f.params.iter().map(|f| f.name.name.clone()).collect();
        self.next_intermediate_variable_id = 0;


        let block = self.emit_block(&f.body, true)?;
        Ok(hir::Function::new(
            name,
            f.params.iter().map(|f| f.name.name.clone()).collect(),
            self.current_local_vars.drain().collect(),
            block
        ))
    }

    fn emit_dynamic_dispatch_condition_signature_match(&mut self, sig: Vec<(String, StaticType)>) -> HIRValueResult {
        // We want here to compute the condition for a signature match.
        let mut stmts = vec![];
        let mut conds_val = vec![];
        for (arg_name, expected_type) in sig {
            if let Some(r_type) = from_static_type(expected_type) {
                let out = self.mk_intermediate_var();
                conds_val.push(out.clone());
                stmts.push(
                    hir::Statement::Call(hir::LValue::Var(out),
                        hir::Callable::IsType(
                            hir::Val::Var(arg_name.clone()),
                            r_type
                        )
                    )
                );
            }
        }

        // now, we compute the and-value in a fold-fashion.
        let cond_out = self.mk_intermediate_var();
        // $cond_out <- true
        stmts.push(
            hir::Statement::Call(hir::LValue::Var(cond_out.clone()),
                hir::Callable::Assign(
                    hir::Val::Const(
                        hir::Type::Bool,
                        1
                    )
                )
            )
        );

        // $cond_out <- $cond_out && $conds_val[i] for all i.
        conds_val.iter().for_each(|val| {
            stmts.push(
                hir::Statement::Call(
                    hir::LValue::Var(cond_out.clone()),
                    hir::Callable::Bin(
                        hir::BinOp::And,
                        hir::Val::Var(cond_out.clone()),
                        hir::Val::Var(val.clone())
                    )
                )
            );
        });

        Ok((stmts, hir::Val::Var(cond_out)))
    }

    fn emit_dynamic_dispatch(&mut self, name: &String, f_s: &Vec<Function>) -> HIRDeclsResult {
        if f_s.len() > 1 {
            let mut functions = vec![];
            let mut fun_decls = vec![];
            let mut weights = vec![0; f_s.len()]; // Selectivity weights.
            let mut stmts = vec![];
            let args: Vec<hir::Val> = f_s.first().unwrap().params.iter().map(|arg| hir::Val::Var(arg.name.name.clone())).collect();
            let str_sig: Vec<String> = f_s.first().unwrap().params.iter().map(|arg| arg.name.name.clone()).collect();
            let out = self.mk_intermediate_var();

            for (index, f) in f_s.iter().enumerate() {
                // Generate condition: typeof(arg_1) == param_1 && typeof(arg_2) == param_2 && …
                let (cond_stmt, cond_val) = self.emit_dynamic_dispatch_condition_signature_match(
                    f.params.iter().map(|param| (param.name.name.clone(), param.ty.clone())).collect()
                )?;
                stmts.extend(cond_stmt);

                // Compute selectivity weight
                weights[index] = f.params.iter().map(|param| return if param.ty != StaticType::Any { 1 } else { 0 }).sum();

                // Rename function.
                let new_fun_name = format!("{}_{}", name, index).to_string();
                functions.push((weights[index], cond_val, new_fun_name.clone()));
            }

            let intermediate_vars = self.current_local_vars.drain().collect();

            // We cannot regroup the for-loops as we rely on the implicit behavior of intermediate
            // values counting.

            for (index, f) in f_s.iter().enumerate() {
                fun_decls.push(hir::Decl::Function(self.emit_fn(f, format!("{}_{}", name, index).to_string())?));
            }

            // Sanity checks:
            // we only have at most 1 weight of 0.
            // which is the generic function which can fit our "generic" else case.
            assert!(weights.iter().filter(|&n| *n == 0).count() <= 1, "Dynamic dispatch disaster: more than one generic function found during phase 1 of compilation");

            // Sort the functions by INCREASING selectivity, we want to build the nodes in the
            // reverse order.
            // Starting from the else and going until the first one.
            functions.sort_by_key(|(w, _, _)| *w);

            // fold over all blocks to build the if cascade in selectivity order.
            let mut body = hir::Block::new(stmts).merge(functions.into_iter().fold(hir::Block::new(vec![
                        hir::Statement::Call(hir::LValue::Var(out.clone()),
                            hir::Callable::Call("panic".to_string(), true, vec![
                                hir::Val::Str(format!("Dynamic dispatch failure for function call '{}'", name))
                            ])
                        )
            ]), |prev_block, (weight, cond_val, name)| {
                let call_block = hir::Block::new(vec![
                    hir::Statement::Call(hir::LValue::Var(out.clone()),
                        hir::Callable::Call(name, false,
                            args.clone())
                        )
                ]);

                if weight == 0 { call_block }
                else {
                    hir::Block::new(vec![hir::Statement::If(cond_val.clone(), call_block, prev_block)])
                }
            }));

            body.push(hir::Statement::Return(
                    hir::Val::Var(out)
            ));

            Ok(fun_decls
                .into_iter()
                .chain(once(hir::Decl::Function(
                            hir::Function::new(name.clone(), str_sig, intermediate_vars, body))))
                .collect())
        } else {
            Ok(vec![hir::Decl::Function(self.emit_fn(f_s.first().unwrap(), name.clone())?)])
        }
    }

    fn emit_entrypoint(&mut self, fun_names: HashSet<String>, toplevel: Vec<Exp>) -> HIREntrypointResult {
        // __start0000000000… let's say (0… to avoid the case where the user already defined
        // __start00…)
        // we generate all global variables so that we can hand it out to the emitter for intermediate
        // variables generation.
        // then we set __start00… to be the entrypoint.

        // 1. select the name.
        let mut entrypoint_name = "__start".to_string();
        let mut idx = 0;
        while fun_names.contains(&entrypoint_name) {
            entrypoint_name = format!("__start{}", idx); // ensure no collision with dynamic dispatch variants.
            idx += 1;
        }
        // 2. collect all assignments in the body and mark them as global.
        // perform renames with globals → __gXXX + suffix until unique.
        let mut raw_global_vars = collect_all_assign_in_array(&toplevel)
            .into_iter()
            .map(|lident| lident.name)
            .collect::<Vec<_>>();

        raw_global_vars.push("nothing".to_string()); // Implicit variable of type Nothing.

        for gvar in raw_global_vars {
            let gvar_new_name = self.emit_unique_gvar_name(&gvar);
            self.global_vars.insert(gvar_new_name.clone());
            self.old_global_vars.insert(gvar, gvar_new_name);
        }

        // 3. build the body by concatenating the statements of all expression in order.
        let body: hir::Block = hir::Block::new(self.emit_flattened_statements(&toplevel)?);

        Ok((self.global_vars.iter().cloned().collect(), hir::Function::new(entrypoint_name, vec![], self.current_local_vars.drain().collect(), body)))
    }
}

fn emit_struct_decl(s: &Structure) -> HIRStructDeclResult {
    Ok(hir::StructDecl::new(s.name.name.clone(),
        s.fields.iter().map(|f| f.name.name.clone()).collect()
    ))
}

fn fun_name_variants(name: &String, variants: usize) -> Vec<String> {
    (0..variants).into_iter().map(|i| format!("{}_{}", name, i)).collect()
}

pub fn typed_ast_to_hir(t_ast: TypedDecls) -> HIRSourceResult {
    let mut compiled = Vec::new();

    for s in t_ast.structures.values() {
        compiled.push(hir::Decl::Struct(emit_struct_decl(s)?));
    }

    let mut emitter = Emitter::init(t_ast.structures.keys().cloned().collect());
    // generate entrypoint based on the global expressions, where all variables *are global*.
    let (globals, fun) = emitter.emit_entrypoint(
                t_ast.functions.iter().flat_map(|(name, f_s)| fun_name_variants(name, f_s.len())).collect(),
                t_ast.global_expressions
        )?;

    // generate dynamic dispatch thunk.
    for (name, f_s) in t_ast.functions {
        compiled.extend(emitter.emit_dynamic_dispatch(&name, &f_s)?);
    }

    let entrypoint_name = fun.name.clone();

    compiled.push(hir::Decl::Function(fun));
    
    Ok(hir::Source::new(globals, entrypoint_name, compiled))
}
