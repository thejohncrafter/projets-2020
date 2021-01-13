
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

pub fn parse_hir<'a>(file_name: &'a str, contents: &'a str) -> Result<Vec<Function>, ReadError<'a>> {
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
                _ => panic!()
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
            IF: (),
            ELSE: (),
            WHILE: (),

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

            INT64: (),
            BOOL: (),
            STR: (),
            STRUCT: (),
        ]
        nterms: [
            functions_list: Vec<Function>,

            ident_list: Vec<String>,
            val_list: Vec<Val>,
            function_head: (String, Vec<String>),
            statements_list: Vec<Statement>,

            function: Function,
            block: Block,
            statement: Statement,
            statement_semi: Statement,
            callable: Callable,
            val: Val,
            ty: Type,
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
                                "if" => $IF(()),
                                "else" => $ELSE(()),
                                "while" => $WHILE(()),

                                "Int64" => $INT64(()),
                                "Bool" => $BOOL(()),
                                "Str" => $STR(()),
                                "struct" => $STRUCT(()),

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

            (statements_list -> s:statement) => {
                Ok(vec!($s))
            },
            (statements_list -> v:statements_list s:statement) => {
                let mut v = $v;
                v.push($s);
                Ok(v)
            },

            (block -> LBRACKET RBRACKET) => {
                Ok(Block::new(vec!()))
            },
            (block -> LBRACKET l:statements_list RBRACKET) => {
                Ok(Block::new($l))
            },

            (function -> h:function_head b:block) => {
                Ok(Function::new($h.0, $h.1, $b))
            },

            (statement -> dest:ident ARROW c:callable SEMICOLON) => {
                Ok(Statement::Call($dest, $c))
            },
            (statement -> RETURN v:val) => {
                Ok(Statement::Return($v))
            },
            
            (statement -> IF v:val b1:block ELSE b2:block) => {
                Ok(Statement::If($v, $b1, $b2))
            },

            (statement -> WHILE v:val b:block) => {
                Ok(Statement::While($v, $b))
            },

            (callable -> a:val EQU b:val) => {Ok(Callable::Equ($a, $b))},
            (callable -> a:val NEQ b:val) => {Ok(Callable::Neq($a, $b))},
            (callable -> a:val LT  b:val) => {Ok(Callable::Lt($a, $b))},
            (callable -> a:val LEQ b:val) => {Ok(Callable::Leq($a, $b))},
            (callable -> a:val GT  b:val) => {Ok(Callable::Gt($a, $b))},
            (callable -> a:val GEQ b:val) => {Ok(Callable::Geq($a, $b))},
            
            (callable -> a:val AND b:val) => {Ok(Callable::And($a, $b))},
            (callable -> a:val OR  b:val) => {Ok(Callable::Or($a, $b))},

            (callable -> a:val ADD b:val) => {Ok(Callable::Add($a, $b))},
            (callable -> a:val SUB b:val) => {Ok(Callable::Sub($a, $b))},
            (callable -> a:val MUL b:val) => {Ok(Callable::Mul($a, $b))},
            (callable -> a:val DIV b:val) => {Ok(Callable::Div($a, $b))},

            (callable -> a:val) => {
                Ok(Callable::Assign($a))
            },

            (callable -> LPAR t:ty RPAR v:val) => {
                Ok(Callable::Cast($v, $t))
            },

            (callable -> v:val LSQUARE i:uint RSQUARE) => {
                Ok(Callable::Access($v, $i))
            },

            (val -> id:ident) => {
                Ok(Val::Var($id))
            },
            (val -> LPAR u:uint COMMA v:uint RPAR) => {
                Ok(Val::Const($u, $v))
            },

            (ty -> INT64) => {Ok(Type::Int64)},
            (ty -> BOOL) => {Ok(Type::Bool)},
            (ty -> STR) => {Ok(Type::Str)},
            (ty -> STRUCT name:ident) => {Ok(Type::Struct($name))},
        }

        on_empty: {Err("Expected a program".to_string())}
        start: functions_list
    };

    res
}
