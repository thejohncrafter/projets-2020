
use std::collections::HashMap;

use super::hir::types as hir;
use super::lir::types as lir;
use super::error::*;

struct LabelGenerator {
    next_label_id: usize,
}

impl LabelGenerator {
    fn new() -> Self {
        LabelGenerator {next_label_id: 0}
    }

    fn new_label(&mut self) -> lir::Label {
        let lbl = lir::Label::new(format!("lbl_{}", self.next_label_id));
        self.next_label_id += 1;
        lbl
    }
}

enum ConcreteType {
    Nothing,
    Int64,
    Bool,
    Str,
    Struct(String),
}

impl From<&hir::Type> for ConcreteType {
    fn from(t: &hir::Type) -> Self {
        match t {
            hir::Type::Int64 => ConcreteType::Int64,
            hir::Type::Bool => ConcreteType::Bool,
            hir::Type::Str => ConcreteType::Str,
            hir::Type::Struct(name) => ConcreteType::Struct(name.clone()),
        }
    }
}

struct FieldData {
    id: u64,
}

impl FieldData {
    fn new(id: u64) -> FieldData {
        FieldData {id}
    }

    fn val_addr(&self) -> u64 {
        16 * self.id + 8
    }

    fn ty_addr(&self) -> u64 {
        16 * self.id
    }
}

struct StructData {
    name: String,
    id: u64,
    fields: HashMap<String, FieldData>,
}

impl StructData {
    fn new(name: String, id: u64, decl: &hir::StructDecl) -> Self {
        StructData {
            name, id,
            fields: decl.fields.iter().enumerate()
                .map(|(i, name)| (name.clone(), FieldData::new(i as u64)))
                .collect()
        }
    }

    fn get_field(&self, name: &str) -> Result<&FieldData, Error> {
        match self.fields.get(name) {
            Some(data) => Ok(data),
            None => Err(format!(
                    "Structure \"{}\" has no field named \"{}\"",
                    self.name, name
                ).into())
        }
    }
}

struct GlobalRegistry {
    map: HashMap<String, StructData>,
    next_label_id: usize,
}

impl GlobalRegistry {
    fn new(vars: &[&hir::StructDecl]) -> Self {
        GlobalRegistry {
            map: vars.iter().enumerate()
                .map(|(i, d)| (d.name.clone(), StructData::new(d.name.clone(), i as u64, d)))
                .collect(),
            next_label_id: 0,
        }
    }

    fn get_ty_id(&self, ty: &ConcreteType) -> Result<lir::Val, Error> {
        match ty {
            ConcreteType::Nothing => Ok(lir::Val::Const(0)),
            ConcreteType::Int64 => Ok(lir::Val::Const(1)),
            ConcreteType::Bool => Ok(lir::Val::Const(2)),
            ConcreteType::Str => Ok(lir::Val::Const(3)),
            ConcreteType::Struct(name) => {
                match self.map.get(name) {
                    Some(data) => Ok(lir::Val::Const(data.id)),
                    None => Err(format!("Structure \"{}\" was not declared", name).into()),
                }
            }
        }
    }

    fn get_struct(&self, name: &str) -> Result<&StructData, Error> {
        match self.map.get(name) {
            Some(data) => Ok(data),
            None => Err(format!("Structure \"{}\" was not declared", name).into()),
        }
    }
}

struct VarData {
    val_name: String,
    ty_name: String,
}

impl VarData {
    fn new(id: usize) -> Self {
        VarData {
            val_name: format!("var_{}_val", id),
            ty_name: format!("var_{}_ty", id),
        }
    }
}

struct CompiledVal {
    ty: lir::Val,
    val: lir::Val,
}

impl CompiledVal {
    fn new(ty: lir::Val, val: lir::Val) -> Self {
        CompiledVal {ty, val}
    }
}

struct LocalRegistry<'a> {
    parent: &'a GlobalRegistry,
    map: HashMap<String, VarData>,
}

impl<'a> LocalRegistry<'a> {
    fn new(parent: &'a GlobalRegistry, vars: &[String]) -> Self {
        LocalRegistry {
            parent,
            map: vars.iter().enumerate()
                .map(|(i, v)| (v.clone(), VarData::new(i)))
                .collect(),
        }
    }

