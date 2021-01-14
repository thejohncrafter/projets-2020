
use automata::lexer::*;
use automata::parser::*;
use automata::line_counter::*;
use automata::read_error::*;

use parsergen::{lex, parse};

use super::types::*;

enum BinOp {
    Equ,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,

    And,
    Or,

    Add,
    Sub,
    Mul,
    Div,
}

enum Punct {
    LBracket,
    RBracket,
    LPar,
    RPar,
    LSquare,
    RSquare,
    Comma,
    Colon,
    Semicolon,

    Arrow,

    Equ,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,

    And,
    Or,

    Add,
    Sub,
    Mul,
    Div,

    Not,
}

enum Token {
    Ident(String),
    Num(u64),
    Str(String),
    Punct(Punct),
}

pub fn parse_lir<'a>(file_name: &'a str, contents: &'a str) -> Result<Vec<Function>, ReadError<'a>> {
    let chars = LineIter::new(contents);
    let input = IndexedString::new(file_name, contents);

    fn parse_u64(text: &str) -> Result<u64, String> {
        text.parse().map_err(|_| "This number does not fit in 64 bits.".to_string())
    }

    macro_rules! punct {
        ($variant:ident) => {Ok(Some(Token::Punct(Punct::$variant)))};
    }

    let dfa: DFA<LineIter, IndexedString, Option<Token>, ReadError> = lex! {
        chars: {chars}
        input: {&input}

        ((' ' | '\t' | '\n') & (' ' | '\t' | '\n')*) => {
            Ok(None)
        },
        ('#' & behaved* & '\n') => {
            Ok(None)
        },

        ((alpha | '_') & (alpha | '_' | num)*) => {
            Ok(Some(Token::Ident($text.to_string())))
        },
        (num & num*) => {
            Ok(Some(Token::Num(parse_u64($text)?)))
        },
        ('"' & (behaved | '\\' & ('\\' | '"' | 'n' | 't'))* & '"') => {
            Ok(Some(Token::Str($text.to_string())))
        },

        ('{') => {punct!(LBracket)},
        ('}') => {punct!(RBracket)},
        ('(') => {punct!(LPar)},
        (')') => {punct!(RPar)},
        ('[') => {punct!(LSquare)},
        (']') => {punct!(RSquare)},
        (',') => {punct!(Comma)},
        (':') => {punct!(Colon)},
        (';') => {punct!(Semicolon)},

        ('<' & '-') => {punct!(Arrow)},

        ('=' & '=') => {punct!(Equ)},
        ('!' & '=') => {punct!(Neq)},
        ('<') => {punct!(Lt)},
        ('<' & '=') => {punct!(Leq)},
        ('>') => {punct!(Gt)},
        ('>' & '=') => {punct!(Geq)},

        ('&' & '&') => {punct!(And)},
        ('|' & '|') => {punct!(Or)},

        ('+') => {punct!(Add)},
        ('-') => {punct!(Sub)},
        ('*') => {punct!(Mul)},
        ('%') => {punct!(Div)},

        ('!') => {punct!(Not)},
    };
 
    let tokens = dfa.filter_map(|x| {
        match x {
            Ok((span, x)) => {
                match x {
                    TokenOrEof::Token(Some(x)) => Some(Ok((span, Some(x)))),
                    TokenOrEof::Token(None) => None,
                    TokenOrEof::Eof => Some(Ok((span, None)))
                }
            },
            Err(e) => Some(Err(e)) // We will handle that later.
        }
    });

    macro_rules! make_instruction {
        ($op:ident, $dest:ident, $a:ident, $b:ident, $($id:ident),*) => {
            match $op {
                $(BinOp::$id => Ok(Statement::Inst(Instruction::$id($dest, $a, $b)))),*,
            }
        };
    }
    
    let res = parse! {
        src_lifetime: 'a
        span: Span<'a>

        terms: [
            ident: String,
            uint: u64,

            FN: (),
            JUMP: (),
            JUMPIF: (),
            CALL: (),
            RETURN: (),

            LBRACKET: (),
            RBRACKET: (),
            LPAR: (),
            RPAR: (),
            LSQUARE: (),
            RSQUARE: (),
            COMMA: (),
            COLON: (),
            SEMICOLON: (),

            ARROW: (),

            EQU: (),
            NEQ: (),
            LT: (),
            LEQ: (),
            GT: (),
            GEQ: (),

            AND: (),
            OR: (),

            ADD: (),
            SUB: (),
            MUL: (),
            DIV: (),

            NOT: (),
        ]
        nterms: [
            functions_list: Vec<Function>,

            ident_list: Vec<String>,
            val_list: Vec<Val>,
            function_head: (String, Vec<String>),

            function: Function,
            block: Vec<Statement>,
            statement: Statement,
            statement_semi: Statement,
            bin_op: BinOp,
            val: Val,
        ]

        tokens: {
            tokens.map(move |x| -> Result<(Span, _), ReadError> {
                let (span, x) = x.unwrap();
                
                if let Some(x) = x {
                    let token = match x {
                        Token::Ident(name) => {
                            match name.as_str() {
                                "fn" => $FN(()),
                                "jump" => $JUMP(()),
                                "jumpif" => $JUMPIF(()),
                                "call" => $CALL(()),
                                "return" => $RETURN(()),

                                _ => $ident(name)
                            }
                        },
                        Token::Num(i) => $uint(i),
                        Token::Punct(p) => {
                            use Punct::*;

                            match p {
                                LBracket => $LBRACKET(()),
                                RBracket => $RBRACKET(()),
                                LPar => $LPAR(()),
                                RPar => $RPAR(()),
                                LSquare => $LSQUARE(()),
                                RSquare => $RSQUARE(()),
                                Comma => $COMMA(()),
                                Colon => $COLON(()),
                                Semicolon => $SEMICOLON(()),

                                Arrow => $ARROW(()),

                                Equ => $EQU(()),
                                Neq => $NEQ(()),
                                Lt => $LT(()),
                                Leq => $LEQ(()),
                                Gt => $GT(()),
                                Geq => $GEQ(()),

                                And => $AND(()),
                                Or => $OR(()),

                                Add => $ADD(()),
                                Sub => $SUB(()),
                                Mul => $MUL(()),
                                Div => $DIV(()),

                                Not => $NOT(()),
                            }
                        },
                        _ => panic!(),
                    };
                    Ok((span, TokenOrEof::Token(token)))
                } else {
                    Ok((span, TokenOrEof::Eof))
                }
            })
        }

        rules: {
            (functions_list -> f:function) => {
                Ok(vec!($f))
            },
            (functions_list -> v:functions_list f:function) => {
                let mut v = $v;
                v.push($f);
                Ok(v)
            },

            (ident_list -> id:ident) => {
                Ok(vec!($id))
            },
            (ident_list -> v:ident_list COMMA id:ident) => {
                let mut v = $v;
                v.push($id);
                Ok(v)
            },

            (val_list -> val:val) => {
                Ok(vec!($val))
            },
            (val_list -> v:val_list COMMA val:val) => {
                let mut v = $v;
                v.push($val);
                Ok(v)
            },

            (function_head -> FN id:ident LPAR RPAR) => {
                Ok(($id, vec!()))
            },
            (function_head -> FN id:ident LPAR vars:ident_list RPAR) => {
                Ok(($id, $vars))
            },

            (function -> head:function_head LBRACKET RBRACKET) => {
                Ok(Function::new($head.0, $head.1, Block::new(vec!())))
            },
            (function -> head:function_head LBRACKET body:block RBRACKET) => {
                Ok(Function::new($head.0, $head.1, Block::new($body)))
            },

            (block -> s:statement) => {
                Ok(vec!($s))
            },
            (block -> v:block s:statement) => {
                let mut v = $v;
                v.push($s);
                Ok(v)
            },

            (statement -> l:ident COLON) => {
                Ok(Statement::Label(Label::new($l)))
            },

            (statement -> s:statement_semi SEMICOLON) => {
                Ok($s)
            },

            (statement_semi -> dest:ident ARROW a:val op:bin_op b:val) => {
                make_instruction!(
                    $op, $dest, $a, $b,
                    Equ, Neq, Lt, Leq, Gt, Geq,
                    And, Or,
                    Add, Sub, Mul, Div
                )
            },

            (statement_semi -> dest:ident ARROW v:val) => {
                Ok(Statement::Inst(Instruction::Mov($dest, $v)))
            },

            (statement_semi -> dest:ident ARROW p:val LSQUARE i:uint RSQUARE) => {
                Ok(Statement::Inst(Instruction::Access($dest, $p, $i)))
            },

            (statement_semi -> JUMP l:ident) => {
                Ok(Statement::Inst(Instruction::Jump(Label::new($l))))
            },
            (statement_semi -> JUMPIF cond:val l:ident) => {
                Ok(Statement::Inst(Instruction::Jumpif($cond, Label::new($l))))
            },

            (statement_semi -> dest:ident ARROW CALL f:ident LBRACKET RBRACKET) => {
                Ok(Statement::Inst(Instruction::Call($dest, $f, vec!())))
            },
            (statement_semi -> dest:ident ARROW CALL f:ident LBRACKET v:val_list RBRACKET) => {
                Ok(Statement::Inst(Instruction::Call($dest, $f, $v)))
            },

            (statement_semi -> RETURN v0:val COMMA v1:val) => {
                Ok(Statement::Inst(Instruction::Return($v0, $v1)))
            },

            (bin_op -> EQU) => {Ok(BinOp::Equ)},
            (bin_op -> NEQ) => {Ok(BinOp::Neq)},
            (bin_op -> LT ) => {Ok(BinOp::Lt)},
            (bin_op -> LEQ) => {Ok(BinOp::Leq)},
            (bin_op -> GT ) => {Ok(BinOp::Gt)},
            (bin_op -> GEQ) => {Ok(BinOp::Geq)},
            
            (bin_op -> AND) => {Ok(BinOp::And)},
            (bin_op -> OR ) => {Ok(BinOp::Or)},

            (bin_op -> ADD) => {Ok(BinOp::Add)},
            (bin_op -> SUB) => {Ok(BinOp::Sub)},
            (bin_op -> MUL) => {Ok(BinOp::Mul)},
            (bin_op -> DIV) => {Ok(BinOp::Div)},

            (val -> id:ident) => {
                Ok(Val::Var($id))
            },
            (val -> i:uint) => {
                Ok(Val::Const($i))
            },
        }

        on_empty: {Err("Expected a program".to_string())}
        start: functions_list
    };

    res
}

