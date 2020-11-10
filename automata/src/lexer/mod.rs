
mod types;
mod sets;
mod building;
mod dfa;
mod macros;

pub use dfa::*;
pub use types::{Character, Regexp};
pub use building::build_automaton;
pub use macros::*;

