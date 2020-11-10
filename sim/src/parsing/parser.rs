
use automata::lexer::*;
use automata::parser::*;
use automata::line_counter::*;
use automata::read_error::*;

use parsergen::{lex, parse};

use super::types::*;

pub enum Symbol {
    Comma,
    Colon,
    Equals,
    Plus,
    Times,
    LPar,
    RPar,
}

pub enum Token {
    Ident(String),
    Num(String),
    Str(String),
    Sym(Symbol),
}

pub fn parse_netlist<'a>(file_name: &'a str, contents: &'a str) -> Result<Netlist, ReadError<'a>> {
    let chars = LineIter::new(contents);
    let input = IndexedString::new(file_name, contents);

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
            Ok(Some(Token::Num($text.to_string())))
        },
        ('"' & (behaved | '\\' & ('\\' | '"' | 'n' | 't'))* & '"') => {
            Ok(Some(Token::Str($text.to_string())))
        },

        ',' => {Ok(Some(Token::Sym(Symbol::Comma)))},
        ':' => {Ok(Some(Token::Sym(Symbol::Colon)))},
        '=' => {Ok(Some(Token::Sym(Symbol::Equals)))},
        '+' => {Ok(Some(Token::Sym(Symbol::Plus)))},
        '*' => {Ok(Some(Token::Sym(Symbol::Times)))},
        '(' => {Ok(Some(Token::Sym(Symbol::LPar)))},
        ')' => {Ok(Some(Token::Sym(Symbol::RPar)))},
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
        terms: [
            INPUT: (), OUTPUT: (), VAR: (), IN: (),
            NOT: (),
            OR: (), XOR: (), AND: (), NAND: (),
            MUX: (),
            REG: (),
            RAM: (), ROM: (),
            SELECT: (), SLICE: (), CONCAT: (),
            COMMA: (), COLON: (), EQUALS: (),
            ident: String,
            /*
             * We will decide later the type of number
             * this token represents.
             */
            uint: String,
        ]
        nterms: [
            netlist: Netlist,
            
            decl: (String, ValueType),
            name_list: Vec<String>,
            inputs: Vec<String>,
            outputs: Vec<String>,
            vars: Vec<(String, ValueType)>,
            decl_list: Vec<(String, ValueType)>,
            
            defs: Vec<(String, Def)>,
            def: (String, Def),
            
            op: Def,
            bin_op_type: BinOpType,
            
            arg: Arg,
        ]

        tokens: {
            tokens.map(move |x| -> Result<(Span, _), ReadError> {
                let (span, x) = x.unwrap();
                
                if let Some(x) = x {
                    let token = match x {
                        Token::Ident(name) => {
                            match name.as_str() {
                                "INPUT" => $INPUT(()),
                                "OUTPUT" => $OUTPUT(()),
                                "VAR" => $VAR(()),
                                "IN" => $IN(()),

                                "NOT" => $NOT(()),

                                "OR" => $OR(()),
                                "XOR" => $XOR(()),
                                "AND" => $AND(()),
                                "NAND" => $NAND(()),
                                
                                "MUX" => $MUX(()),

                                "REG" => $REG(()),

                                "RAM" => $RAM(()),
                                "ROM" => $ROM(()),
                                
                                "SELECT" => $SELECT(()),
                                "SLICE" => $SLICE(()),
                                "CONCAT" => $CONCAT(()),

                                _ => $ident(name)
                            }
                        },
                        Token::Num(repr) => $uint(repr),
                        Token::Sym(Symbol::Comma) => $COMMA(()),
                        Token::Sym(Symbol::Colon) => $COLON(()),
                        Token::Sym(Symbol::Equals) => $EQUALS(()),
                        _ => panic!(),
                    };
                    Ok((span, TokenOrEof::Token(token)))
                } else {
                    Ok((span, TokenOrEof::Eof))
                }
            })
        }

        rules: {
            (netlist ->
                inputs:inputs
                outputs:outputs
                vars:vars IN
                defs:defs
            ) => {
                Ok(Netlist {
                    $inputs,
                    $outputs,
                    $vars,
                    $defs,
                })
            },
            (netlist ->
                inputs:inputs
                outputs:outputs
                vars:vars IN
            ) => {
                Ok(Netlist {
                    $inputs,
                    $outputs,
                    $vars,
                    defs: vec!(),
                })
            },

            (inputs -> INPUT inputs:name_list) => {Ok($inputs)},
            (inputs -> INPUT) => {Ok(vec!())},
            (outputs -> OUTPUT outputs:name_list) => {Ok($outputs)},
            (outputs -> OUTPUT) => {Ok(vec!())},
            (vars -> VAR vars:decl_list) => {Ok($vars)},
            (vars -> VAR) => {Ok(vec!())},

            (name_list -> l:name_list COMMA r:ident) => {
                let mut names = $l;
                names.push($r);
                Ok(names)
            },
            (name_list -> id:ident) => {Ok(vec!($id))},
            (decl_list -> l:decl_list COMMA r:decl) => {
                let mut decls = $l;
                decls.push($r);
                Ok(decls)
            },
            (decl_list -> d:decl) => {Ok(vec!($d))},

            (decl -> id:ident) => {
                Ok(($id, ValueType::Bit))
            },
            (decl -> id:ident COLON len:uint) => {
                let len = $len.parse().unwrap();
                
                if len != 0 {
                    Ok(($id, ValueType::BitArray(len)))
                } else {
                    Err("Illegal length (0) for a bit array.".to_string())
                }
            },

            (defs -> l:defs r:def) => {
                let mut ops = $l;
                ops.push($r);
                Ok(ops)
            },
            (defs -> d:def) => {Ok(vec!($d))},

            (def -> l:ident EQUALS r:op) => {Ok(($l, $r))},

            (op -> a:arg) => {Ok(Def::Fwd($a))},

            (op -> NOT a:arg) => {Ok(Def::Not($a))},

            (op -> ty:bin_op_type l:arg r:arg) => {Ok(Def::Bin($ty, $l, $r))},
            (bin_op_type -> OR) => {Ok(BinOpType::Or)},
            (bin_op_type -> XOR) => {Ok(BinOpType::Xor)},
            (bin_op_type -> AND) => {Ok(BinOpType::And)},
            (bin_op_type -> NAND) => {Ok(BinOpType::Nand)},
            
            (op -> MUX sel:arg l:arg r:arg) => {Ok(Def::Mux($sel, $l, $r))},

            (op -> REG r:ident) => {Ok(Def::Reg($r))},
            
            (op -> RAM
                address_size:uint word_size:uint
                read_address:arg
                write_enable:arg write_address:arg data:arg) => {
                Ok(Def::Ram(RamData {
                    address_size: $address_size.parse().unwrap(),
                    word_size: $word_size.parse().unwrap(),
                    $read_address,
                    $write_enable, $write_address, $data,
                }))
            },
            (op -> ROM
                address_size:uint word_size:uint
                read_address:arg) => {
                Ok(Def::Rom(RomData {
                    address_size: $address_size.parse().unwrap(),
                    word_size: $word_size.parse().unwrap(),
                    $read_address,
                }))
            },
            
            (op -> SELECT index:uint bus:arg) => {
                Ok(Def::Select($index.parse().unwrap(), $bus))
            },
            (op -> SLICE start:uint end:uint bus:arg) => {
                match $end.parse() {
                    Ok(end) => Ok(Def::Slice($start.parse().unwrap(), end, $bus)),
                    Err(_) => Err("Number too big.".to_string())
                }
            },
            (op -> CONCAT l:arg r:arg) => {
                Ok(Def::Concat($l, $r))
            },

            (arg -> name:ident) => {Ok(Arg::Var($name))},
            (arg -> v:uint) => {
                if $v.chars().all(|c| c == '1' || c == '0') {
                    Ok(Arg::Const(match $v.len() {
                        0 => panic!(), // Can't happen given how the lexer is defined.
                        1 => Value::Bit($v.chars().next().unwrap() == '1'),
                        _ => Value::BitArray(
                            $v.chars().map(|c| c == '1').collect()
                        )
                    }))
                } else {
                    Err("Expected a list of '1's and '0's".to_string())
                }
            },
        }

        start: netlist
    };

    res
}

