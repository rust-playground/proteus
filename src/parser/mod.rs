//! Parser of transformation syntax into [Action(s)](action/trait.Action.html).

mod errors;
mod parsable_action;
mod parser;

pub use errors::Error;
pub use parsable_action::ParsableAction;
pub use parser::{Parsable, Parser, ParserBuilder, COMMA_SEP_RE, QUOTED_STR_RE};
