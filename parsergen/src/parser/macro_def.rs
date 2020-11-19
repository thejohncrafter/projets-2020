
use std::collections::HashMap;

use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{parse_macro_input, Result, Error, Ident, Type, Lifetime};

use super::input::*;

fn check_tokens(tokens: &[&Ident]) -> Result<()> {
    tokens.iter().enumerate().try_for_each(|(i, ident)| {
        if tokens[0..i].iter().any(|other| other.to_string() == ident.to_string()) {
            Err(Error::new(ident.span(), "Already defined."))
        } else {
            Ok(())
        }
    })
}

fn check(input: &MacroInput) -> Result<()> {
    let tokens = input.terms.iter().chain(input.nterms.iter()).map(|typed| &typed.0).collect::<Vec<&Ident>>();
    let terms = input.terms.iter().map(|typed| typed.0.to_string()).collect::<Vec<String>>();
    let token_names = tokens.iter().map(|ident| ident.to_string()).collect::<Vec<String>>();
    let nterm_names = input.nterms.iter().map(|typed| typed.0.to_string()).collect::<Vec<String>>();

    check_tokens(&tokens)?;
    input.tokenizer.body.check(&terms)?;
    input.on_empty.check(&vec!())?;

    input.rules.iter().try_for_each::<_, Result<()>>(|rule| {
        let vars = rule.expand.iter()
            .filter_map(|(x, _)| {
                if let Some(ident) = x {
                    Some(ident.to_string())
                } else {
                    None
                }
            })
            .chain(std::iter::once("span".to_string()))
            .collect::<Vec<String>>();

        rule.block.check(&vars)?;

        if !nterm_names.contains(&rule.token.to_string()) {
            Err(Error::new(rule.token.span(), "No such non-terminal."))?
        }

        rule.expand.iter().try_for_each(|(name, token)| {
            if let Some(name) = name {
                if name == "span" {
                    Err(Error::new(name.span(), "Name \"span\" is reserved."))?
                }
            }

            if !token_names.contains(&token.to_string()) {
                Err(Error::new(token.span(), "No such token."))
            } else {
                Ok(())
            }
        })?;

        Ok(())
    })?;

    if !nterm_names.contains(&input.start_token.to_string()) {
        Err(Error::new(input.start_token.span(), "No such non-terminal."))?
    }

    Ok(())
}

fn build_decls(src_lifetime: &Lifetime, tokens: &[(Ident, Type)]) -> TokenStream {
    tokens.iter().enumerate().map(|(i, typed)| {
        let ident = Ident::new(typed.0.to_string().as_str(), Span::mixed_site());

        let i = i + 1;
        let ty = &typed.1;
        let holder_ident = Ident::new("Holder", Span::mixed_site());
         
        quote! {
            fn #ident<#src_lifetime>(x: #ty) -> (usize, #holder_ident<#src_lifetime>) {(#i, #holder_ident::#ident(x))}
        }
    }).flatten().collect()
}

fn build_holder(src_lifetime: &Lifetime, tokens: &[&(Ident, Type)]) -> TokenStream {
    let holder_ident = Ident::new("Holder", Span::mixed_site());

    let variants = tokens.iter().map(|(ident, ty)| {
        let e: TokenStream = (quote! {#ident(#ty)}).into();
        e
    });

    let intos = tokens.iter().map(|(ident, ty)| {
        let into_name = Ident::new(&format!("into_{}", ident), Span::mixed_site());
        quote! {
            fn #into_name(self) -> #ty {
                match self {
                    #holder_ident::#ident(x) => x,
                    _ => panic!()
                }
            } 
        }
    }).flatten().collect::<TokenStream>();

    quote! {
        #[allow(non_camel_case_types)]
        enum #holder_ident<#src_lifetime> {
            __PHANTOM_HOLDER(::std::marker::PhantomData<&#src_lifetime ()>),
            #(#variants),*
        }

        impl<#src_lifetime> #holder_ident<#src_lifetime> {
            #intos
        }
    }
}

