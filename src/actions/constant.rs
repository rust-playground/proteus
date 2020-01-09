use crate::action::Action;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// This type represents an [Action](../action/trait.Action.html) which returns a constant Value
/// instead of it originating from the source JSON data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Constant {
    value: Value,
}

impl Constant {
    pub const fn new(value: Value) -> Self {
        Self { value }
    }
}

#[typetag::serde]
impl Action for Constant {
    fn apply(&self, _source: &Value, _destination: &mut Value) -> Result<Option<Value>, Error> {
        Ok(Some(self.value.clone()))
    }
}
