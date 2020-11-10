
mod hooked_contents;
mod lexer;
mod parser;

#[proc_macro]
pub fn lex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    lexer::lex(input)
}

#[proc_macro]
pub fn reg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    lexer::reg(input)
}

#[proc_macro]
pub fn parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parser::parse(input)
}

