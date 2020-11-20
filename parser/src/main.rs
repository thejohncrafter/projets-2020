
mod ast;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{Arg, App, SubCommand};

use automata::lexer::*;
use automata::parser::*;
use automata::line_counter::*;
use automata::read_error::*;

use parsergen::{lex, parse};

use ast::*;

#[derive(Debug)]
enum Keyword {
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    If,
    Mutable,
    Return,
    Struct,
    True,
    While,
}

#[derive(Debug)]
enum Punct {
    LPar,
    RPar,
    Comma,
    Colon,
    DoubleColon,
    Semicolon,

    Equ,
    DoubleEqu,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,

    And,
    Or,

    Plus,
    Minus,
    Times,
    Div,

    Not,

    Pow,
    
    Dot,
}

#[derive(Debug)]
enum Token {
    Int(i64),
    Str(String),
    Ident(String),

    IntIdent(i64, String),
    IntLPar(i64),
    IdentLPar(String),
    RParIdent(String),

    Keyword(Keyword),
    Punct(Punct),
}

#[derive(Debug)]
enum PreToken {
    None,
    Newline,
    Token(Token),
}

fn parse<'a>(file_name: &'a str, contents: &'a str) -> Result<Vec<Decl<'a>>, ReadError<'a>> {
    let chars = LineIter::new(contents);
    let input = IndexedString::new(file_name, contents);

    fn parse_i64(text: &str) -> Result<i64, String> {
        text.parse().map_err(|_| "This number does not fit in 64 bits.".to_string())
    }

    enum IdentOrKeyword {
        Ident(String),
        Keyword(Keyword),
    }

    impl IdentOrKeyword {
        fn expect_ident(self) -> Result<String, String> {
            use IdentOrKeyword::*;

            match self {
                Ident(id) => Ok(id),
                Keyword(_) => Err("Expected an identifier, found a keyword.".to_string())
            }
        }
        
        fn into_pre_token(self) -> PreToken {
            use IdentOrKeyword::*;

            PreToken::Token(match self {
                Ident(id) => Token::Ident(id),
                Keyword(kw) => Token::Keyword(kw),
            })
        }
    }

    fn ident_or_keyword(text: &str) -> IdentOrKeyword {
        use IdentOrKeyword::{Ident, Keyword as Kw};

        match text {
            "else" => Kw(Keyword::Else),
            "elseif" => Kw(Keyword::Elseif),
            "end" => Kw(Keyword::End),
            "false" => Kw(Keyword::False),
            "for" => Kw(Keyword::For),
            "function" => Kw(Keyword::Function),
            "if" => Kw(Keyword::If),
            "mutable" => Kw(Keyword::Mutable),
            "return" => Kw(Keyword::Return),
            "struct" => Kw(Keyword::Struct),
            "true" => Kw(Keyword::True),
            "while" => Kw(Keyword::While),
            _ => Ident(text.to_string())
        }
    }

    macro_rules! punct {
        ($variant:ident) => {Ok(PreToken::Token(Token::Punct(Punct::$variant)))};
    }

    let mut dfa: DFA<LineIter, IndexedString, PreToken, ReadError> = lex! {
        chars: {chars}
        input: {&input}

        ((' ' | '\t') & (' ' | '\t')*) => {Ok(PreToken::None)},
        ('#' & (behaved | '\\' | '"')* & '\n') => {Ok(PreToken::Newline)}, 
        ('\n') => {Ok(PreToken::Newline)},

        ((alpha | '_') & (alpha | '_' | num)*) => {
            Ok(ident_or_keyword($text).into_pre_token())
        },
        (num & num*) => {Ok(PreToken::Token(Token::Int({
            parse_i64($text)?
        })))},
        ('"' & (behaved | '\\' & ('\\' | '"' | 'n' | 't'))* & '"') => {
            let mut v = Vec::new();
            let mut chars = $text.chars();

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

            Ok(PreToken::Token(Token::Str(v.into_iter().collect())))
        },

        (num & num* & alpha & (alpha | num)*) => {
            let i = $text.chars().enumerate()
                .find_map(|(i, c)| {
                    if !(c == '-' || c.is_ascii_digit()) {Some(i)}
                    else {None}
                })
                .unwrap();

            let k = parse_i64(&$text.chars().take(i).collect::<String>())?;
            let id = ident_or_keyword(&$text.chars().skip(i).collect::<String>())
                .expect_ident()?;

            Ok(PreToken::Token(Token::IntIdent(k, id)))
        },
        ((alpha | '_') & (alpha | '_' | num)* & '(') => {
            let last = $text.chars().enumerate().map(|(i, _)| i).last().unwrap();
            Ok(PreToken::Token(Token::IdentLPar(
                ident_or_keyword(&$text.chars().take(last).collect::<String>())
                    .expect_ident()?
            )))
        },
        (num & num* & '(') => {
            let last = $text.chars().enumerate().map(|(i, _)| i).last().unwrap();
            Ok(PreToken::Token(Token::IntLPar(parse_i64(
                    &$text.chars().take(last).collect::<String>()
                )?)))
        },
        (')' & (alpha | '_') & (alpha | '_' | num)*) => {
            Ok(PreToken::Token(Token::RParIdent(
                ident_or_keyword(&$text.chars().skip(1).collect::<String>())
                    .expect_ident()?
            )))
        },

        ('(') => {punct!(LPar)},
        (')') => {punct!(RPar)},
        (',') => {punct!(Comma)},
        (':') => {punct!(Colon)},
        (':' & ':') => {punct!(DoubleColon)},
        (';') => {punct!(Semicolon)},
        
        ('=') => {punct!(Equ)},
        ('=' & '=') => {punct!(DoubleEqu)},
        ('!' & '=') => {punct!(Neq)},
        ('<') => {punct!(Lt)},
        ('<' & '=') => {punct!(Leq)},
        ('>') => {punct!(Gt)},
        ('>' & '=') => {punct!(Geq)},

        ('&' & '&') => {punct!(And)},
        ('|' & '|') => {punct!(Or)},

        ('+') => {punct!(Plus)},
        ('-') => {punct!(Minus)},
        ('*') => {punct!(Times)},
        ('%') => {punct!(Div)},

        ('!') => {punct!(Not)},
       
        ('^') => {punct!(Pow)},

        ('.') => {punct!(Dot)},
    };

    struct Adapter<'a, I> {
        inner: &'a mut I,
        can_add_semi: bool,
        saw_else: bool,
    }

    /*
     * Eliminates whitespaces and inserts semicolons.
     */
    impl<'a, I> Adapter<'a, I> {
        fn new(inner: &'a mut I) -> Self {
            Adapter {inner, can_add_semi: false, saw_else: false}
        }
    }

    impl<'a, 'b, I> Iterator for Adapter<'a, I>
        where I: Iterator<Item = Result<(Span<'b>, TokenOrEof<PreToken>), ReadError<'b>>>
    {
        type Item = Result<(Span<'b>, Option<Token>), ReadError<'b>>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let (span, item) = match self.inner.next()? {
                    Ok(item) => item,
                    Err(e) => return Some(Err(e))
                };

                match item {
                    TokenOrEof::Token(token) => {
                        match token {
                            PreToken::None => continue,
                            PreToken::Newline => {
                                if self.can_add_semi {
                                    self.can_add_semi = false;
                                    return Some(Ok((span, Some(Token::Punct(Punct::Semicolon)))))
                                } else {
                                    continue
                                }
                            },
                            PreToken::Token(token) => {
                                self.can_add_semi = match token { // Semicolon insertion.
                                    Token::Ident(_) => true,
                                    Token::Int(_) => true,
                                    Token::IntIdent(_, _) => true,
                                    Token::RParIdent(_) => true,
                                    Token::Str(_) => true,
                                    Token::Keyword(Keyword::True) => true,
                                    Token::Keyword(Keyword::False) => true,
                                    Token::Punct(Punct::RPar) => true,
                                    Token::Keyword(Keyword::End) => true,
                                    _ => false
                                };

                                if let Token::Keyword(Keyword::If) = token {
                                    if self.saw_else {
                                        return Some(Err((
                                            span,
                                            "Illegal \"if\" after \"else\" (please use \"elif\").".to_string()
                                        ).into()))
                                    }
                                }

                                if let Token::Keyword(Keyword::Else) = token {
                                    self.saw_else = true;
                                } else {
                                    self.saw_else = false;
                                }

                                return Some(Ok((span, Some(token))))
                            }
                        }
                    },
                    TokenOrEof::Eof => return Some(Ok((span, None)))
                }
            }
        }
    }

    let tokens = Adapter::new(&mut dfa);

    let ast = parse! {
        src_lifetime: 'a
        span: Span<'a>

        terms: [
            int: i64,
            string: String,
            ident: String,
            intident: (i64, String),
            intlpar: i64,
            identlpar: String,
            rparident: String,
            
            ELSE: (),
            ELSEIF: (),
            END: (),
            FALSE: (),
            FOR: (),
            FUNCTION: (),
            IF: (),
            MUTABLE: (),
            RETURN: (),
            STRUCT: (),
            TRUE: (),
            WHILE: (),

            LPAR: (),
            RPAR: (),
            COMMA: (),
            COLON: (),
            DOUBLECOLON: (),
            SEMICOLON: (),

            EQU: (),
            DOUBLEEQU: (),
            NEQ: (),
            LT: (),
            LEQ: (),
            GT: (),
            GEQ: (),

            AND: (),
            OR: (),

            PLUS: (),
            MINUS: (),
            TIMES: (),
            DIV: (),

            NOT: (),

            POW: (),

            DOT: (),
        ]
        nterms: [
            file: Vec<Decl<'a>>,

            located_ident: LocatedIdent<'a>,

            decl: Decl<'a>,
           
            param: Param<'a>,
            params: Vec<Param<'a>>,

            fields: Vec<Param<'a>>,
            struct_head: bool,
            structure: Structure<'a>,

            function_head: (String, Vec<Param<'a>>),
            function_signature: (String, Vec<Param<'a>>, Option<LocatedIdent<'a>>),
            function: Function<'a>,

            range: Range<'a>,

            comparison_op: BinOp,
            sum_op: BinOp,
            product_op: BinOp,

            // "clean" expressions are expressions that do not
            // start with a "-"
            // (defined in order to unambiguously define rules
            // where "exp exp" appears).
            exp: Exp<'a>,
            clean_exp: Exp<'a>,
            exp_return: Exp<'a>,
            exp_clean_return: Exp<'a>,
            exp_assign: Exp<'a>,
            exp_clean_assign: Exp<'a>,
            exp_disjunctions: Exp<'a>,
            exp_clean_disjunctions: Exp<'a>,
            exp_conjunctions: Exp<'a>,
            exp_clean_conjunctions: Exp<'a>,
            exp_comparisons: Exp<'a>,
            exp_clean_comparisons: Exp<'a>,
            exp_sums: Exp<'a>,
            exp_clean_sums: Exp<'a>,
            exp_products: Exp<'a>,
            exp_clean_products: Exp<'a>,
            exp_unary: Exp<'a>,
            exp_clean_unary: Exp<'a>,
            exp_powers: Exp<'a>,
            exp_atom: Exp<'a>,

            lvalue: LValue<'a>,

            else_block: Else<'a>,

            call_args: Vec<Exp<'a>>,

            block_0: Block<'a>,
            // A block that does not start with "-" (see comment
            // on clean expressions above.
            clean_block_0: Block<'a>,
            // We can only define block_1, and not block
            // as in the specification, because
            // with this generator it is impossible
            // to parse tokens that expand to an empty
            // sequence.
            block_1: Block<'a>,
            block_2: Block<'a>, // A block that starts with a semicolon and
                            // that can be just a sequence of semicolons.
        ]
        
        tokens: {
            tokens.map(|x| -> Result<(Span, _), ReadError> {
                let (span, x) = x?;

                if let Some(x) = x {
                    let token = match x {
                        Token::Int(val) => $int(val),
                        Token::Str(val) => $string(val),
                        Token::Ident(val) => $ident(val),

                        Token::IntIdent(l, r) => $intident((l, r)),
                        Token::IntLPar(val) => $intlpar(val),
                        Token::IdentLPar(val) => $identlpar(val),
                        Token::RParIdent(val) => $rparident(val),

                        Token::Keyword(kw) => {
                            use Keyword::*;

                            match kw {
                                Else => $ELSE(()),
                                Elseif => $ELSEIF(()),
                                End => $END(()),
                                False => $FALSE(()),
                                For => $FOR(()),
                                Function => $FUNCTION(()),
                                If => $IF(()),
                                Mutable => $MUTABLE(()),
                                Return => $RETURN(()),
                                Struct => $STRUCT(()),
                                True => $TRUE(()),
                                While => $WHILE(()),
                            }
                        },
                        Token::Punct(p) => {
                            use Punct::*;

                            match p {
                                LPar => $LPAR(()),
                                RPar => $RPAR(()),
                                Comma => $COMMA(()),
                                Colon => $COLON(()),
                                DoubleColon => $DOUBLECOLON(()),
                                Semicolon => $SEMICOLON(()),

                                Equ => $EQU(()),
                                DoubleEqu => $DOUBLEEQU(()),
                                Neq => $NEQ(()),
                                Lt => $LT(()),
                                Leq => $LEQ(()),
                                Gt => $GT(()),
                                Geq => $GEQ(()),

                                And => $AND(()),
                                Or => $OR(()),

                                Plus => $PLUS(()),
                                Minus => $MINUS(()),
                                Times => $TIMES(()),
                                Div => $DIV(()),

                                Not => $NOT(()),

                                Pow => $POW(()),
                                
                                Dot => $DOT(()),
                            }
                        }
                    };

                    Ok((span, TokenOrEof::Token(token)))
                } else {
                    Ok((span, TokenOrEof::Eof))
                }
            })
        }

        rules: {
            // We can't accept an empty file from this grammar.
            // Empty files will be handled manually.
            // TODO: handle empty files
            (file -> d:decl) => {Ok(vec!($d))},
            (file -> f:file d:decl) => {
                let mut v = $f;
                v.push($d);
                Ok(v)
            },

            (decl -> s:structure SEMICOLON) => {Ok(Decl::new($span, DeclVal::Structure($s)))},
            (decl -> f:function SEMICOLON) => {Ok(Decl::new($span, DeclVal::Function($f)))},
            (decl -> e:exp SEMICOLON) => {Ok(Decl::new($span, DeclVal::Exp($e)))},

            (located_ident -> id:ident) => {Ok(LocatedIdent::new($span, $id))},

            (fields -> p:param) => {Ok(vec!($p))},
            (fields -> SEMICOLON) => {Ok(vec!())},
            (fields -> SEMICOLON p:param) => {Ok(vec!($p))},
            (fields -> f:fields SEMICOLON) => {
                Ok($f)
            },
            (fields -> f:fields SEMICOLON p:param) => {
                let mut v = $f;
                v.push($p);
                Ok(v)
            },

            (struct_head -> STRUCT) => {Ok(false)},
            (struct_head -> MUTABLE STRUCT) => {Ok(true)},
            (structure -> mutable:struct_head name:located_ident END) => {
                Ok(Structure::new($span, $mutable, $name, vec!()))
            },
            (structure -> mutable:struct_head name:located_ident f:fields END) => {
                Ok(Structure::new($span, $mutable, $name, $f))
            },

            (param -> name:located_ident) => {Ok(Param::new($span, $name, None))},
            (param -> name:located_ident DOUBLECOLON ty:located_ident) => {
                Ok(Param::new($span, $name, Some($ty)))
            },

            (params -> p:param) => {Ok(vec!($p))},
            (params -> p:param COMMA) => {Ok(vec!($p))},
            (params -> p:param COMMA l:params) => {
                let mut v = $l;
                v.insert(0, $p);
                Ok(v)
            },

            (function_head -> FUNCTION name:identlpar RPAR) => {Ok(($name, vec!()))},
            (function_head -> FUNCTION name:identlpar p:params RPAR) => {Ok(($name, $p))},
            (function_signature -> h:function_head) => {Ok(($h.0, $h.1, None))},
            (function_signature -> h:function_head DOUBLECOLON ty:located_ident) => {
                Ok(($h.0, $h.1, Some($ty)))
            },
            (function -> s:function_signature END) => {
                Ok(Function::new($span, $s.0, $s.1, $s.2, Block::new($span, vec!())))
            },
            (function -> s:function_signature b:block_2 END) => {
                Ok(Function::new($span, $s.0, $s.1, $s.2, $b))
            },

            (range -> low:exp COLON high:exp) => {Ok(Range::new($span, $low, $high))},

            (comparison_op -> DOUBLEEQU) => {Ok(BinOp::Equ)},
            (comparison_op -> NEQ) => {Ok(BinOp::Neq)},
            (comparison_op -> LT) => {Ok(BinOp::Lt)},
            (comparison_op -> LEQ) => {Ok(BinOp::Leq)},
            (comparison_op -> GT) => {Ok(BinOp::Gt)},
            (comparison_op -> GEQ) => {Ok(BinOp::Geq)},

            (sum_op -> PLUS) => {Ok(BinOp::Plus)},
            (sum_op -> MINUS) => {Ok(BinOp::Minus)},

            (product_op -> TIMES) => {Ok(BinOp::Times)},
            (product_op -> DIV) => {Ok(BinOp::Div)},

            (exp -> e:exp_return) => {Ok($e)},
            (clean_exp -> e:exp_clean_return) => {Ok($e)},

            (exp_return -> RETURN e:exp_assign) => {
                Ok(Exp::new($span, ExpVal::Return($e)))
            },
            (exp_return -> e:exp_assign) => {Ok($e)},
            (exp_clean_return -> RETURN e:exp_assign) => {
                Ok(Exp::new($span, ExpVal::Return($e)))
            },
            (exp_clean_return -> e:exp_clean_assign) => {Ok($e)},

            (exp_assign -> l:lvalue EQU r:exp_disjunctions) => {
                Ok(Exp::new($span, ExpVal::Assign($l, $r)))
            },
            (exp_assign -> e:exp_disjunctions) => {Ok($e)},
            (exp_clean_assign -> l:lvalue EQU r:exp_disjunctions) => {
                Ok(Exp::new($span, ExpVal::Assign($l, $r)))
            },
            (exp_clean_assign -> e:exp_clean_disjunctions) => {Ok($e)},

            (exp_disjunctions -> l:exp_disjunctions OR r:exp_conjunctions) => {
                Ok(Exp::new($span, ExpVal::BinOp(BinOp::Or, $l, $r)))
            },
            (exp_disjunctions -> e:exp_conjunctions) => {
                Ok($e)
            },
            (exp_clean_disjunctions -> l:exp_clean_disjunctions OR r:exp_conjunctions) => {
                Ok(Exp::new($span, ExpVal::BinOp(BinOp::Or, $l, $r)))
            },
            (exp_clean_disjunctions -> e:exp_clean_conjunctions) => {
                Ok($e)
            },

            (exp_conjunctions -> l:exp_conjunctions AND r:exp_comparisons) => {
                Ok(Exp::new($span, ExpVal::BinOp(BinOp::And, $l, $r)))
            },
            (exp_conjunctions -> e:exp_comparisons) => {Ok($e)},
            (exp_clean_conjunctions -> l:exp_clean_conjunctions AND r:exp_comparisons) => {
                Ok(Exp::new($span, ExpVal::BinOp(BinOp::And, $l, $r)))
            },
            (exp_clean_conjunctions -> e:exp_clean_comparisons) => {Ok($e)},

            (exp_comparisons -> l:exp_comparisons op:comparison_op r:exp_sums) => {
                Ok(Exp::new($span, ExpVal::BinOp($op, $l, $r)))
            },
            (exp_comparisons -> e:exp_sums) => {Ok($e)},
            (exp_clean_comparisons -> l:exp_clean_comparisons op:comparison_op r:exp_sums) => {
                Ok(Exp::new($span, ExpVal::BinOp($op, $l, $r)))
            },
            (exp_clean_comparisons -> e:exp_clean_sums) => {Ok($e)},

            (exp_sums -> l:exp_sums op:sum_op r:exp_products) => {
                Ok(Exp::new($span, ExpVal::BinOp($op, $l, $r)))
            },
            (exp_sums -> e:exp_products) => {Ok($e)},
            (exp_clean_sums -> l:exp_clean_sums op:sum_op r:exp_products) => {
                Ok(Exp::new($span, ExpVal::BinOp($op, $l, $r)))
            },
            (exp_clean_sums -> e:exp_clean_products) => {Ok($e)},

            (exp_products -> l:exp_products op:product_op r:exp_unary) => {
                Ok(Exp::new($span, ExpVal::BinOp($op, $l, $r)))
            },
            (exp_products -> e:exp_unary) => {Ok($e)},
            (exp_clean_products -> l:exp_clean_products op:product_op r:exp_unary) => {
                Ok(Exp::new($span, ExpVal::BinOp($op, $l, $r)))
            },
            (exp_clean_products -> e:exp_clean_unary) => {Ok($e)},

            (exp_unary -> MINUS e:exp_unary) => {
                Ok(Exp::new($span, ExpVal::UnaryOp(UnaryOp::Neg, $e)))
            },
            (exp_unary -> NOT e:exp_unary) => {
                Ok(Exp::new($span, ExpVal::UnaryOp(UnaryOp::Not, $e)))
            },
            (exp_unary -> e:exp_powers) => {
                Ok($e)
            },
            (exp_clean_unary -> NOT e:exp_unary) => {
                Ok(Exp::new($span, ExpVal::UnaryOp(UnaryOp::Not, $e)))
            },
            (exp_clean_unary -> e:exp_powers) => {
                Ok($e)
            },

            (exp_powers -> l:exp_atom POW r:exp_powers) => {
                Ok(Exp::new($span, ExpVal::BinOp(BinOp::Pow, $l, $r)))
            },
            (exp_powers -> e:exp_atom) => {
                Ok($e)
            },

            (exp_atom -> v:int) => {Ok(Exp::new($span, ExpVal::Int($v)))},
            (exp_atom -> v:string) => {Ok(Exp::new($span, ExpVal::Str($v)))},
            (exp_atom -> TRUE) => {Ok(Exp::new($span, ExpVal::Bool(true)))},
            (exp_atom -> FALSE) => {Ok(Exp::new($span, ExpVal::Bool(false)))},
            (exp_atom -> v:lvalue) => {Ok(Exp::new($span, ExpVal::LValue($v)))},
            
            (exp_atom -> ii:intident) => {
                Ok(Exp::new($span, ExpVal::Mul($ii.0, $ii.1)))
            },
            (exp_atom -> l:intlpar b:block_1 RPAR) => {
                Ok(Exp::new($span, ExpVal::LMul($l, $b)))
            },
            (exp_atom -> LPAR e:exp r:rparident) => {
                Ok(Exp::new($span, ExpVal::RMul($e, $r)))
            },
            (exp_atom -> f:identlpar RPAR) => {
                Ok(Exp::new($span, ExpVal::Call($f, vec!())))
            },
            (exp_atom -> f:identlpar a:call_args RPAR) => {
                Ok(Exp::new($span, ExpVal::Call($f, $a)))
            },

            (exp_atom -> LPAR b:block_1 RPAR) => {
                Ok(Exp::new($span, ExpVal::Block($b)))
            },

            (call_args -> e:exp) => {
                Ok(vec!($e))
            },
            (call_args -> e:exp COMMA) => {
                Ok(vec!($e))
            },
            (call_args -> e:exp COMMA a:call_args) => {
                let mut v = $a;
                v.insert(0, $e);
                Ok(v)
            },

            (exp -> IF cond:exp e:else_block) => {
                Ok(Exp::new($span, ExpVal::If($cond, Block::new($span, vec!()), $e)))
            },
            (exp -> IF cond:exp b:clean_block_0 e:else_block) => {
                Ok(Exp::new($span, ExpVal::If($cond, $b, $e)))
            },

            (else_block -> END) => {Ok(Else::new($span, ElseVal::End))},
            (else_block -> ELSE END) => {
                Ok(Else::new($span, ElseVal::Else(Block::new($span, vec!()))))
            },
            (else_block -> ELSE b:block_0 END) => {
                Ok(Else::new($span, ElseVal::Else($b)))
            },
            (else_block -> ELSEIF cond:exp e:else_block) => {
                Ok(Else::new($span, ElseVal::ElseIf($cond, Block::new($span, vec!()), $e)))
            },
            (else_block -> ELSEIF cond:exp b:clean_block_0 e:else_block) => {
                Ok(Else::new($span, ElseVal::ElseIf($cond, $b, $e)))
            },

            (exp -> FOR id:located_ident EQU range:range END) => {
                Ok(Exp::new($span, ExpVal::For($id, $range, Block::new($span, vec!()))))
            },
            (exp -> FOR id:located_ident EQU range:range b:clean_block_0 END) => {
                Ok(Exp::new($span, ExpVal::For($id, $range, $b)))
            },

            (exp -> WHILE cond:exp END) => {
                Ok(Exp::new($span, ExpVal::While($cond, Block::new($span, vec!()))))
            },
            (exp -> WHILE cond:exp b:clean_block_0 END) => {
                Ok(Exp::new($span, ExpVal::While($cond, $b)))
            },

            (lvalue -> l:lvalue DOT r:ident) => {
                let mut v = $l.val;
                v.push($r);
                Ok(LValue::new($span, v))
            },
            (lvalue -> id:ident) => {
                Ok(LValue::new($span, vec!($id)))
            },
            
            (block_0 -> e:exp) => {Ok(Block::new($span, vec!($e)))},
            (block_0 -> SEMICOLON) => {Ok(Block::new($span, vec!()))},
            (block_0 -> SEMICOLON e:exp) => {Ok(Block::new($span, vec!($e)))},
            (block_0 -> b:block_0 SEMICOLON) => {Ok($b)},
            (block_0 -> b:block_0 SEMICOLON e:exp) => {
                let mut v = $b.val;
                v.push($e);
                Ok(Block::new($span, v))
            },
            (clean_block_0 -> e:clean_exp) => {Ok(Block::new($span, vec!($e)))},
            (clean_block_0 -> SEMICOLON) => {Ok(Block::new($span, vec!()))},
            (clean_block_0 -> SEMICOLON e:exp) => {Ok(Block::new($span, vec!($e)))},
            (clean_block_0 -> b:clean_block_0 SEMICOLON) => {Ok($b)},
            (clean_block_0 -> b:clean_block_0 SEMICOLON e:exp) => {
                let mut v = $b.val;
                v.push($e);
                Ok(Block::new($span, v))
            },
            (block_1 -> e:exp) => {Ok(Block::new($span, vec!($e)))},
            (block_1 -> e:exp b:block_2) => {
                let mut v = $b.val;
                v.insert(0, $e);
                Ok(Block::new($span, v))
            },
            (block_2 -> SEMICOLON) => {Ok(Block::new($span, vec!()))},
            (block_2 -> SEMICOLON b:block_2) => {
                Ok(Block::new($span, $b.val))
            },
            (block_2 -> SEMICOLON e:exp) => {
                Ok(Block::new($span, vec!($e)))
            },
            (block_2 -> SEMICOLON e:exp b:block_2) => {
                let mut v = $b.val;
                v.insert(0, $e);
                Ok(Block::new($span, v))
            },
        }

        on_empty: {Ok(Vec::new())}
        start: file
    };

    Ok(ast?)
}

