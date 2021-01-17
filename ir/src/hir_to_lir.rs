
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
            hir::Type::Nothing => ConcreteType::Nothing,
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

    fn get_mem_len(&self) -> usize {
        16 * self.fields.len()
    }
}

struct VarData {
    val_name: String,
    ty_name: String,
}

impl VarData {
    fn new(id: usize, is_global: bool) -> Self {
        let prefix = if is_global {"global_"} else {""};
        VarData {
            val_name: format!("{}var_{}_val", prefix, id),
            ty_name: format!("{}var_{}_ty", prefix, id),
        }
    }
}

struct GlobalRegistry {
    globals_map: HashMap<String, VarData>,
    structs_map: HashMap<String, StructData>,
    fn_map: HashMap<String, usize>,
}

impl GlobalRegistry {
    fn new(globals: &[String], vars: &[&hir::StructDecl], fns: &[&str]) -> Self {
        GlobalRegistry {
            globals_map: globals.iter().enumerate()
                .map(|(i, v)| (v.clone(), VarData::new(i, true)))
                .collect(),
            structs_map: vars.iter().enumerate()
                .map(|(i, d)| (d.name.clone(), StructData::new(d.name.clone(), i as u64, d)))
                .collect(),
            fn_map: fns.iter().enumerate()
                .map(|(i, name)| (name.to_string(), i))
                .collect(),
        }
    }

    fn compiled_var_names(&self) -> Vec<String> {
        self.globals_map.values()
            .map(|data| vec!(data.ty_name.clone(), data.val_name.clone()))
            .flatten().collect()
    }

    fn get_var(&self, name: &str) -> Result<&VarData, Error> {
        match self.globals_map.get(name) {
            Some(data) => Ok(data),
            None => Err(format!("Variable \"{}\" was not declared", name).into()),
        }
    }

    fn get_ty_id(&self, ty: &ConcreteType) -> Result<lir::Val, Error> {
        match ty {
            ConcreteType::Nothing => Ok(lir::Val::Const(0)),
            ConcreteType::Int64 => Ok(lir::Val::Const(1)),
            ConcreteType::Bool => Ok(lir::Val::Const(2)),
            ConcreteType::Str => Ok(lir::Val::Const(3)),
            ConcreteType::Struct(name) => {
                match self.structs_map.get(name) {
                    Some(data) => Ok(lir::Val::Const(data.id + 4)),
                    None => Err(format!("Structure \"{}\" was not declared", name).into()),
                }
            }
        }
    }

    fn get_struct(&self, name: &str) -> Result<&StructData, Error> {
        match self.structs_map.get(name) {
            Some(data) => Ok(data),
            None => Err(format!("Structure \"{}\" was not declared", name).into()),
        }
    }

