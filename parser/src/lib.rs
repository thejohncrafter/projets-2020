pub mod ast;
pub mod typing;

mod parse;

pub use parse::parse;
pub use typing::main::static_type;
use typing::data::TypedDecls;
use ast::Decl;

pub type ASTDeclarations<'a> = Vec<Decl<'a>>;
pub type TypedASTDeclarations<'a> = TypedDecls<'a>;

pub fn parse_and_type_file<'a>(file_name: &'a str, contents: &'a str) -> Result<TypedASTDeclarations<'a>, String> {
    static_type(parse(file_name, &contents).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
