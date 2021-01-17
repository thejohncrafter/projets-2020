
use std::collections::HashMap;
use std::fmt::Write;

use super::lir::types::*;
use super::error::*;

struct GlobalRegistry {
    var_ids: HashMap<String, usize>,
}

impl GlobalRegistry {
    fn new(vars: &[String]) -> Self {
        GlobalRegistry {
            var_ids: vars.iter().enumerate()
                .map(|(i, v)| (v.clone(), i))
                .collect(),
        }
    }

    fn get_var_access(&self, name: &str) -> Result<String, Error> {
        match self.var_ids.get(name) {
            Some(i) => Ok(format!("(global_var_{})", i)),
            None => Err(format!("[LIR] Variable {} was not declared", name).into())
        }
    }

    fn compile_vars_decls(&self, asm: &mut String) -> Result<(), Error> {
        self.var_ids.values().try_for_each(|id| -> Result<(), Error> {
                writeln!(asm, "global_var_{}:", id)?;
                writeln!(asm, "\t.quad 0")?;
                Ok(())
            })?;
        Ok(())
    }
}

fn extract_labels(f: &Function) -> HashMap<String, usize> {
    struct Receiver {
        map: HashMap<String, usize>,
        next_id: usize,
    }

    impl Receiver {
        fn new() -> Self {
            Receiver {
                map: HashMap::new(),
                next_id: 0,
            }
        }

        fn recv_label(&mut self, label: &String) {
            if !self.map.contains_key(label) {
                self.map.insert(label.clone(), self.next_id);
                self.next_id += 1;
            }
        }
    }

    let mut recv = Receiver::new();

    f.body.stmts.iter().for_each(|stmt| match stmt {
            Statement::Label(label) => {
                recv.recv_label(&label.name)
            },
            _ => ()
        });

    recv.map
}

struct StringRegistry {
    strings: Vec<String>,
    next_id: usize,
}

impl StringRegistry {
    fn new() -> Self {
        StringRegistry {
            strings: Vec::new(),
            next_id: 0,
        }
    }

    /*
     * Registers a String `s` and returns the ID `s` was given.
     */
    fn register(&mut self, s: String) -> usize {
        let id = self.next_id;
        self.strings.push(s);
        self.next_id += 1;
        id
    }
}

struct LocalRegistry<'a> {
    parent: &'a GlobalRegistry,
    map: HashMap<String, usize>,
}

impl<'a> LocalRegistry<'a> {
    fn new(parent: &'a GlobalRegistry, vars: &[String]) -> Self {
        LocalRegistry {
            parent,
            map: vars.iter().enumerate()
                .map(|(i, name)| (name.clone(), i))
                .collect(),
        }
    }

    fn get_var_count(&self) -> usize {
        self.map.len()
    }

    fn get_var_access_with_extra(&self, stack_extra: usize, name: &str) -> Result<String, Error> {
        match self.map.get(name) {
            Some(i) => Ok(format!("{}(%rsp)", 8 * i + stack_extra)),
            None => self.parent.get_var_access(name),
        }
    }

    fn get_var_access(&self, name: &str) -> Result<String, Error> {
        self.get_var_access_with_extra(0, name)
    }
}

fn write_get_val(
    asm: &mut String,
    reg: &mut StringRegistry,
    local: &LocalRegistry,
    val: &Val,
    dest: &str
) -> Result<(), Error> {
    match val {
        Val::Var(name) => {
            writeln!(asm, "\tmovq {}, {}", local.get_var_access(name)?, dest)?
        },
        Val::Const(i) => {
            writeln!(asm, "\tmovq ${}, {}", i, dest)?
        },
        Val::Str(s) => {
            let id = reg.register(s.clone());
            writeln!(asm, "\tmovq $string_{}, %rax", id)?;
        },
    }

    Ok(())
}

