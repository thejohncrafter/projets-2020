
use automata::lexer::*;
use automata::parser::*;
use automata::line_counter::*;
use automata::read_error::*;

use parsergen::{lex, parse};

use super::types::*;

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
    Mod,

    Not,
}

enum Token {
    Ident(String),
    Num(i64),
    Str(String),
    Punct(Punct),
}

pub fn parse_lir<'a>(file_name: &'a str, contents: &'a str) -> Result<Source, ReadError<'a>> {
    let chars = LineIter::new(contents);
    let input = IndexedString::new(file_name, contents);

    fn parse_i64(text: &str) -> Result<i64, String> {
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
            Ok(Some(Token::Num(parse_i64($text)?)))
        },
        ('"' & (behaved | '\\' & ('\\' | '"' | 'n' | 't'))* & '"') => {
            let mut v = Vec::new();
            let mut chars = $text.chars();
            chars.next().unwrap();

            loop {
                let c = chars.next().unwrap();

                let d = if c == '\\' {
                    match chars.next().unwrap() {
                        '\\' => '\\',
                        '"' => '"',
                        'n' => '\n',
                        't' => '\t',
                        _ => panic!()
                    }
                } else if c == '"' {
                    break
                } else { c };

                v.push(d);
            }

            Ok(Some(Token::Str(v.into_iter().collect())))
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
        ('%') => {punct!(Mod)},

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
    
    let res = parse! {
        src_lifetime: 'a
        span: Span<'a>

        terms: [
            ident: String,
            uint: i64,
            string: String,

            GLOBALS: (),
            FN: (),
            VARS: (),
            JUMP: (),
            JUMPIF: (),
            CALL: (),
            NATIVE: (),
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
            MOD: (),

            NOT: (),
        ]
        nterms: [
            ident_list: Vec<String>,
            val_list: Vec<Val>,
            function_head: (String, Vec<String>),
            vars_list: Vec<String>,
            call_head: bool,
            globals: Vec<String>,
            functions_list: Vec<Function>,

            source: Source,
            function: Function,
            block: Vec<Statement>,
            statement: Statement,
            statement_semi: Statement,
            bin_op: BinOp,
            unary_op: UnaryOp,
            val: Val,
        ]

        tokens: {
            tokens.map(move |x| -> Result<(Span, _), ReadError> {
                let (span, x) = x.unwrap();
                
                if let Some(x) = x {
                    let token = match x {
                        Token::Ident(name) => {
                            match name.as_str() {
                                "globals" => $GLOBALS(()),
                                "fn" => $FN(()),
                                "vars" => $VARS(()),
                                "jump" => $JUMP(()),
                                "jumpif" => $JUMPIF(()),
                                "call" => $CALL(()),
                                "native" => $NATIVE(()),
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
                                Mod => $MOD(()),

                                Not => $NOT(()),
                            }
                        },
                        Token::Str(s) => $string(s),
                    };
                    Ok((span, TokenOrEof::Token(token)))
                } else {
                    Ok((span, TokenOrEof::Eof))
                }
            })
        }

        rules: {
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

            (vars_list -> VARS COLON SEMICOLON) => {
                Ok(vec!())
            },
            (vars_list -> VARS COLON vars:ident_list SEMICOLON) => {
                Ok($vars)
            },

            (call_head -> CALL) => {Ok(false)},
            (call_head -> CALL NATIVE) => {Ok(true)},

            (globals -> GLOBALS COLON SEMICOLON) => {
                Ok(vec!())
            },
            (globals -> GLOBALS COLON v:ident_list SEMICOLON) => {
                Ok($v)
            },

            (functions_list -> f:function) => {
                Ok(vec!($f))
            },
            (functions_list -> v:functions_list f:function) => {
                let mut v = $v;
                v.push($f);
                Ok(v)
            },

            (source -> globals:globals) => {
                Ok(Source::new($globals, vec!()))
            },
            (source -> globals:globals v:functions_list) => {
                Ok(Source::new($globals, $v))
            },

            (function -> head:function_head vars:vars_list LBRACKET RBRACKET) => {
                Ok(Function::new($head.0, $head.1, $vars, Block::new(vec!())))
            },
            (function -> head:function_head vars:vars_list LBRACKET body:block RBRACKET) => {
                Ok(Function::new($head.0, $head.1, $vars, Block::new($body)))
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
                Ok(Statement::Inst(Instruction::Bin($dest, $op, $a, $b)))
            },

            (statement_semi -> dest:ident ARROW op:unary_op a:val) => {
                Ok(Statement::Inst(Instruction::Unary($dest, $op, $a)))
            },

            (statement_semi -> dest:ident ARROW v:val) => {
                Ok(Statement::Inst(Instruction::Mov($dest, $v)))
            },

            (statement_semi -> dest:val LSQUARE offset:uint RSQUARE ARROW v:val) => {
                Ok(Statement::Inst(Instruction::AssignArray($dest, $offset, $v)))
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
            (statement_semi -> JUMPIF NOT cond:val l:ident) => {
                Ok(Statement::Inst(Instruction::JumpifNot($cond, Label::new($l))))
            },

            (statement_semi -> native:call_head f:ident LPAR RPAR) => {
                Ok(Statement::Inst(Instruction::Call(None, $native, $f, vec!())))
            },
            (statement_semi -> native:call_head f:ident LPAR v:val_list RPAR) => {
                Ok(Statement::Inst(Instruction::Call(None, $native, $f, $v)))
            },
            (statement_semi -> LPAR dest1:ident COMMA dest2:ident RPAR ARROW native:call_head f:ident LPAR RPAR) => {
                Ok(Statement::Inst(Instruction::Call(Some(($dest1, $dest2)), $native, $f, vec!())))
            },
            (statement_semi -> LPAR dest1:ident COMMA dest2:ident RPAR ARROW native:call_head f:ident LPAR v:val_list RPAR) => {
                Ok(Statement::Inst(Instruction::Call(Some(($dest1, $dest2)), $native, $f, $v)))
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
            (bin_op -> MOD) => {Ok(BinOp::Mod)},

            (unary_op -> SUB) => {Ok(UnaryOp::Neg)},
            (unary_op -> NOT) => {Ok(UnaryOp::Not)},

            (val -> id:ident) => {
                Ok(Val::Var($id))
            },
            (val -> i:uint) => {
                Ok(Val::Const($i))
            },
            (val -> s:string) => {
                Ok(Val::Str($s))
            }
        }

        on_empty: {Err("Expected a program".to_string())}
        start: source
    };

    res
}

