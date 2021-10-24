use crate::action::Action;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::ops::Deref;

/// This type represents an [Action](../action/trait.Action.html) which returns the length of a
/// String, Array or Object..
#[derive(Debug, Serialize, Deserialize)]
pub struct Len {
    action: Box<dyn Action>,
}

impl Len {
    pub fn new(action: Box<dyn Action>) -> Self {
        Len { action }
    }
}

#[typetag::serde]
impl Action for Len {
    fn apply<'a>(
        &'a self,
        source: &'a Value,
        destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, Error> {
        match self.action.apply(source, destination)? {
            Some(v) => match v.deref() {
                Value::String(s) => Ok(Some(Cow::Owned(Value::Number(s.len().into())))),
                Value::Array(arr) => Ok(Some(Cow::Owned(Value::Number(arr.len().into())))),
                Value::Object(o) => Ok(Some(Cow::Owned(Value::Number(o.len().into())))),
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }
}
