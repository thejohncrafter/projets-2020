
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
    Dot,
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

pub fn parse_hir<'a>(file_name: &'a str, contents: &'a str) -> Result<Source, ReadError<'a>> {
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
        ('.') => {punct!(Dot)},
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
    
    let res = parse! {
        src_lifetime: 'a
        span: Span<'a>

        terms: [
            ident: String,
            uint: u64,
            string: String,

            GLOBALS: (),
            FN: (),
            VARS: (),
            CALL: (),
            NATIVE: (),
            RETURN: (),
            IF: (),
            ELSE: (),
            WHILE: (),
            TYPEOF: (),

            LBRACKET: (),
            RBRACKET: (),
            LPAR: (),
            RPAR: (),
            LSQUARE: (),
            RSQUARE: (),
            DOT: (),
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
            ident_list: Vec<String>,
            val_list: Vec<Val>,
            function_head: (String, Vec<String>),
            vars_list: Vec<String>,
            statements_list: Vec<Statement>,
            call_head: bool,
            globals: Vec<String>,
            decls_list: Vec<Decl>,

            source: Source,
            decl: Decl,
            structure: StructDecl,
            function: Function,
            block: Block,
            statement: Statement,
            statement_semi: Statement,
            callable: Callable,
            bin_op: BinOp,
            unary_op: UnaryOp,
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
                                "globals" => $GLOBALS(()),
                                "fn" => $FN(()),
                                "vars" => $VARS(()),
                                "call" => $CALL(()),
                                "native" => $NATIVE(()),
                                "return" => $RETURN(()),
                                "if" => $IF(()),
                                "else" => $ELSE(()),
                                "while" => $WHILE(()),
                                "typeof" => $TYPEOF(()),

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
                                Dot => $DOT(()),
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
                        Token::Str(s) => {
                            $string(s)
                        },
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

            (statements_list -> s:statement) => {
                Ok(vec!($s))
            },
            (statements_list -> v:statements_list s:statement) => {
                let mut v = $v;
                v.push($s);
                Ok(v)
            },

            (call_head -> CALL) => {Ok(false)},
            (call_head -> CALL NATIVE) => {Ok(true)},

            (globals -> GLOBALS COLON SEMICOLON) => {
                Ok(vec!())
            },
            (globals -> GLOBALS COLON v:ident_list SEMICOLON) => {
                Ok($v)
            },

            (decls_list -> d:decl) => {
                Ok(vec!($d))
            },
            (decls_list -> v:decls_list d:decl) => {
                let mut v = $v;
                v.push($d);
                Ok(v)
            },

            (source -> globals:globals) => {
                Ok(Source::new($globals, vec!()))
            },
            (source -> globals:globals v:decls_list) => {
                Ok(Source::new($globals, $v))
            },

            (decl -> s:structure) => {
                Ok(Decl::Struct($s))
            },
            (decl -> f:function) => {
                Ok(Decl::Function($f))
            },

            (structure -> STRUCT name:ident LBRACKET RBRACKET) => {
                Ok(StructDecl::new($name, vec!()))
            },
            (structure -> STRUCT name:ident LBRACKET fields:ident_list RBRACKET) => {
                Ok(StructDecl::new($name, $fields))
            },

            (function -> h:function_head vars:vars_list b:block) => {
                Ok(Function::new($h.0, $h.1, $vars, $b))
            },

            (block -> LBRACKET RBRACKET) => {
                Ok(Block::new(vec!()))
            },
            (block -> LBRACKET l:statements_list RBRACKET) => {
                Ok(Block::new($l))
            },

            (statement -> dest:ident ARROW c:callable SEMICOLON) => {
                Ok(Statement::Call($dest, $c))
            },
            (statement -> RETURN v:val SEMICOLON) => {
                Ok(Statement::Return($v))
            },
            
            (statement -> IF v:val b1:block ELSE b2:block) => {
                Ok(Statement::If($v, $b1, $b2))
            },

            (statement -> WHILE v:val b:block) => {
                Ok(Statement::While($v, $b))
            },

            (callable -> native:call_head fn_name:ident LPAR RPAR) => {
                Ok(Callable::Call($fn_name, $native, vec!()))
            },
            (callable -> native:call_head f_name:ident LPAR v:val_list RPAR) => {
                Ok(Callable::Call($f_name, $native, $v))
            },

            (callable -> a:val op:bin_op b:val) => {
                Ok(Callable::Bin($op, $a, $b))
            },

            (callable -> op:unary_op a:val) => {
                Ok(Callable::Unary($op, $a))
            },

            (callable -> a:val) => {
                Ok(Callable::Assign($a))
            },

            (callable -> TYPEOF v:val EQU t:ty) => {
                Ok(Callable::IsType($v, $t))
            },

            (callable -> v:val LSQUARE structure:ident DOT field:ident RSQUARE) => {
                Ok(Callable::Access($v, $structure, $field))
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

            (unary_op -> SUB) => {Ok(UnaryOp::Neg)},
            (unary_op -> NOT) => {Ok(UnaryOp::Not)},

            (val -> id:ident) => {
                Ok(Val::Var($id))
            },
            (val -> LPAR t:ty COMMA v:uint RPAR) => {
                Ok(Val::Const($t, $v))
            },
            (val -> s:string) => {
                Ok(Val::Str($s))
            },

            (ty -> LPAR RPAR) => {Ok(Type::Nothing)},
            (ty -> INT64) => {Ok(Type::Int64)},
            (ty -> BOOL) => {Ok(Type::Bool)},
            (ty -> STR) => {Ok(Type::Str)},
            (ty -> STRUCT name:ident) => {Ok(Type::Struct($name))},
        }

        on_empty: {Err("Expected a program".to_string())}
        start: source
    };

    res
}

