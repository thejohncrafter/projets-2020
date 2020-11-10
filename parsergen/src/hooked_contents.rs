
use proc_macro2::{TokenStream, TokenTree, Span, Group};
use syn::{Result, Error, Ident};
use syn::parse::{Parse, ParseStream};

pub struct HookedContents {
    pub contents: TokenStream,
    pub referenced: Vec<Ident>,
}

impl Parse for HookedContents {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut referenced = Vec::new();
        let tts = input.parse::<TokenStream>()?;

        fn visit(referenced: &mut Vec<Ident>, tts: TokenStream) -> Result<Vec<TokenTree>> {
            let mut input = tts.into_iter().peekable();
            let mut output = Vec::new();

            while let Some(peeked) = input.peek() {
                if let TokenTree::Punct(p) = peeked {
                    if p.as_char() == '$' {
                        let p = match input.next() {
                            Some(TokenTree::Punct(p)) => p,
                            _ => panic!()
                        };

                        if let Some(TokenTree::Ident(ident)) = input.next() {
                            let replacement = Ident::new(
                                &ident.to_string(),
                                Span::mixed_site().located_at(ident.span())
                            );

                            referenced.push(ident);
                            output.push(TokenTree::Ident(replacement));
                        } else {
                            Err(Error::new(p.span(), "Expected an idendifier following this token."))?
                        }
                    } else {
                        output.push(input.next().unwrap());
                    }
                } else if let TokenTree::Group(_) = peeked {
                    let group = match input.next() {
                        Some(TokenTree::Group(group)) => group,
                        _ => panic!()
                    };
                    
                    let mut body = syn::parse2::<HookedContents>(group.stream())?;
                    
                    let new_group = Group::new(group.delimiter(), body.contents);

                    referenced.append(&mut body.referenced);
                    output.push(TokenTree::Group(new_group))
                } else {
                    output.push(input.next().unwrap())
                }
            }

            Ok(output.into_iter().collect())
        }

        let res = visit(&mut referenced, tts)?;
        Ok(HookedContents {
            contents: res.into_iter().collect(),
            referenced,
        })
    }
}

impl HookedContents {
    pub fn check(&self, vars: &[String]) -> Result<()> {
        self.referenced.iter().try_for_each(|ident| {
            if !vars.contains(&ident.to_string()) {
                Err(Error::new(ident.span(), "No such hooked variable."))
            } else {
                Ok(())
            }
        })
    }
}

