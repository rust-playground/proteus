use crate::action::Action;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::ops::Deref;

/// This represents the trim operation type
#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    Trim,
    TrimStart,
    TrimEnd,
}

/// This type represents an [Action](../action/trait.Action.html) which trims the whitespace from
/// the left and right of a string.
#[derive(Debug, Serialize, Deserialize)]
pub struct Trim {
    r#type: Type,
    action: Box<dyn Action>,
}

impl Trim {
    pub fn new(r#type: Type, action: Box<dyn Action>) -> Self {
        Self { r#type, action }
    }
}

#[typetag::serde]
impl Action for Trim {
    fn apply<'a>(
        &self,
        source: &'a Value,
        destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, Error> {
        match self.action.apply(source, destination)? {
            Some(v) => match v.deref() {
                Value::String(s) => {
                    let s = match self.r#type {
                        Type::Trim => s.trim(),
                        Type::TrimStart => s.trim_start(),
                        Type::TrimEnd => s.trim_end(),
                    }
                    .to_owned();
                    Ok(Some(Cow::Owned(Value::String(s))))
                }
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }
}
