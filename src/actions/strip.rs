use crate::action::Action;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::ops::Deref;

/// This represents the trim operation type
#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    StripPrefix,
    StripSuffix,
}

/// This type represents an [Action](../action/trait.Action.html) which trims the whitespace from
/// the left and right of a string.
#[derive(Debug, Serialize, Deserialize)]
pub struct Strip {
    r#type: Type,
    trim: String,
    action: Box<dyn Action>,
}

impl Strip {
    pub fn new(r#type: Type, trim: String, action: Box<dyn Action>) -> Self {
        Self {
            r#type,
            trim,
            action,
        }
    }
}

#[typetag::serde]
impl Action for Strip {
    fn apply<'a>(
        &'a self,
        source: &'a Value,
        destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, Error> {
        let res: Option<Cow<'a, Value>> = self.action.apply(source, destination)?;
        match &res {
            Some(v) => match v.deref() {
                Value::String(s) => {
                    let stripped = match self.r#type {
                        Type::StripPrefix => s.strip_prefix(&self.trim),
                        Type::StripSuffix => s.strip_suffix(&self.trim),
                    };
                    match stripped {
                        Some(s) => Ok(Some(Cow::Owned(Value::String(s.to_owned())))),
                        None => Ok(res),
                    }
                }
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }
}