fn run(file_name: &str) -> Result<(), String> {
    let path = Path::new(file_name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {} : {}", display, why),
        Ok(file) => file,
    };
    
    let mut s = String::new();
    file.read_to_string(&mut s).map_err(|e| e.to_string())?;
   
    let ast = parse(file_name, &s).map_err(|e| e.to_string())?;

    println!("{:?}", ast);

    Ok(())
}

fn test(dir_name: &str) -> Result<(), String> {
    /*
     * Returns true if all the tests pass.
     */
    fn test<'a, I: Iterator<Item = std::path::PathBuf>>(tests: I, expect_success: bool)
        -> Result<bool, String>
    {
        let mut failed = false;

        for path in tests {
            let display = path.display();
            let mut file = match File::open(&path) {
                Err(why) => panic!("Couldn't open {} : {}", display, why),
                Ok(file) => file,
            };

            let mut s = String::new();
            file.read_to_string(&mut s).map_err(|e| e.to_string())?;

            let file_name = path.file_name().and_then(|n| n.to_str()).map(|n| n.to_string()).unwrap_or("test".to_string());
            match parse(&file_name, &s) {
                Ok(_) => {
                    if expect_success {
                        println!("    \u{2713} {}", file_name);
                    } else {
                        failed = true;
                        println!("    \u{2717} {} : expected a failure, got a success", file_name);
                    }
                },
                Err(_) => {
                    if expect_success {
                        failed = true;
                        println!("    \u{2717} {} : expected a success, got a failure", file_name);
                    } else {
                        // "Task failed successfully."
                        println!("    \u{2713} {}", file_name);
                    }
                }
            }
        }

        Ok(failed)
    }
 
    fn get_paths(path: &Path) -> Result<Vec<std::path::PathBuf>, String> {
        let mut members = Vec::new();

        fs::read_dir(&path)
            .map_err(|e| e.to_string())?
            .map(|e| e.map_err(|e| e.to_string()).map(|e| e.path()))
            .try_for_each(|e| -> Result<(), String> {members.push(e?); Ok(())})?;

        members.sort();
        Ok(members)
    }

    let good_name = format!("{}/good", dir_name);
    let good_path = Path::new(&good_name);
    let bad_name = format!("{}/bad", dir_name);
    let bad_path = Path::new(&bad_name);

    let good_tests = get_paths(good_path)?;

    let bad_tests = get_paths(bad_path)?;
    
    let mut failed = false;
    println!("Testing \"good\" inputs :");
    failed = failed || test(good_tests.into_iter(), true)?;
    println!("Testing \"bad\" inputs :");
    failed = failed || test(bad_tests.into_iter(), false)?;

    if failed {
        println!("*** FAILED ***");
    } else {
        println!("*** SUCCESS ***");
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let matches = App::new("petit-julia")
        .version("1.0")
        .author("Julien Marquet")
        .subcommand(SubCommand::with_name("run")
            .about("Runs the given program")
            .arg(Arg::with_name("input")
                .help("The program to run")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("test")
            .about("Runs all the tests")
            .arg(Arg::with_name("input")
                .help("The directory that contains the tests")
                .required(true)
                .index(1)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let file_name = matches.value_of("input").unwrap();
        run(file_name)?;
    } else if let Some(matches) = matches.subcommand_matches("test") {
        let dir_name = matches.value_of("input").unwrap();
        test(dir_name)?;
    }

    Ok(())
}