    fn get_fn_compiled_name(&self, name: &str) -> Result<String, Error> {
        match self.fn_map.get(name) {
            Some(id) => Ok(format!("usr_fn_{}", id)),
            None => Err(format!("No user function named \"{}\"", name).into())
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
    additional_vars: Vec<VarData>,
    next_additional_id: usize,
}

impl<'a> LocalRegistry<'a> {
    fn new(parent: &'a GlobalRegistry, vars: &[String]) -> Self {
        LocalRegistry {
            parent,
            map: vars.iter().enumerate()
                .map(|(i, v)| (v.clone(), VarData::new(i, false)))
                .collect(),
            next_additional_id: vars.len(),
            additional_vars: Vec::new(),
        }
    }

    fn compiled_var_names(&self) -> Vec<String> {
        self.map.values()
            .chain(self.additional_vars.iter())
            .map(|data| vec!(data.ty_name.clone(), data.val_name.clone()))
            .flatten().collect()
    }

    fn compile_val(&self, val: &hir::Val) -> Result<CompiledVal, Error> {
        match val {
            hir::Val::Nothing => {
                Ok(CompiledVal::new(
                    self.parent.get_ty_id(&ConcreteType::Nothing)?,
                    lir::Val::Const(0),
                ))
            },
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
            None => self.parent.get_var(name),
        }
    }

    fn mk_additional_var(&mut self) -> VarData {
        let data = VarData::new(self.next_additional_id, false);
        self.additional_vars.push(VarData::new(self.next_additional_id, false));
        self.next_additional_id += 1;
        data
    }
}

fn compile_call(
    global: &GlobalRegistry,
    local: &mut LocalRegistry,
    dest: &hir::LValue,
    call: &hir::Callable
) -> Result<Vec<lir::Statement>, Error> {
    let mut out = Vec::new();
    let mut maybe_store = Vec::new();
    
    let dest_var = match dest {
        hir::LValue::Var(dest) => local.get_var(dest)?,
        hir::LValue::Access(_, _, _) => {
            let data = local.mk_additional_var();
            maybe_store.push(data);
            maybe_store.last().unwrap()
        },
    };

    match call {
        hir::Callable::Call(fn_name, native, args) => {
            let mut vars = Vec::new();
            
            args.iter().try_for_each(|arg| -> Result<(), Error> {
                let arg = local.compile_val(arg)?;
                vars.push(arg.ty); vars.push(arg.val);
                Ok(())
            })?;

            out.push(lir::Statement::Inst(lir::Instruction::Call(
                Some((dest_var.ty_name.clone(), dest_var.val_name.clone())),
                *native,
                if *native {
                    format!("native_{}", fn_name)
                } else {
                    global.get_fn_compiled_name(fn_name)?
                },
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
                    dest_var.ty_name.clone(),
                    global.get_ty_id(&dest_ty)?
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Bin(
                    dest_var.val_name.clone(),
                    lir_op,
                    local.compile_val(a)?.val,
                    local.compile_val(b)?.val,
                )
            ));
        },
        hir::Callable::Unary(op, a) => {
            let (dest_ty, lir_op) = match op {
                hir::UnaryOp::Neg => (ConcreteType::Int64, lir::UnaryOp::Neg),
                hir::UnaryOp::Not => (ConcreteType::Bool, lir::UnaryOp::Not),
            };

            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    dest_var.ty_name.clone(),
                    global.get_ty_id(&dest_ty)?
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Unary(
                    dest_var.val_name.clone(),
                    lir_op,
                    local.compile_val(a)?.val,
                )
            ));
        },
        hir::Callable::Assign(a) => {
            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    dest_var.ty_name.clone(),
                    local.compile_val(a)?.ty,
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    dest_var.val_name.clone(),
                    local.compile_val(a)?.val,
                )
            ));
        },
        hir::Callable::Alloc(structure) => {
            out.push(lir::Statement::Inst(
                lir::Instruction::Call(
                    Some((dest_var.ty_name.clone(), dest_var.val_name.clone())),
                    true,
                    "native_alloc".to_string(),
                    vec!(
                        global.get_ty_id(&ConcreteType::Struct(structure.clone()))?,
                        lir::Val::Const(global.get_struct(structure)?.get_mem_len() as u64),
                    )
                )
            ));
        },
        hir::Callable::IsType(a, t) => {
            out.push(lir::Statement::Inst(
                lir::Instruction::Mov(
                    dest_var.ty_name.clone(),
                    global.get_ty_id(&ConcreteType::Bool)?,
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Bin(
                    dest_var.val_name.clone(),
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
                    dest_var.ty_name.clone(),
                    local.compile_val(a)?.val,
                    data.ty_addr(),
                )
            ));
            out.push(lir::Statement::Inst(
                lir::Instruction::Access(
                    dest_var.val_name.clone(),
                    local.compile_val(a)?.val,
                    data.val_addr(),
                )
            ));
        },
    }

    if let hir::LValue::Access(x, structure, field) = dest {
        let data = global.get_struct(structure)?.get_field(field)?;

        out.push(lir::Statement::Inst(
            lir::Instruction::AssignArray(
                local.compile_val(x)?.val,
                data.ty_addr(),
                lir::Val::Var(dest_var.ty_name.clone())
            )
        ));
        out.push(lir::Statement::Inst(
            lir::Instruction::AssignArray(
                local.compile_val(x)?.val,
                data.val_addr(),
                lir::Val::Var(dest_var.val_name.clone())
            )
        ));
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
    local: &mut LocalRegistry,
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
    local: &mut LocalRegistry,
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
    local: &mut LocalRegistry,
    block: &hir::Block,
) -> Result<Vec<lir::Statement>, Error> {
    let mut out = Vec::new();

    block.stmts.iter().try_for_each(|stmt| -> Result<(), Error> {
            match stmt {
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
    let mut local = LocalRegistry::new(&global, &f.vars);
    let mut lbl_gen = LabelGenerator::new();

    let body = compile_block(&mut lbl_gen, &global, &mut local, &f.body)?;

    let mut lir_args = Vec::new();
    f.args.iter().try_for_each(|arg| -> Result<(), Error> {
        let arg = local.get_var(arg)?;
        lir_args.push(arg.ty_name.clone());
        lir_args.push(arg.val_name.clone());
        Ok(())
    })?;
    
    Ok(lir::Function::new(
        global.get_fn_compiled_name(&f.name)?,
        lir_args,
        local.compiled_var_names(),
        lir::Block::new(body),
    ))
}

pub fn hir_to_lir(hir: &hir::Source) -> Result<lir::Source, Error> {
    let mut compiled = Vec::new();

    let structs = hir.decls.iter().filter_map(|d| {
            match d {
                hir::Decl::Struct(s) => Some(s),
                _ => None
            }
        }).collect::<Vec<_>>();

    let functions = hir.decls.iter().filter_map(|d| {
            match d {
                hir::Decl::Function(decl) => Some(decl.name.as_str()),
                _ => None
            }
        }).collect::<Vec<_>>();

    let global = GlobalRegistry::new(&hir.globals, &structs, &functions);

    let main_fn = match hir.decls.iter().find_map(|d| {
            match d {
                hir::Decl::Function(f) if f.name == "main" => Some(f),
                _ => None
            }
        })
    {
        Some(f) => f,
        None => Err("No \"main\" function".to_string())?
    };

    hir.decls.iter().try_for_each(|d| -> Result<(), Error> {
            match d {
                hir::Decl::Struct(_) => (),
                hir::Decl::Function(f) => compiled.push(compile_fn(&global, f)?),
            }
            Ok(())
        })?;

    compiled.push(lir::Function::new(
        "main".to_string(),
        vec!(),
        vec!("ret_code_ty".to_string(), "ret_code_val".to_string()),
        lir::Block::new(vec!(
            lir::Statement::Inst(lir::Instruction::Call(
                    Some(("ret_code_ty".to_string(), "ret_code_val".to_string())),
                    false,
                    global.get_fn_compiled_name(&hir.entrypoint)?,
                    vec!()
            )),
            lir::Statement::Inst(lir::Instruction::Return(
                lir::Val::Var("ret_code_ty".to_string()),
                lir::Val::Var("ret_code_val".to_string()),
            ))
        ))
    ));

    Ok(lir::Source::new(global.compiled_var_names(), compiled))
}