    fn compile_val(&self, val: &hir::Val) -> Result<CompiledVal, Error> {
        match val {
            hir::Val::Var(name) => {
                let data = self.get_var(name)?;
                Ok(CompiledVal::new(
                    lir::Val::Var(data.ty_name.clone()),
                    lir::Val::Var(data.val_name.clone()),
                ))
            },
            hir::Val::Const(t, i) => {
                Ok(CompiledVal::new(
                    self.parent.get_ty_id(&t.into())?,
                    lir::Val::Const(*i),
                ))
            },
            hir::Val::Str(s) => {
                Ok(CompiledVal::new(
                    self.parent.get_ty_id(&ConcreteType::Str)?,
                    lir::Val::Str(s.clone())
                ))
            },
        }
    }

    fn get_var(&self, name: &str) -> Result<&VarData, Error> {
        match self.map.get(name) {
            Some(data) => Ok(data),
            None => Err(format!("Variable \"{}\" was not declared", name).into()),
        }
    }
}

fn compile_call(
    global: &GlobalRegistry,
    local: &LocalRegistry,
    dest: &str,
    call: &hir::Callable
) -> Result<Vec<lir::Statement>, Error> {
    let mut out = Vec::new();

    match call {
        hir::Callable::Call(fn_name, args) => {
            let dest_var = local.get_var(dest)?;
            let mut vars = Vec::new();
            args.iter().try_for_each(|arg| -> Result<(), Error> {
                let arg = local.compile_val(arg)?;
                vars.push(arg.ty); vars.push(arg.val);
                Ok(())
            })?;

            out.push(lir::Statement::Inst(lir::Instruction::Call(
                Some((dest_var.ty_name.clone(), dest_var.val_name.clone())),
                fn_name.clone(),
                vars,
            )));
        },
        hir::Callable::Bin(op, a, b) => {
            let (dest_ty, lir_op) = match op {
                hir::BinOp::And => (ConcreteType::Bool, lir::BinOp::And),
                hir::BinOp::Or  => (ConcreteType::Bool, lir::BinOp::Or ),
                hir::BinOp::Equ => (ConcreteType::Bool, lir::BinOp::Equ),
                hir::BinOp::Neq => (ConcreteType::Bool, lir::BinOp::Neq),
                hir::BinOp::Lt  => (ConcreteType::Bool, lir::BinOp::Lt ),
                hir::BinOp::Leq => (ConcreteType::Bool, lir::BinOp::Leq),
                hir::BinOp::Gt  => (ConcreteType::Bool, lir::BinOp::Gt ),
                hir::BinOp::Geq => (ConcreteType::Bool, lir::BinOp::Geq),

                hir::BinOp::Add => (ConcreteType::Int64, lir::BinOp::Add),
                hir::BinOp::Sub => (ConcreteType::Int64, lir::BinOp::Sub),
                hir::BinOp::Mul => (ConcreteType::Int64, lir::BinOp::Mul),
                hir::BinOp::Div => (ConcreteType::Int64, lir::BinOp::Div),
            };

            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    local.get_var(dest)?.ty_name.clone(),
                    global.get_ty_id(&dest_ty)?
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Bin(
                    local.get_var(dest)?.val_name.clone(),
                    lir_op,
                    local.compile_val(a)?.val,
                    local.compile_val(b)?.val,
                )
            ));
        },
        hir::Callable::Assign(a) => {
            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    local.get_var(dest)?.ty_name.clone(),
                    local.compile_val(a)?.ty,
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    local.get_var(dest)?.val_name.clone(),
                    local.compile_val(a)?.ty,
                )
            ));
        },
        hir::Callable::IsType(a, t) => {
            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    local.get_var(dest)?.ty_name.clone(),
                    global.get_ty_id(&ConcreteType::Bool)?,
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Bin(
                    local.get_var(dest)?.val_name.clone(),
                    lir::BinOp::Equ,
                    local.compile_val(a)?.ty,
                    global.get_ty_id(&t.into())?,
                )
            ));
        },
        hir::Callable::Access(a, structure, field) => {
            let data = global.get_struct(structure)?.get_field(field)?;
            out.push(lir::Statement::Inst(
                lir::Instruction::Access(
                    local.get_var(dest)?.ty_name.clone(),
                    local.compile_val(a)?.ty,
                    data.ty_addr(),
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Access(
                    local.get_var(dest)?.val_name.clone(),
                    local.compile_val(a)?.val,
                    data.val_addr(),
                )
            ));
        },
    }

    Ok(out)
}

fn compile_return(
    _global: &GlobalRegistry,
    local: &LocalRegistry,
    a: &hir::Val,
) -> Result<Vec<lir::Statement>, Error> {
    let data = local.compile_val(a)?;

    Ok(vec!(lir::Statement::Inst(
        lir::Instruction::Return(data.ty, data.val)
    )))
}

