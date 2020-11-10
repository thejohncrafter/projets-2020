
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized, Token,
    Result, Error, Ident, LitChar,
};
use syn::parse::{Parse, ParseStream};

pub enum Character {
    Char(char),
    Alpha,
    Num,
    Behaved,
    Any,
}

pub enum Regexp {
    Epsilon,
    Character(Character),
    Union(Box<Regexp>, Box<Regexp>),
    Concat(Box<Regexp>, Box<Regexp>),
    Star(Box<Regexp>),
}

impl Parse for Regexp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![_]) {
            input.parse::<Token![_]>()?;

            Ok(Regexp::Epsilon)
        } else if lookahead.peek(Ident) {
            let ident = input.parse::<Ident>()?;

            let c = match ident.to_string().as_str() {
                "alpha" => Character::Alpha,
                "num" => Character::Num,
                "behaved" => Character::Behaved,
                "any" => Character::Any,
                _ => Err(Error::new(ident.span(), "Unknown character class."))?
            };

            Ok(Regexp::Character(c))
        } else if lookahead.peek(LitChar) {
            let c = input.parse::<LitChar>()?.value();
            
            Ok(Regexp::Character(Character::Char(c)))
        } else {
            let contents;
            parenthesized!(contents in input);

            let mut sum = None;
            let mut product = None;
            let mut star;

            fn pop_star(star: &mut Option<Regexp>, product: &mut Option<Regexp>) {
                if let Some(f) = star.take() {
                    if let Some(g) = product.take() {
                        *product = Some(Regexp::Concat(Box::new(g), Box::new(f)));
                    } else {
                        *product = Some(f);
                    }
                    // star is now None
                }
            }

            fn pop_product(product: &mut Option<Regexp>, sum: &mut Option<Regexp>) {
                if let Some(f) = product.take() {
                    if let Some(g) = sum.take() {
                        *sum = Some(Regexp::Union(Box::new(g), Box::new(f)));
                    } else {
                        *sum = Some(f);
                    }
                    // product is now None
                }
            };

            loop {
                star = Some(contents.parse::<Regexp>()?);
                let lookahead = contents.lookahead1();

                let lookahead = if lookahead.peek(Token![*]) {
                    contents.parse::<Token![*]>()?;

                    if let Some(e) = star.take() {
                        star = Some(Regexp::Star(Box::new(e)))
                    }

                    contents.lookahead1()
                } else {
                    lookahead
                };

                if contents.is_empty() {
                    pop_star(&mut star, &mut product);
                    pop_product(&mut product, &mut sum);

                    break
                } else if lookahead.peek(Token![|]) {
                    contents.parse::<Token![|]>()?;

                    pop_star(&mut star, &mut product);
                    pop_product(&mut product, &mut sum);
                } else if lookahead.peek(Token![&]) {
                    contents.parse::<Token![&]>()?;

                    pop_star(&mut star, &mut product);
                } else {
                    Err(lookahead.error())?
                }
            }

            if let Some(e) = sum {
                Ok(e)
            } else {
                Err(Error::new(contents.span(), "Expressions can't be empty."))
            }
        }
    }
}

fn serialize_character(c: &Character) -> TokenStream {
    match c {
        Character::Char(c) => quote! {::automata::lexer::Character::Char(#c)},
        Character::Alpha => quote! {::automata::lexer::Character::Alpha},
        Character::Num => quote! {::automata::lexer::Character::Num},
        Character::Behaved => quote! {::automata::lexer::Character::Behaved},
        Character::Any => quote! {::automata::lexer::Character::Any},
    }
}

impl Regexp {
    pub fn serialize(&self) -> TokenStream {
        match self {
            Regexp::Epsilon => quote! {::automata::lexer::Regexp::Epsilon},
            Regexp::Character(c) => {
                let c = serialize_character(c);
                quote! {::automata::lexer::Regexp::Character(#c)}
            },
            Regexp::Union(l, r) => {
                let (l, r) = (l.serialize(), r.serialize());
                quote! {::automata::lexer::Regexp::Union(::std::boxed::Box::new(#l), ::std::boxed::Box::new(#r))}
            },
            Regexp::Concat(l, r) => {
                let (l, r) = (l.serialize(), r.serialize());
                quote! {::automata::lexer::Regexp::Concat(::std::boxed::Box::new(#l), ::std::boxed::Box::new(#r))}
            },
            Regexp::Star(e) => {
                let e = e.serialize();
                quote! {::automata::lexer::Regexp::Star(::std::boxed::Box::new(#e))}
            },
        }
    }
}
