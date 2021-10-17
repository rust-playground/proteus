use crate::action::Action;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::ops::Deref;

/// This type represents an [Action](../action/trait.Action.html) which sums two or more Value's
/// and returns a Value::Number.
#[derive(Debug, Serialize, Deserialize)]
pub struct Sum {
    values: Vec<Box<dyn Action>>,
}

impl Sum {
    pub fn new(values: Vec<Box<dyn Action>>) -> Self {
        Self { values }
    }
}

#[typetag::serde]
impl Action for Sum {
    fn apply<'a>(
        &self,
        source: &'a Value,
        destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, Error> {
        let mut result: f64 = 0.0;
        let mut has_f64_value = false;

        for v in self.values.iter() {
            match v.apply(source, destination)? {
                Some(v) => {
                    match v.deref() {
                        Value::Number(num) => {
                            if num.is_f64() {
                                has_f64_value = true;
                            }
                            if let Some(n) = num.as_f64() {
                                result += n;
                            }
                        }
                        Value::Array(arr) => {
                            for v in arr {
                                match v {
                                    Value::Number(num) => {
                                        if num.is_f64() {
                                            has_f64_value = true;
                                        }
                                        if let Some(n) = num.as_f64() {
                                            result += n;
                                        }
                                    }
                                    _ => continue,
                                }
                            }
                        }
                        _ => continue,
                    };
                }
                None => continue,
            };
        }

        if has_f64_value {
            Ok(Some(Cow::Owned(result.into())))
        } else {
            Ok(Some(Cow::Owned((result as i64).into())))
        }
    }
}