fn build_rules(types_info: &TypesInfo, token_types: &HashMap<String, &Type>, rules: &[Rule]) -> TokenStream {
    rules.iter().enumerate().map(|(i, rule)| {
        let src_lifetime = &types_info.src_lifetime;
        let span_ty = &types_info.span_ty;
        let holder_ident = Ident::new("Holder", Span::mixed_site());
        let fn_ident = Ident::new(&format!("rule_{}", i + 1), Span::mixed_site());
        let fn_return_type = token_types.get(&rule.token.to_string()).unwrap();
        let fn_return_variant = Ident::new(&format!("{}", rule.token), Span::mixed_site());
        let closure_ident = Ident::new(&format!("prod_{}", i + 1), Span::mixed_site());
        let body = &rule.block.contents;

        let span_ident = Ident::new("span", Span::mixed_site());
        
        let args_decl = rule.expand.iter().filter_map(|(x, ident)| {
            if let Some(arg_ident) = x {
                let token = ident.to_string();
                let ty = token_types.get(&token).unwrap();
                let arg_name = Ident::new(&arg_ident.to_string(), Span::mixed_site());
                let exp: TokenStream = quote! {
                    #arg_name: #ty
                };
                Some(exp)
            } else {
                None
            }
        });
        
        let args_getters = rule.expand.iter().enumerate().filter_map(|(i, (x, ident))| {
            if x.is_some() {
                let into_name = Ident::new(&format!("into_{}", ident.to_string()), Span::mixed_site());
                let exp: TokenStream = quote! {
                    a[#i].take().unwrap().#into_name()
                };
                Some(exp)
            } else {
                None
            }
        });

        quote! {
            fn #fn_ident<#src_lifetime>(#span_ident: #span_ty, #(#args_decl),*) -> Result<#fn_return_type, String> {
                #body
            }

            let #closure_ident = |span: #span_ty, mut a: Vec<Option<#holder_ident>>|
                #fn_ident(span, #(#args_getters),*).map(|x| #holder_ident::#fn_return_variant(x));
        }
    }).flatten().collect()
}

struct TableData {
    terms: Vec<String>,
    nterms: Vec<String>,
    prods: Vec<(String, Vec<String>)>,
}

fn build_table_data(input: &MacroInput) -> TableData {
    TableData {
        terms: input.terms.iter().map(|typed| typed.0.to_string()).collect(),
        nterms: input.nterms.iter().map(|typed| typed.0.to_string()).collect(),
        prods: input.rules.iter().map(|rule| {
            (
                rule.token.to_string(),
                rule.expand.iter().map(|(_, ident)| ident.to_string()).collect()
            )
        }).collect()
    }
}

fn serialize_table_data(data: &TableData) -> TokenStream {
    let terms = &data.terms;
    let nterms = &data.nterms;

    let prods = data.prods.iter().map(|(token, expand)| {
        quote! {
            (#token, vec!(#(#expand),*))
        }
    }).flatten();

    quote! {
        let terms = vec!(#(#terms),*);
        let nterms = vec!(#(#nterms),*);
        let prods = vec!(#(#prods),*);
    }
}

pub fn parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput); 

    let expanded = match check(&input) {
        Ok(()) => {
            let src_lifetime = &input.types_info.src_lifetime;
            let span_ty = &input.types_info.span_ty;

            let pairs = input.terms.iter().chain(input.nterms.iter()).collect::<Vec<_>>();
            let token_types: HashMap<_, _> = pairs.iter()
                .map(|(ident, ty)| (ident.to_string(), ty)).collect();
            let rules = build_rules(&input.types_info, &token_types, &input.rules);

            let body = &input.tokenizer.body.contents;
            let term_decls = build_decls(src_lifetime, &input.terms);
            
            let holder = build_holder(src_lifetime, &pairs);
            let holder_ident = Ident::new("Holder", Span::mixed_site());

            let rule_count = input.rules.len() + 1;

            let rule_names = (0..input.rules.len()).map(|i| {
                Ident::new(&format!("prod_{}", i + 1), Span::mixed_site())
            });

            let on_empty_fn = Ident::new("on_empty", Span::mixed_site());
            let start_token = input.start_token.to_string();
            let start_token_ident = Ident::new(&start_token, Span::mixed_site());
            let start_token_type = token_types.get(&start_token).unwrap();
            let on_empty = &input.on_empty.contents;

            let table_data = build_table_data(&input);
            let serialized_table = serialize_table_data(&table_data);
            let into_result_type = Ident::new(&format!("into_{}", start_token), Span::mixed_site());

            quote! {
                {      
                    #holder

                    let mut tokens_iter = {
                        #term_decls
                        #body
                    };
                    
                    #rules

                    let rules: [
                        &dyn Fn(#span_ty, Vec<Option<#holder_ident<#src_lifetime>>>)
                            -> Result<#holder_ident<#src_lifetime>, String>; #rule_count
                    ] = [
                        &|_span: #span_ty, mut a: Vec<Option<#holder_ident<#src_lifetime>>>| Ok(a[0].take().unwrap()),
                        #(&#rule_names),*
                    ];

                    #serialized_table

                    fn #on_empty_fn() -> Result<#start_token_type, String> {
                        #on_empty
                    }

                    let pda = build_pda::<#holder_ident<#src_lifetime>>(&terms, &nterms, &prods, #start_token);
                    let res = pda.parse(
                        &mut tokens_iter,
                        &mut || #on_empty_fn().map(|x| Holder::#start_token_ident(x)),
                        &rules,
                    );

                    res.map(|res| res.#into_result_type())
                }
            }
        },
        Err(e) => e.to_compile_error()
    };

    proc_macro::TokenStream::from(expanded)
}