fn compile_if(
    lbl_gen: &mut LabelGenerator,
    global: &GlobalRegistry,
    local: &LocalRegistry,
    cond: &hir::Val,
    block_true: &hir::Block,
    block_false: &hir::Block,
) -> Result<Vec<lir::Statement>, Error> {
    let mut out = Vec::new();

    let lbl_true = lbl_gen.new_label();
    let lbl_end = lbl_gen.new_label();

    out.push(lir::Statement::Inst(
        lir::Instruction::Jumpif(
            local.compile_val(cond)?.val,
            lbl_true.clone(),
        )
    ));

    out.extend(compile_block(lbl_gen, global, local, block_false)?);

    out.push(lir::Statement::Inst(
        lir::Instruction::Jump(lbl_end.clone(),)
    ));
    out.push(lir::Statement::Label(lbl_true.clone()));

    out.extend(compile_block(lbl_gen, global, local, block_true)?);

    out.push(lir::Statement::Label(lbl_end.clone()));

    Ok(out)
}

fn compile_while(
    lbl_gen: &mut LabelGenerator,
    global: &GlobalRegistry,
    local: &LocalRegistry,
    cond: &hir::Val,
    block: &hir::Block,
) -> Result<Vec<lir::Statement>, Error> {
    let mut out = Vec::new();

    let lbl_body = lbl_gen.new_label();
    let lbl_end = lbl_gen.new_label();

    out.push(lir::Statement::Label(lbl_body.clone()));
    out.push(lir::Statement::Inst(
        lir::Instruction::JumpifNot(
            local.compile_val(cond)?.val,
            lbl_end.clone(),
        )
    ));

    out.extend(compile_block(lbl_gen, global, local, block)?);

    out.push(lir::Statement::Inst(
        lir::Instruction::Jump(lbl_body.clone(),)
    ));

    out.push(lir::Statement::Label(lbl_end.clone()));

    Ok(out)
}

fn compile_block(
    lbl_gen: &mut LabelGenerator,
    global: &GlobalRegistry,
    local: &LocalRegistry,
    block: &hir::Block,
) -> Result<Vec<lir::Statement>, Error> {
    let mut out = Vec::new();

    block.stmts.iter().try_for_each(|stmt| -> Result<(), Error> {
            match stmt {
                hir::Statement::FnCall(fn_name, args) => {
                    let mut vars = Vec::new();
                    args.iter().try_for_each(|arg| -> Result<(), Error> {
                        let arg = local.compile_val(arg)?;
                        vars.push(arg.ty); vars.push(arg.val);
                        Ok(())
                    })?;

                    out.push(lir::Statement::Inst(lir::Instruction::Call(
                        None,
                        fn_name.clone(),
                        vars,
                    )));
                },
                hir::Statement::Call(dest, call) => {
                    out.extend(compile_call(global, local, dest, call)?);
                },
                hir::Statement::Return(a) => {
                    out.extend(compile_return(global, local, a)?);
                },
                hir::Statement::If(cond, block_true, block_false) => {
                    out.extend(compile_if(lbl_gen, global, local, cond, block_true, block_false)?);
                },
                hir::Statement::While(cond, block) => {
                    out.extend(compile_while(lbl_gen, global, local, cond, block)?);
                },
            }
            
            Ok(())
        })?;

    Ok(out)
}

fn compile_fn(
    global: &GlobalRegistry,
    f: &hir::Function
) -> Result<lir::Function, Error> {
    let local = LocalRegistry::new(&global, &f.vars);
    let mut lbl_gen = LabelGenerator::new();

    let body = compile_block(&mut lbl_gen, &global, &local, &f.body)?;

    let mut lir_args = Vec::new();
    f.args.iter().try_for_each(|arg| -> Result<(), Error> {
        let arg = local.get_var(arg)?;
        lir_args.push(arg.ty_name.clone());
        lir_args.push(arg.val_name.clone());
        Ok(())
    })?;
    
    Ok(lir::Function::new(
        f.name.clone(),
        lir_args,
        lir::Block::new(body),
    ))
}

pub fn hir_to_lir(hir: &[hir::Decl]) -> Result<Vec<lir::Function>, Error> {
    let mut compiled = Vec::new();

    let structs = hir.iter().filter_map(|d| {
            match d {
                hir::Decl::Struct(s) => Some(s),
                _ => None
            }
        }).collect::<Vec<_>>();
    let global = GlobalRegistry::new(&structs);

    hir.iter().try_for_each(|d| -> Result<(), Error> {
            match d {
                hir::Decl::Struct(_) => (),
                hir::Decl::Function(f) => compiled.push(compile_fn(&global, f)?),
            }
            Ok(())
        })?;

    Ok(compiled)
}

