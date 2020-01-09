//! Parser of transformation syntax into [Action(s)](action/trait.Action.html).

mod errors;
mod parser;

pub use errors::Error;
pub use parser::{Parsable, Parser};
