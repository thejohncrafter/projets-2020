
use syn::{
    braced, bracketed, parenthesized, Token,
    Result, Ident, Type,
};
use syn::parse::{Parse, ParseStream};

use crate::hooked_contents::HookedContents;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(terms);
    custom_keyword!(nterms);
    custom_keyword!(tokens);
    custom_keyword!(rules);
    custom_keyword!(on_empty);
    custom_keyword!(start);
}

pub struct MacroInput {
    pub terms: Vec<(Ident, Type)>,
    pub nterms: Vec<(Ident, Type)>,
    pub tokenizer: TokensInput,
    pub rules: Vec<Rule>,
    pub on_empty: HookedContents,
    pub start_token: Ident,
}

fn parse_typed(input: ParseStream) -> Result<(Ident, Type)> {
    let ident = input.parse()?;
    input.parse::<Token![:]>()?;
    let ty = input.parse()?;
    Ok((ident, ty))
}

fn parse_names<KW: Parse>(input: ParseStream) -> Result<Vec<(Ident, Type)>> {
    input.parse::<KW>()?;
    input.parse::<Token![:]>()?;

    let terms;
    bracketed!(terms in input);
    let terms = terms.parse_terminated::<_, Token![,]>(parse_typed)?;

    Ok(terms.into_iter().collect())
}

pub struct TokensInput {
    pub body: HookedContents,
}

fn parse_tokens(input: ParseStream) -> Result<TokensInput> {
    input.parse::<kw::tokens>()?;
    input.parse::<Token![:]>()?;

    let body;
    braced!(body in input);

    Ok(TokensInput {
        body: body.parse()?
    })
}

pub struct Rule {
    pub token: Ident,
    pub expand: Vec<(Option<Ident>, Ident)>,
    pub block: HookedContents,
}

fn parse_rule_lhs(input: ParseStream) -> Result<(Ident, Vec<(Option<Ident>, Ident)>)> {
    let token = input.parse::<Ident>()?;
    input.parse::<Token![->]>()?;
    
    let mut expand = Vec::new();

    while !input.is_empty() {
        let ident = input.parse::<Ident>()?;
        
        let e = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let name = input.parse::<Ident>()?;
            (Some(ident), name)
        } else {
            (None, ident)
        };

        expand.push(e)
    }

    Ok((token, expand))
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        let lhs;
        parenthesized!(lhs in input);
        let (token, expand) = parse_rule_lhs(&lhs)?;
        
        input.parse::<Token![=>]>()?;
        let block;
        braced!(block in input);
        let block = block.parse()?;

        Ok(Rule {token, expand, block})
    }
}

fn parse_rules(input: ParseStream) -> Result<Vec<Rule>> {
    input.parse::<kw::rules>()?;
    input.parse::<Token![:]>()?;

    let rules;
    braced!(rules in input);
    let rules = rules.parse_terminated::<_, Token![,]>(Rule::parse)?;

    Ok(rules.into_iter().collect())
}

fn parse_on_empty(input: ParseStream) -> Result<HookedContents> {
    input.parse::<kw::on_empty>()?;
    input.parse::<Token![:]>()?;

    let body;
    braced!(body in input);

    Ok(body.parse()?)
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let terms = parse_names::<kw::terms>(input)?;
        let nterms = parse_names::<kw::nterms>(input)?;
        let tokenizer = parse_tokens(input)?;
        let rules = parse_rules(input)?;
        let on_empty = parse_on_empty(input)?;

        input.parse::<kw::start>()?;
        input.parse::<Token![:]>()?;
        let start_token = input.parse()?;

        Ok(MacroInput {
            terms,
            nterms,
            tokenizer,
            rules,
            on_empty,
            start_token,
        })
    }
}

