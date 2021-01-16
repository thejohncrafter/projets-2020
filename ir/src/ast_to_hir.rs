use std::collections::HashSet;

use super::hir::types as hir;
use super::error::*;

use parser::ast::*;
use parser::typing::data::TypedDecls;

pub type HIRValueResult = Result<(Vec<hir::Statement>, hir::Val), Error>;
pub type HIRMultiValuesResult = Result<(Vec<hir::Statement>, Vec<hir::Val>), Error>;
pub type HIRStatementsResult = Result<Vec<hir::Statement>, Error>;
pub type HIRBlockResult = Result<hir::Block, Error>;
pub type HIRFunctionResult = Result<hir::Function, Error>;
pub type HIRStructDeclResult = Result<hir::StructDecl, Error>;
pub type HIRDeclsResult = Result<Vec<hir::Decl>, Error>;

struct Emitter {
    pub next_intermediate_variable_id: u64,
    pub current_local_vars: Vec<String>
}

impl Emitter {
    fn init() -> Self {
        Emitter { next_intermediate_variable_id: 0, current_local_vars: vec![] }
    }

    fn mk_intermediate_var(&mut self) -> String {
        let out = format!("__intermediate_internal{}", self.next_intermediate_variable_id);
        self.current_local_vars.push(out.clone());
        self.next_intermediate_variable_id += 1;
        out
    }

    fn emit_value(&mut self, e: &Exp) -> HIRValueResult {
        match e.val.as_ref() {
            ExpVal::BinOp(op, a, b) => {
                let (stmts_a, val_a) = self.emit_value(a)?;
                let (stmts_b, val_b) = self.emit_value(b)?;
                let mut stmts = stmts_a.into_iter().chain(stmts_b).collect::<Vec<_>>();
                let out = self.mk_intermediate_var();
                stmts.push(hir::Statement::Call(out.clone(),
                        hir::Callable::Bin(hir::BinOp::from(*op), val_a, val_b)));
                Ok((stmts, hir::Val::Var(out)))
            },
            ExpVal::UnaryOp(op, e) => {
                Ok((vec![], hir::Val::Var(self.mk_intermediate_var())))
            },
            ExpVal::Int(cst) => Ok((vec![], hir::Val::Const(hir::Type::Int64, *cst as u64))),
            ExpVal::Bool(cst) => Ok((vec![], hir::Val::Const(hir::Type::Bool, u64::from(*cst)))),
            ExpVal::Str(cst) => Ok((vec![], hir::Val::Str(cst.clone()))),
            ExpVal::LValue(lv) => {
                match lv.in_exp.as_ref() {
                    None => Ok((vec![], hir::Val::Var(lv.name.clone()))),
                    Some(p_exp) => {
                        match &p_exp.static_ty {
                            StaticType::Struct(s) => {
                                let (mut stmts, st_val) = self.emit_value(&p_exp)?;
                                let access_out = self.mk_intermediate_var();

                                stmts.push(hir::Statement::Call(access_out.clone(),
                                            hir::Callable::Access(st_val, s.clone(), lv.name.clone())));

                                Ok((stmts, hir::Val::Var(access_out)))
                            },
                            _ => Err(format!("Unexpected error, lvalue of type '{}' has no field '{}'!", p_exp.static_ty, lv.name).into())
                        }
                    }
                }
            },
            ExpVal::Mul(cst, var) => {
                let out = self.mk_intermediate_var();
                Ok((vec![
                    hir::Statement::Call(out.clone(),
                        hir::Callable::Bin(
                            hir::BinOp::Mul,
                            hir::Val::Const(hir::Type::Int64, *cst as u64),
                            hir::Val::Var(var.clone())
                        )
                    )
                ], hir::Val::Var(out)))
            },
            ExpVal::LMul(cst, block) => {
                // FIXME: handle block implicit values.
                Ok((vec![], hir::Val::Var(self.mk_intermediate_var())))
            },
            ExpVal::RMul(exp, var) => {
                let out = self.mk_intermediate_var();
                let (mut stmts, val) = self.emit_value(&exp)?;
                stmts.push(
                    hir::Statement::Call(out.clone(),
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
                    None => Ok((vec![], hir::Val::Var("nothing".to_string()))),
                    Some(actual_exp) => {
                        self.emit_value(actual_exp)
                    }
                }
            },
            ExpVal::Call(name, args) => {
                let out = self.mk_intermediate_var();

                let (mut stmts, vals) = self.emit_values(args)?;

                stmts.push(
                    hir::Statement::Call(out.clone(),
                    hir::Callable::Call(name.clone(), vals))
                );

                Ok((stmts, hir::Val::Var(out)))
            },
            _ => Ok((vec![], hir::Val::Var("nothing".to_string())))
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
                    self.emit_block(&then)?,
                    self.emit_else_block(&else_)?
                ));

                Ok(stmts)
            },
            ExpVal::For(c, range, body) => {
                let (stmts_start, val_start) = self.emit_value(&range.start)?;
                let (stmts_end, val_end) = self.emit_value(&range.end)?;

                let mut stmts = stmts_start.into_iter().chain(stmts_end).collect::<Vec<_>>();
                stmts.push(hir::Statement::Call(c.name.clone(),
                        hir::Callable::Assign(val_start)));
                let boolean_val = self.mk_intermediate_var();

                let increment_counter_stmt = hir::Statement::Call(
                    c.name.clone(),
                    hir::Callable::Bin(hir::BinOp::Add,
                        hir::Val::Var(c.name.clone()),
                        hir::Val::Const(hir::Type::Int64, 1)));

                let boolean_update_stmt = hir::Statement::Call(
                    boolean_val.clone(),
                    hir::Callable::Bin(hir::BinOp::Leq,
                        hir::Val::Var(c.name.clone()),
                        val_end
                    ));

                let mut body_block = self.emit_block(&body)?;

                body_block.push(increment_counter_stmt);
                body_block.push(boolean_update_stmt);

                stmts.push(hir::Statement::While(hir::Val::Var(boolean_val), body_block));

                Ok(stmts)
            },
            ExpVal::While(cond, body) => {
                let (stmts_cond, val_cond) = self.emit_value(&cond)?;
                let mut while_body = self.emit_block(&body)?;

                let mut stmts = stmts_cond.clone();
                while_body.extend(stmts_cond);
                stmts.push(hir::Statement::While(
                        val_cond, while_body));

                Ok(stmts)
            },
            ExpVal::Call(f_name, args) => {
                let (mut stmts, vals) = self.emit_values(&args)?;

                stmts.push(hir::Statement::FnCall(f_name.clone(), vals));

                Ok(stmts)
            },
            ExpVal::Assign(lv, e) => {
                match lv.in_exp.as_ref() {
                    None => self.emit_global_assign(&lv.name, &e),
                    Some(p_exp) => self.emit_complex_assign(p_exp, &lv.name, &e)
                }
            },
            ExpVal::Block(block) => {
                Ok(self.emit_block(block)?.stmts)
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
        Ok(vec![])
    }