fn inst_to_asm(
    asm: &mut String,
    reg: &mut StringRegistry,
    fn_ids: &HashMap<String, usize>,
    label_ids: &HashMap<String, usize>,
    local: &LocalRegistry,
    fn_id: usize,
    inst: &Instruction
) -> Result<(), Error> {
    match inst {
        Instruction::Bin(dest, op, a, b) => {
            write_get_val(asm, reg, local, a, "%rax")?;
            write_get_val(asm, reg, local, b, "%rbx")?;

            match op {
                BinOp::And => writeln!(asm, "\tandq %rbx, %rax")?,
                BinOp::Or => writeln!(asm, "\torq %rbx, %rax")?,

                BinOp::Equ => {
                    writeln!(asm, "\tcmp %rbx, %rax")?;
                    writeln!(asm, "\tmovq $0, %rax")?;
                    writeln!(asm, "\tsete %al")?;
                },
                BinOp::Neq => {
                    writeln!(asm, "\tcmp %rbx, %rax")?;
                    writeln!(asm, "\tmovq $0, %rax")?;
                    writeln!(asm, "\tsetne %al")?;
                },
                BinOp::Lt => {
                    writeln!(asm, "\tcmp %rbx, %rax")?;
                    writeln!(asm, "\tmovq $0, %rax")?;
                    writeln!(asm, "\tsetl %al")?;
                },
                BinOp::Leq => {
                    writeln!(asm, "\tcmp %rbx, %rax")?;
                    writeln!(asm, "\tmovq $0, %rax")?;
                    writeln!(asm, "\tsetle %al")?;
                },
                BinOp::Gt => {
                    writeln!(asm, "\tcmp %rbx, %rax")?;
                    writeln!(asm, "\tmovq $0, %rax")?;
                    writeln!(asm, "\tsetg %al")?;
                },
                BinOp::Geq => {
                    writeln!(asm, "\tcmp %rbx, %rax")?;
                    writeln!(asm, "\tmovq $0, %rax")?;
                    writeln!(asm, "\tsetge %al")?;
                },

                BinOp::Add => writeln!(asm, "\taddq %rbx, %rax")?,
                BinOp::Sub => writeln!(asm, "\tsubq %rbx, %rax")?,
                BinOp::Mul => writeln!(asm, "\timulq %rbx, %rax")?,
                BinOp::Div => {
                    writeln!(asm, "\tcqto")?;
                    writeln!(asm, "\tidivq %rbx")?;
                },
            }

            writeln!(asm, "\tmovq %rax, {}", local.get_var_access(dest)?)?;
        },
        Instruction::Unary(dest, op, a) => {
            write_get_val(asm, reg, local, a, "%rax")?;

            match op {
                UnaryOp::Neg => {
                    writeln!(asm, "\tnegq %rax")?;
                },
                UnaryOp::Not => {
                    writeln!(asm, "\tnotq %rax")?;
                    writeln!(asm, "\tandq $0, %rax")?;
                },
            }

            writeln!(asm, "\tmovq %rax, {}", local.get_var_access(dest)?)?;
        },
        Instruction::Mov(dest, a) => {
            write_get_val(asm, reg, local, a, "%rax")?;
            writeln!(asm, "\tmovq %rax, {}", local.get_var_access(dest)?)?;
        },
        Instruction::AssignArray(dest, offset, a) => {
            write_get_val(asm, reg, local, a, "%rax")?;
            write_get_val(asm, reg, local, dest, "%rbx")?;
            writeln!(asm, "\tmov %rax, {}(%rbx)", offset)?;
        },
        Instruction::Access(dest, a, offset) => {
            write_get_val(asm, reg, local, a, "%rax")?;
            writeln!(asm, "\tmov {}(%rax), %rbx", offset)?;
            writeln!(asm, "\tmov %rbx, {}", local.get_var_access(dest)?)?;
        },
        Instruction::Jump(label) => {
            writeln!(asm, "\tjmp fn_{}_lbl_{}", fn_id, label_ids.get(&label.name).unwrap())?;
        },
        Instruction::Jumpif(a, label) => {
            write_get_val(asm, reg, local, a, "%rax")?;
            writeln!(asm, "\tmovq $0, %rbx")?;
            writeln!(asm, "\tcmp %rax, %rbx")?;
            
            if let Some(label_id) = label_ids.get(&label.name) {
                writeln!(asm, "\tjnz fn_{}_lbl_{}", fn_id, label_id)?;
            } else {
                Err(format!("[LIR] No label named \"{}\"", label.name))?
            }
        },
        Instruction::JumpifNot(a, label) => {
            write_get_val(asm, reg, local, a, "%rax")?;
            writeln!(asm, "\tmovq $0, %rbx")?;
            writeln!(asm, "\tcmp %rax, %rbx")?;
            
            if let Some(label_id) = label_ids.get(&label.name) {
                writeln!(asm, "\tjz fn_{}_lbl_{}", fn_id, label_id)?;
            } else {
                Err(format!("[LIR] No label named \"{}\"", label.name))?
            }
        },
        Instruction::Call(dest, native, fn_name, args) => {
            enum UsrOrNative<'a> {
                Usr(&'a Val),
                Native(bool),
            }

            let args: Vec<UsrOrNative> = if *native {
                vec!(UsrOrNative::Native(true), UsrOrNative::Native(false)).into_iter()
                    .chain(args.iter().map(|a| UsrOrNative::Usr(a)))
                    .collect()
            } else {
                args.iter().map(|a| UsrOrNative::Usr(a)).collect()
            };

            let stack_extra = if args.len() >= 6 {
                    if (args.len() - 6) % 2 == 0 {
                        8 * (args.len() - 6)
                    } else {
                        8 * (args.len() - 5)
                    }
                } else {0} + if *native {16} else {0};

            // Reserve space for arguments
            if stack_extra != 0 {
                writeln!(asm, "\tsubq ${}, %rsp", stack_extra)?;
            }

            // Store the arguments
            args.iter().enumerate().try_for_each(|(i, arg)| -> Result<(), Error> {
                    match arg {
                        UsrOrNative::Native(b) => {
                            writeln!(
                                asm, "\tmovq %rsp, %rax"
                            )?;
                            if *b {
                                writeln!(
                                    asm, "\taddq ${}, %rax",
                                    stack_extra - 8
                                )?;
                            } else {
                                writeln!(
                                    asm, "\taddq ${}, %rax",
                                    stack_extra - 16
                                )?;
                            }
                        },
                        UsrOrNative::Usr(Val::Var(name)) => {
                            writeln!(
                                asm, "\tmovq {}, %rax",
                                local.get_var_access_with_extra(stack_extra, name)?
                            )?;
                        },
                        UsrOrNative::Usr(Val::Const(i)) => {
                            writeln!(asm, "\tmovq ${}, %rax", i)?;
                        },
                        UsrOrNative::Usr(Val::Str(s)) => {
                            let id = reg.register(s.clone());
                            writeln!(asm, "\tmovq $string_{}, %rax", id)?;
                        },
                    }

                    match i {
                        0 => writeln!(asm, "\tmovq %rax, %rdi")?,
                        1 => writeln!(asm, "\tmovq %rax, %rsi")?,
                        2 => writeln!(asm, "\tmovq %rax, %rdx")?,
                        3 => writeln!(asm, "\tmovq %rax, %rcx")?,
                        4 => writeln!(asm, "\tmovq %rax, %r8")?,
                        5 => writeln!(asm, "\tmovq %rax, %r9")?,
                        _ => writeln!(asm, "\tmovq %rax, {}(%rsp)", 8 * (i - 6))?,
                    }

                    Ok(())
                })?;

            // Call the function
            if *native {
                writeln!(asm, "\tcall {}", fn_name)?;
            } else {
                if let Some(id) = fn_ids.get(fn_name) {
                    writeln!(asm, "\tcall fn_{}", id)?
                } else {
                    Err(format!("[LIR] No function named \"{}\".", fn_name))?
                }
            }

            // Get the native return values
            if *native {
                writeln!(asm, "\tmovq {}(%rsp), %rax", stack_extra - 8)?;
                writeln!(asm, "\tmovq {}(%rsp), %rdx", stack_extra - 16)?;
            }

            // Free the space we reserved for arguments
            if stack_extra != 0 {
                writeln!(asm, "\taddq ${}, %rsp", stack_extra)?;
            }

            if let Some((dest1, dest2)) = dest {
                // Get the return values
                writeln!(asm, "\tmovq %rax, {}", local.get_var_access(dest1)?)?;
                writeln!(asm, "\tmovq %rdx, {}", local.get_var_access(dest2)?)?;
            }
        },
        Instruction::Return(u, v) => {
            write_get_val(asm, reg, local, u, "%rax")?;
            write_get_val(asm, reg, local, v, "%rdx")?;
            writeln!(asm, "\tjmp fn_{}_exit", fn_id)?;
        },
    }

    Ok(())
}

