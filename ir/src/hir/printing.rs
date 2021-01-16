
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
            Type::Nothing => {
                write!(f, "()")
            }
        }
    }
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Nothing => write!(f, "nothing"),
            Val::Var(name) => write!(f, "{}", name),
            Val::Const(u, v) => write!(f, "({}, {})", u, v),
            Val::Str(s) => write!(f, "{:?}", s),
        }
    }
}

impl std::fmt::Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Call(fn_name, args) => {
                writeln!(
                    f, "call {}({})",
                    fn_name,
                    args.iter().enumerate().map(|(i, arg)| if i == 0 {
                            format!("{}", arg)
                        } else {
                            format!(", {}", arg)
                        }).collect::<String>()
                )?;
            },
            Callable::Bin(op, a, b) => {
                macro_rules! cases {
                        ($(($variant:ident, $symbol:expr)),*) => {
                            match op {
                                $(BinOp::$variant => {
                                write!(f, "{} {} {};", a, $symbol, b)?;
                            },)*
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
            },
            Callable::Assign(v) => {
                write!(f, "{}", v)?;
            },
            Callable::IsType(v, t) => {
                write!(f, "typeof {} == {}", v, t)?;
            },
            Callable::Access(v, structure, field) => {
                write!(f, "{}[{}.{}]", v, structure, field)?;
            },
        }

        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn print_block(f: &mut std::fmt::Formatter<'_>, indent: usize, b: &Block) -> std::fmt::Result {
            b.stmts.iter().try_for_each(|stmt| {
                match stmt {
                    Statement::FnCall(fn_name, args) => {
                        writeln!(
                            f, "{:indent$}call {}({})",
                            "",
                            fn_name,
                            args.iter().enumerate().map(|(i, arg)| if i == 0 {
                                    format!("{}", arg)
                                } else {
                                    format!(", {}", arg)
                                }).collect::<String>(),
                            indent=(4*indent)
                        )?;
                    },
                    Statement::Call(dest, c) => {
                        writeln!(f, "{:indent$}{} <- {}", "", dest, c, indent=(4*indent))?;
                    },
                    Statement::Return(v) => {
                        writeln!(f, "{:indent$}return {};", "", v, indent=(4*indent))?;
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
            f, "fn {}({})",
            self.name,
            self.args.iter().enumerate().map(|(i, arg)| if i == 0 {
                    format!("{}", arg)
                } else {
                    format!(", {}", arg)
                }).collect::<String>()
        )?;

        writeln!(
            f, "    vars: {};",
            self.vars.iter().enumerate().map(|(i, var)| if i == 0 {
                    format!("{}", var)
                } else {
                    format!(", {}", var)
                }).collect::<String>()
        )?;

        writeln!(f, "{{")?;

        print_block(f, 1, &self.body)?;

        write!(f, "}}")?;

        Ok(())
    }
}

impl std::fmt::Display for StructDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "struct {} {{", self.name)?;
        write!(f, "    ")?;

        self.fields.iter().enumerate().try_for_each(|(i, name)| if i == 0 {
                write!(f, "{}", name)
            } else {
                write!(f, ", {}", name)
            })?;

        writeln!(f)?;
        writeln!(f, "}}")?;

        Ok(())
    }
}

impl std::fmt::Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decl::Struct(s) => write!(f, "{}", s),
            Decl::Function(function) => write!(f, "{}", function),
        }
    }
}
