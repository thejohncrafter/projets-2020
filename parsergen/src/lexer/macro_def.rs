
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, braced, Token,
    Result, Ident,
};
use syn::parse::{Parse, ParseStream};

use crate::hooked_contents::*;
use super::regexp::*;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(chars);
    custom_keyword!(input);
}

struct MatcherArm {
    exp: Regexp,
    contents: HookedContents,
}

impl Parse for MatcherArm {
    fn parse(input: ParseStream) -> Result<Self> {
        let exp = input.parse()?;
        input.parse::<Token![=>]>()?;

        let contents;
        braced!(contents in input);
        let contents = contents.parse()?;

        Ok(MatcherArm {
            exp,
            contents,
        })
    }
}

fn parse_labeled_block<KW: Parse>(input: ParseStream) -> Result<HookedContents> { 
    input.parse::<KW>()?;
    input.parse::<Token![:]>()?;
    let block;
    braced!(block in input);
    block.parse()
}

struct MacroInput {
    chars_block: HookedContents,
    input_block: HookedContents,
    arms: Vec<MatcherArm>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let chars_block = parse_labeled_block::<kw::chars>(input)?;
        let input_block = parse_labeled_block::<kw::input>(input)?;

        let arms = input.parse_terminated::<_, Token![,]>(MatcherArm::parse)?;
        
        Ok(MacroInput {
            chars_block,
            input_block,
            arms: arms.into_iter().collect()
        })
    }
}

fn check(input: &MacroInput) -> Result<()> {
    input.chars_block.check(&[])?;
    input.input_block.check(&[])?;

    let hooked = &["span".to_string(), "text".to_string()];
    input.arms.iter().try_for_each(|arm| arm.contents.check(hooked))
}

pub fn reg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Regexp);
    
    input.serialize().into()
}

pub fn lex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);

    let expanded = match check(&input) {
        Ok(()) => {
            let span_ident = Ident::new("span", Span::mixed_site());
            let text_ident = Ident::new("text", Span::mixed_site());
            
            let exp_decls = input.arms.iter().map(|arm| {
                arm.exp.serialize()
            });
            let transformer_decls = input.arms.iter().map(|arm| {
                let contents = &arm.contents.contents;
                quote! {&|#span_ident, #text_ident| {#contents}}
            });

            let chars = input.chars_block.contents;
            let input_block = input.input_block.contents;

            quote! {
                build_automaton(
                    &[#(#exp_decls),*],
                    &[#(#transformer_decls),*],
                    {#chars},
                    {#input_block}
                )
            }
        },
        Err(e) => {
            e.to_compile_error()
        }
    };

    expanded.into()
}

