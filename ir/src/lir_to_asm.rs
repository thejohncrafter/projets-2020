
use std::collections::HashMap;
use std::fmt::Write;

use super::lir::types::*;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error {msg: error}
    }
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Error {msg: format!("{}", error)}
    }
}

fn extract_ids(f: &Function) -> (HashMap<String, usize>, HashMap<String, usize>) {
    struct Receiver {
        var_map: HashMap<String, usize>,
        next_var_id: usize,
        label_map: HashMap<String, usize>,
        next_label_id: usize,
    }

    impl Receiver {
        fn new() -> Self {
            Receiver {
                var_map: HashMap::new(),
                next_var_id: 0,
                label_map: HashMap::new(),
                next_label_id: 0,
            }
        }

        fn recv_name(&mut self, name: &String) {
            if !self.var_map.contains_key(name) {
                self.var_map.insert(name.clone(), self.next_var_id);
                self.next_var_id += 1;
            }
        }

        fn recv(&mut self, v: &Val) {
            match v {
                Val::Var(name) => {
                    self.recv_name(name)
                },
                _ => ()
            }
        }

        fn recv_label(&mut self, label: &String) {
            if !self.label_map.contains_key(label) {
                self.label_map.insert(label.clone(), self.next_label_id);
                self.next_label_id += 1;
            }
        }
    }

    let mut recv = Receiver::new();

    f.args.iter().for_each(|arg| recv.recv_name(arg));
    f.body.stmts.iter().for_each(|stmt| match stmt {
            Statement::Inst(inst) => {
                match inst {
                    Instruction::Bin(dest, _, a, b) => {
                        recv.recv_name(dest);
                        recv.recv(a); recv.recv(b);
                    },
                    Instruction::Mov(dest, a) => {
                        recv.recv_name(dest);
                        recv.recv(a);
                    },
                    Instruction::Access(dest, a, _) => {
                        recv.recv_name(dest);
                        recv.recv(a);
                    },
                    Instruction::Jump(_) => (),
                    Instruction::Jumpif(a, _) => {
                        recv.recv(a);
                    },
                    Instruction::Call(dest, _, v) => {
                        if let Some((dest1, dest2)) = dest {
                            recv.recv_name(dest1);
                            recv.recv_name(dest2);
                        }
                        v.iter().for_each(|a| recv.recv(a));
                    },
                    Instruction::Return(a, b) => {
                        recv.recv(a); recv.recv(b);
                    }
                }
            },
            Statement::Label(label) => {
                recv.recv_label(&label.name)
            },
        });

    (recv.var_map, recv.label_map)
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

fn write_get_val(
    asm: &mut String,
    reg: &mut StringRegistry,
    var_ids: &HashMap<String, usize>,
    val: &Val,
    dest: &str
) -> Result<(), Error> {
    match val {
        Val::Var(name) => {
            writeln!(asm, "\tmovq {}(%rsp), {}", 8 * var_ids.get(name).unwrap(), dest)?
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
    var_ids: &HashMap<String, usize>,
    fn_id: usize,
    inst: &Instruction
) -> Result<(), Error> {
    match inst {
        Instruction::Bin(dest, op, a, b) => {
            write_get_val(asm, reg, var_ids, a, "%rax")?;
            write_get_val(asm, reg, var_ids, b, "%rbx")?;

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

            writeln!(asm, "\tmovq %rax, {}(%rsp)", 8 * var_ids.get(dest).unwrap())?;
        },
        Instruction::Mov(dest, a) => {
            write_get_val(asm, reg, var_ids, a, "%rax")?;
            writeln!(asm, "\tmovq %rax, {}(%rsp)", 8 * var_ids.get(dest).unwrap())?;
        },
        Instruction::Access(dest, a, offset) => {
            write_get_val(asm, reg, var_ids, a, "%rax")?;
            writeln!(asm, "\tmov %{}, {}(%rax)", dest, offset)?;
        },
        Instruction::Jump(label) => {
            writeln!(asm, "\tjmp fn_{}_lbl_{}", fn_id, label_ids.get(&label.name).unwrap())?;
        },
        Instruction::Jumpif(a, label) => {
            write_get_val(asm, reg, var_ids, a, "%rax")?;
            writeln!(asm, "\tmovq $0, %rbx")?;
            writeln!(asm, "\tcmp %rax, %rbx")?;
            
            if let Some(label_id) = label_ids.get(&label.name) {
                writeln!(asm, "\tjnz fn_{}_lbl_{}", fn_id, label_id)?;
            } else {
                Err(format!("No label named \"{}\"", label.name))?
            }
        },
        Instruction::Call(dest, fn_name, args) => {
            let stack_extra = if args.len() >= 6 {
                    if (args.len() - 6) % 2 == 0 {
                        8 * (args.len() - 6)
                    } else {
                        8 * (args.len() - 5)
                    }
                } else {0};

            // Reserve space for arguments
            if stack_extra != 0 {
                writeln!(asm, "\tsubq ${}, %rsp", stack_extra)?;
            }

            // Store the arguments
            args.iter().enumerate().try_for_each(|(i, arg)| -> Result<(), Error> {
                    match arg {
                        Val::Var(name) => {
                            writeln!(
                                asm, "\tmovq {}(%rsp), %rax",
                                8 * var_ids.get(name).unwrap() + stack_extra
                            )?;
                        },
                        Val::Const(i) => {
                            writeln!(asm, "\tmovq ${}, %rax", i)?;
                        },
                        Val::Str(s) => {
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
            match fn_name.as_str() {
                "print_int" => {
                    if args.len() != 1 {
                        Err("Expected 1 argument for \"print_int\".".to_string())?
                    }

                    writeln!(asm, "\tcall print_int")?
                },
                "print_string" => {
                    if args.len() != 1 {
                        Err("Expected 1 argument for \"print_string\".".to_string())?
                    }

                    writeln!(asm, "\tcall print_string")?
                },
                _ => {
                    if let Some(id) = fn_ids.get(fn_name) {
                        writeln!(asm, "\tcall fn_{}", id)?
                    } else {
                        Err(format!("No function named \"{}\".", fn_name))?
                    }
                }
            }

            // Free the space we reserved for arguments
            if stack_extra != 0 {
                writeln!(asm, "\taddq ${}, %rsp", stack_extra)?;
            }

            if let Some((dest1, dest2)) = dest {
                // Get the return values
                writeln!(asm, "\tmovq %rax, {}(%rsp)", 8 * var_ids.get(dest1).unwrap())?;
                writeln!(asm, "\tmovq %rdx, {}(%rsp)", 8 * var_ids.get(dest2).unwrap())?;
            }
        },
        Instruction::Return(u, v) => {
            write_get_val(asm, reg, var_ids, u, "%rax")?;
            write_get_val(asm, reg, var_ids, v, "%rdx")?;
            writeln!(asm, "\tjmp fn_{}_exit", fn_id)?;
        },
    }

    Ok(())
}

fn fn_to_asm(
    asm: &mut String,
    reg: &mut StringRegistry,
    fn_ids: &HashMap<String, usize>,
    f: &Function,
    id: usize
) -> Result<(), Error> {
    let (var_ids, label_ids) = extract_ids(f);
    let var_count = var_ids.len();
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
            writeln!(asm, "\tmovq %rax, {}(%rsp)", 8 * var_ids.get(arg).unwrap())?;
            Ok(())
        })?;

    f.body.stmts.iter().try_for_each(|stmt| -> Result<(), Error> {
            match stmt {
                Statement::Inst(inst) => {
                    inst_to_asm(asm, reg, fn_ids, &label_ids, &var_ids, id, inst)
                },
                Statement::Label(label) => {
                    writeln!(asm, "fn_{}_lbl_{}:", id, label_ids.get(&label.name).unwrap())
                        .map_err(|e| e.into())
                },
            }
        })?;

    // Restore previous frame
    writeln!(asm, "fn_{}_exit:", id)?;
    writeln!(asm, "\taddq ${}, %rsp", frame_size)?;
    writeln!(asm, "\tpopq %rbp")?;
    writeln!(asm, "\tret")?;

    Ok(())
}

pub fn lir_to_asm(fns: &[Function]) -> Result<String, Error> {
    let mut s = String::new();
    let asm = &mut s;
    let mut fn_ids = HashMap::new();
    let mut reg = StringRegistry::new();

    fns.iter().enumerate().try_for_each(|(i, f)| {
            if fn_ids.contains_key(&f.name) {
                Err(format!("Function \"{}\" is not uniquely defined.", f.name))
            } else {
                fn_ids.insert(f.name.clone(), i);
                Ok(())
            }
        })?;

    let main_id = match fn_ids.get("main") {
        Some(id) => id,
        None => Err("No \"main\" function !".to_string())?
    };

    writeln!(asm, "\t.text")?;
    writeln!(asm, "\t.globl main")?;
    writeln!(asm, "main:")?;
    writeln!(asm, "\tcall fn_{}", main_id)?;
    writeln!(asm, "\tmovq $0, %rax")?;
    writeln!(asm, "\tret")?;

    writeln!(asm, "print_int:")?;
    writeln!(asm, "\tmov %rdi, %rsi")?;
    writeln!(asm, "\tmov $message_int, %rdi")?;
    writeln!(asm, "\tmov $0, %rax")?;
    writeln!(asm, "\tcall printf")?;
    writeln!(asm, "\tmov $0, %rax")?;
    writeln!(asm, "\tmov $0, %rdx")?;
    writeln!(asm, "\tret")?;

    writeln!(asm, "print_string:")?;
    writeln!(asm, "\tmov %rdi, %rsi")?;
    writeln!(asm, "\tmov $message_string, %rdi")?;
    writeln!(asm, "\tmov $0, %rax")?;
    writeln!(asm, "\tcall printf")?;
    writeln!(asm, "\tmov $0, %rax")?;
    writeln!(asm, "\tmov $0, %rdx")?;
    writeln!(asm, "\tret")?;

    fns.iter().enumerate().try_for_each(
            |(i, f)| fn_to_asm(asm, &mut reg, &fn_ids, f, i)
        )?;

    writeln!(asm, "\t.data")?;
    writeln!(asm, "message_int:")?;
    writeln!(asm, "\t.string \"%d\\n\"")?;
    writeln!(asm, "message_string:")?;
    writeln!(asm, "\t.string \"%s\\n\"")?;

    reg.strings.iter().enumerate().try_for_each(|(i, s)| -> Result<(), Error> {
            writeln!(asm, "string_{}:", i)?;
            writeln!(asm, "\t.string {:?}", s)?;
            Ok(())
        })?;

    Ok(s)
}

