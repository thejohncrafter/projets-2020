
pub mod lexer;
pub mod parser;
pub mod line_counter;
pub mod read_error;

pub enum TokenOrEof<T> {
    Token(T),
    Eof
}

