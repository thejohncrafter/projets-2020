
use std::fmt::Write;

use super::types::*;

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Var(name) => write!(f, "{}", name),
            Val::Const(i) => write!(f, "{}", i),
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! cases {
            ($(($variant:ident, $symbol:expr)),*) => {
                match self {
                    $(Instruction::$variant(dest, a, b) => {
                        writeln!(f, "\t{} <- {} {} {};", dest, a, $symbol, b)?;
                    },)*
                    _ => ()
                }
            };
        }

        cases!(
            (And, "&&"),
            (Or,  "||"),

            (Equ, "=="),
            (Neq, "!="),
            (Lt,  "<"),
            (Leq, "<="),
            (Gt,  ">"),
            (Geq, ">="),

            (Add, "+"),
            (Sub, "-"),
            (Mul, "*"),
            (Div, "%")
        );

        match self {
            Instruction::Mov(dest, v) => {
                writeln!(f, "\t{} <- {}", dest, v)?
            },
            Instruction::Access(dest, s, i) => {
                writeln!(f, "\t{} <- {}[{}];", dest, s, i)?
            },
            Instruction::Jump(l) => {
                writeln!(f, "\tjump {};", l.name)?
            },
            Instruction::Jumpif(a, l) => {
                writeln!(f, "\tjumpif {} {};", a, l.name)?
            },
            Instruction::Call(dest, fn_name, args) => {
                writeln!(
                    f, "\t{} <- {}({});",
                    dest, fn_name,
                    args.iter().enumerate().map(|(i, a)| if i == 0 {
                            format!("{}", a)
                        } else {
                            format!(", {}", a)
                        }).collect::<String>()
                )?
            },
            Instruction::Return(v0, v1) => {
                writeln!(f, "\treturn {}, {};", v0, v1)?
            },

            _ => ()
        }

        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f, "fn {}({}) {{",
            self.name,
            self.args.iter().enumerate().map(|(i, a)| if i == 0 {
                    a.clone()
                } else {
                    format!(", {}", a)
                }).collect::<String>()
        )?;

        self.body.stmts.iter().try_for_each(|stmt| {
                match stmt {
                    Statement::Label(label) => {
                        writeln!(f, "{}:", label.name)?;
                    },
                    Statement::Inst(inst) => {
                        write!(f, "{}", inst)?;
                    },
                }

                Ok(())
            })?;

        writeln!(f, "}}")?;

        Ok(())
    }
}