    fn emit_complex_assign(&mut self, structure_exp: &Exp, field_name: &String, rhs_expr: &Exp) -> HIRStatementsResult {
        Ok(vec![])
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

    fn emit_block(&mut self, b: &Block) -> HIRBlockResult {
        b.val.iter().map(|e| self.emit_statements(&e)).flat_map(|result| match result {
            Ok(stmts) => stmts.into_iter().map(|item| Ok(item)).collect(),
            Err(err) => vec![Err(err)]
        }).collect::<HIRStatementsResult>().and_then(|stmts| Ok(hir::Block::new(stmts)))
    }

    fn emit_else_block(&mut self, else_: &Else) -> HIRBlockResult {
        match else_.val.as_ref() {
            ElseVal::End => Ok(hir::Block::new(vec![])),
            ElseVal::Else(block) => self.emit_block(&block),
            ElseVal::ElseIf(cond, then, else__) => {
                let (mut stmts, cond_val) = self.emit_value(&cond)?;

                stmts.push(
                    hir::Statement::If(cond_val, self.emit_block(&then)?, self.emit_else_block(&else__)?)
                );

                Ok(hir::Block::new(stmts))
            }
        }
    }

    fn emit_fn(&mut self, f: &Function, name: String) -> HIRFunctionResult {
        // Reset the counters and local state.
        self.current_local_vars.clear();
        self.next_intermediate_variable_id = 0;

        let block = self.emit_block(&f.body)?;

        Ok(hir::Function::new(
            name,
            f.params.iter().map(|f| f.name.name.clone()).collect(),
            self.current_local_vars.drain(..).collect(), // FIXME: collect all assigns.
            block
        ))
    }

    fn emit_dynamic_dispatch(&mut self, name: &String, f_s: &Vec<Function>) -> HIRDeclsResult {
        if f_s.len() > 1 {
            let mut functions = vec![];
            for (index, f) in f_s.iter().enumerate() {
                functions.push(hir::Decl::Function(self.emit_fn(f, format!("{}_{}", name, index).to_string())?));
            }

            // now the dynamic dispatch thunk
            // generate condition: typeof(arg_1) == param_1 && typeof(arg_2) == param_2 && …
            // generate blocks: call function of corresponding signature
            // generate if/elseif/else blocks.
            // FIXME: do it.

            Ok(functions)
        } else {
            Ok(vec![hir::Decl::Function(self.emit_fn(f_s.first().unwrap(), name.clone())?)])
        }
    }
}

fn emit_struct_decl(s: &Structure) -> HIRStructDeclResult {
    Ok(hir::StructDecl::new(s.name.name.clone(),
        s.fields.iter().map(|f| f.name.name.clone()).collect()
    ))
}

pub fn typed_ast_to_hir(t_ast: TypedDecls) -> HIRDeclsResult {
    let mut compiled = Vec::new();
    let mut emitter = Emitter::init();

    for s in t_ast.structures.values() {
        compiled.push(hir::Decl::Struct(emit_struct_decl(s)?));
    }

    // generate dynamic dispatch thunk.
    for (name, f_s) in t_ast.functions {
        compiled.extend(emitter.emit_dynamic_dispatch(&name, &f_s)?);
    }

    // FIXME: build an adhoc thunk for global expressions as a main function.
    
    Ok(compiled)
}