fn fn_to_asm(
    asm: &mut String,
    reg: &mut StringRegistry,
    fn_ids: &HashMap<String, usize>,
    global: &GlobalRegistry,
    f: &Function,
    id: usize
) -> Result<(), Error> {
    let label_ids = extract_labels(f);
    let local = LocalRegistry::new(global, &f.vars);
    let var_count = local.get_var_count();
    let frame_size = 8 * if var_count % 2 == 0 {var_count} else {var_count + 1};

    // Declare function
    writeln!(asm, "fn_{}:", id)?;

    // Create new frame
    writeln!(asm, "\tpushq %rbp")?;
    writeln!(asm, "\tmovq %rsp, %rbp")?;
    writeln!(asm, "\tsubq ${}, %rsp", frame_size)?;

    // Push all the arguments on the stack
    f.args.iter().enumerate().try_for_each(|(i, arg)| -> Result<(), Error> {
            match i {
                0 => writeln!(asm, "\tmovq %rdi, %rax")?,
                1 => writeln!(asm, "\tmovq %rsi, %rax")?,
                2 => writeln!(asm, "\tmovq %rdx, %rax")?,
                3 => writeln!(asm, "\tmovq %rcx, %rax")?,
                4 => writeln!(asm, "\tmovq %r8, %rax")?,
                5 => writeln!(asm, "\tmovq %r9, %rax")?,
                _ => writeln!(asm, "\tmovq {}(%rbp), %rax", 8 * (i - 4))?,
                    // i - 4 = i - 6 + 2
                    // (-6 as we store on the stack from the 7th argument on,
                    // and +2 because the two last elements on the stack store
                    // informations about the last frame)
            }
            writeln!(asm, "\tmovq %rax, {}", local.get_var_access(arg)?)?;
            Ok(())
        })?;

    f.body.stmts.iter().try_for_each(|stmt| -> Result<(), Error> {
            match stmt {
                Statement::Inst(inst) => {
                    inst_to_asm(asm, reg, fn_ids, &label_ids, &local, id, inst)
                },
                Statement::Label(label) => {
                    writeln!(asm, "fn_{}_lbl_{}:", id, label_ids.get(&label.name).unwrap())
                        .map_err(|e| e.into())
                },
            }
        })?;

    // Return Nothing by default
    writeln!(asm, "\tmovq $0, %rax")?;
    writeln!(asm, "\tmovq $0, %rdx")?;

    // Restore previous frame
    writeln!(asm, "fn_{}_exit:", id)?;
    writeln!(asm, "\taddq ${}, %rsp", frame_size)?;
    writeln!(asm, "\tpopq %rbp")?;
    writeln!(asm, "\tret")?;

    Ok(())
}

