//! Action trait and definitions.

use crate::errors::Error;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Debug;

/// An action represents an operation to be carried out on a serde_json::Value object.
#[typetag::serde(tag = "type")]
pub trait Action: Send + Sync + Debug {
    fn apply<'a>(
        &'a self,
        source: &'a Value,
        destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, Error>;
}
