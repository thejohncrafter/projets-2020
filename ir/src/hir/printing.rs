
use super::types::*;

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int64 => {
                write!(f, "Int64")
            },
            Type::Bool => {
                write!(f, "Bool")
            },
            Type::Str => {
                write!(f, "Str")
            },
            Type::Struct(id) => {
                write!(f, "Struct {}", id)
            },
        }
    }
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Var(name) => write!(f, "{}", name),
            Val::Const(u, v) => write!(f, "{{{}, {}}}", u, v),
        }
    }
}

impl std::fmt::Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! cases {
                ($(($variant:ident, $symbol:expr)),*) => {
                    match self {
                        $(Callable::$variant(a, b) => {
                        write!(f, "{} {} {};", a, $symbol, b)?;
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
            Callable::Assign(v) => {
                write!(f, "{}", v)?;
            },
            Callable::Cast(v, t) => {
                write!(f, "({}) {}", t, v)?;
            },
            Callable::Access(v, i) => {
                write!(f, "{}[{}]", v, i)?;
            },

            _ => ()
        }

        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_block(f: &mut std::fmt::Formatter<'_>, indent: usize, b: &Block) -> std::fmt::Result {
            b.stmts.iter().try_for_each(|stmt| {
                match stmt {
                    Statement::Call(dest, c) => {
                        writeln!(f, "{:indent$}{} <- {}", "", dest, c, indent=(4*indent))?;
                    },
                    Statement::Return(v) => {
                        writeln!(f, "{:indent$}return {}", "", v, indent=(4*indent))?;
                    },

                    Statement::If(v, b1, b2) => {
                        writeln!(f, "{:indent$}if {} {{", "", v, indent=(4*indent))?;
                        print_block(f, indent + 1, b1)?;
                        writeln!(f, "{:indent$}}}else {{", "", indent=(4*indent))?;
                        print_block(f, indent + 1, b2)?;
                        writeln!(f, "{:indent$}}}", "", indent=(4*indent))?;
                    },
                    Statement::While(v, b1) => {
                        writeln!(f, "{:indent$}while {} {{", "", v, indent=(4*indent))?;
                        print_block(f, indent + 1, b1)?;
                        writeln!(f, "{:indent$}}}", "", indent=(4*indent))?;
                    },
                }

                Ok(())
            })?;


            Ok(())
        }

        writeln!(
            f, "fn {}({}) {{",
            self.name,
            self.args.iter().enumerate().map(|(i, arg)| if i == 0 {
                    format!("{}", arg)
                } else {
                    format!(", {}", arg)
                }).collect::<String>()
        )?;

        print_block(f, 1, &self.body)?;

        write!(f, "}}")?;

        Ok(())
    }
}