pub fn lir_to_asm(source: &Source) -> Result<String, Error> {
    let mut s = String::new();
    let asm = &mut s;
    
    let global = GlobalRegistry::new(&source.globals);
    let mut fn_ids = HashMap::new();
    let mut reg = StringRegistry::new();

    source.functions.iter().enumerate().try_for_each(|(i, f)| {
            if fn_ids.contains_key(&f.name) {
                Err(format!("[LIR] Function \"{}\" is not uniquely defined.", f.name))
            } else {
                fn_ids.insert(f.name.clone(), i);
                Ok(())
            }
        })?;

    let main_id = match fn_ids.get("main") {
        Some(id) => id,
        None => Err("[LIR] No \"main\" function !".to_string())?
    };

    writeln!(asm, "\t.text")?;
    writeln!(asm, "\t.globl main")?;
    writeln!(asm, "main:")?;
    writeln!(asm, "\tcall fn_{}", main_id)?;
    writeln!(asm, "\tmovq $0, %rax")?;
    writeln!(asm, "\tret")?;

    source.functions.iter().enumerate().try_for_each(
            |(i, f)| fn_to_asm(asm, &mut reg, &fn_ids, &global, f, i)
        )?;

    writeln!(asm, "\t.data")?;

    reg.strings.iter().enumerate().try_for_each(|(i, s)| -> Result<(), Error> {
            writeln!(asm, "string_{}:", i)?;
            writeln!(asm, "\t.string {:?}", s)?;
            Ok(())
        })?;

    global.compile_vars_decls(asm)?;

    Ok(s)
}

