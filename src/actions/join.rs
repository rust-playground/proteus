use crate::action::Action;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::ops::Deref;

/// This type represents an [Action](../action/trait.Action.html) which joins two or more Value's
/// separated by the provided `sep` and returns a Value::String(String).
///
/// This also works with non-string types but they will be converted into a string prior to joining.
#[derive(Debug, Serialize, Deserialize)]
pub struct Join {
    sep: String,
    values: Vec<Box<dyn Action>>,
}

impl Join {
    pub fn new(sep: String, values: Vec<Box<dyn Action>>) -> Self {
        Self { sep, values }
    }
}

#[typetag::serde]
impl Action for Join {
    fn apply<'a>(
        &self,
        source: &'a Value,
        destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, Error> {
        let l = self.values.len() - 1;
        let mut result = String::new();
        for (i, v) in self.values.iter().enumerate() {
            match v.apply(source, destination)? {
                Some(v) => {
                    match v.deref() {
                        Value::String(s) => {
                            if s.is_empty() {
                                continue;
                            }
                            result.push_str(s);
                        }
                        _ => {
                            let s = v.to_string();
                            if s.is_empty() {
                                continue;
                            }
                            result.push_str(&s);
                        }
                    };
                    if i != l {
                        result.push_str(&self.sep);
                    }
                }
                None => continue,
            };
        }

        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(Cow::Owned(Value::String(result))))
    }
}
