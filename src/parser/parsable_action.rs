//! ParsableAction trait and definitions.

use crate::action::Action;
use crate::parser::Error;
use crate::Parser;
use std::fmt::Debug;

/// A parsable action represents logic that can parse it's inner syntax and return an Action  
pub trait ParsableAction: Debug {
    fn parse(&self, parser: &Parser, value: &str) -> Result<Box<dyn Action>, Error>;
}
