
mod ast;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

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

fn parse<'a>(file_name: &'a str, contents: &'a str) -> Result<(), ReadError<'a>> {
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

    // TODO: Comments
    let mut dfa: DFA<LineIter, IndexedString, PreToken, ReadError> = lex! {
        chars: {chars}
        input: {&input}

        ((' ' | '\t') & (' ' | '\t')*) => {Ok(PreToken::None)},
        ('\n') => {Ok(PreToken::Newline)},

        ((alpha | '_') & (alpha | '_' | num)*) => {
            Ok(ident_or_keyword($text).into_pre_token())
        },
        (('-' | _) & num & num*) => {Ok(PreToken::Token(Token::Int({
            parse_i64($text)?
        })))},
        ('"' & (behaved | '\\' & ('\\' | '"' | 'n' | 't'))* & '"') => {
            // TODO: Remove quotes and handle escape sequences.
            Ok(PreToken::Token(Token::Str($text.to_string())))
        },

        (('-' | _) & num* & alpha & (alpha | num)*) => {
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
        (('-' | _) & num* & '(') => {
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

        ('^') => {punct!(Pow)},

        ('.') => {punct!(Dot)},
    };

    struct Adapter<'a, I> {
        inner: &'a mut I,
        can_add_semi: bool,
    }

    /*
     * Eliminates whitespaces and inserts semicolons.
     */
    impl<'a, I> Adapter<'a, I> {
        fn new(inner: &'a mut I) -> Self {
            Adapter {inner, can_add_semi: false}
        }
    }

    impl<'a, I> Iterator for Adapter<'a, I>
        where I: Iterator<Item = Result<(Span<'a>, TokenOrEof<PreToken>), ReadError<'a>>>
    {
        type Item = Result<(Span<'a>, Option<Token>), ReadError<'a>>;

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
    
    /*tokens.map(|t| {
        t.unwrap().1
    }).take_while(|t| {
        match t {
            Some(_) => true,
            _ => false
        }
    }).map(|t| t.unwrap()).for_each(|token| {
        println!("{:?}", token)
    });*/

    let ast = parse! {
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

            POW: (),

            DOT: (),
        ]
        nterms: [
            exp_disjunctions: Exp,
            exp_conjunctions: Exp,
            exp_comparisons: Exp,
            exp_sums: Exp,
            exp_products: Exp,
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
            (exp_disjunctions -> l:exp_disjunctions OR r:exp_conjunctions) => {
                Ok(Exp::new(ExpVal::BinOp(BinOp::Or, $l, $r)))
            },
            (exp_disjunctions -> e:exp_conjunctions) => {
                Ok($e)
            },

            (exp_conjunctions -> l:exp_conjunctions AND r:exp_comparisons) => {
                Ok(Exp::new(ExpVal::BinOp(BinOp::And, $l, $r)))
            },
            (exp_conjunctions -> e:exp_comparisons) => {
                Ok($e)
            },
        }

        start: exp_disjunctions
    };

    Ok(())
}

fn main() -> Result<(), String> {
    let file_name = "test.pj";
    let path = Path::new(file_name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {} : {}", display, why),
        Ok(file) => file,
    };
    
    let mut s = String::new();
    file.read_to_string(&mut s).map_err(|e| e.to_string())?;
   
    parse(file_name, &s).map_err(|e| e.to_string())?;

    Ok(())
}

