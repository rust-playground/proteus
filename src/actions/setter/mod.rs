mod errors;
pub mod namespace;

pub use errors::Error;

use crate::action::Action;
use crate::actions::setter::namespace::Namespace;
use crate::actions::setter::Error as SetterError;
use crate::errors::Error as CrateErr;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::borrow::Cow;

/// This type represents an [Action](../action/trait.Action.html) which sets data to the
/// destination JSON Value.
#[derive(Debug, Serialize, Deserialize)]
pub struct Setter {
    namespace: Vec<Namespace>,
    child: Box<dyn Action>,
}

impl Setter {
    pub fn new(namespace: Vec<Namespace>, child: Box<dyn Action>) -> Self {
        Self { namespace, child }
    }
}

#[typetag::serde]
impl Action for Setter {
    fn apply<'a>(
        &self,
        source: &'a Value,
        destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, CrateErr> {
        if let Some(field) = self.child.apply(source, destination)? {
            let field = field.into_owned();
            let mut current = destination;
            for ns in &self.namespace {
                match ns {
                    Namespace::Object { id } => {
                        match current {
                            Value::Object(o) => {
                                current = o.entry(id.clone()).or_insert(Value::Null);
                            }
                            Value::Null => {
                                let mut o = Map::new();
                                o.insert(id.clone(), Value::Null);
                                *current = Value::Object(o);
                                current = current.as_object_mut().unwrap().get_mut(id).unwrap();
                            }
                            _ => {
                                return Err(SetterError::InvalidDestinationType(format!(
                                    "Attempting to set an Object by id to an {:?}",
                                    current
                                ))
                                .into())
                            }
                        };
                    }
                    Namespace::Array { index } => {
                        let index = *index;
                        match current {
                            Value::Array(arr) => {
                                if index >= arr.len() {
                                    arr.resize_with(index + 1, Value::default);
                                    arr[index] = Value::Null;
                                }
                                current = &mut arr[index];
                            }
                            Value::Null => {
                                *current = Value::Array(vec![Value::Null; index + 1]);
                                current = &mut current.as_array_mut().unwrap()[index];
                            }
                            _ => {
                                return Err(SetterError::InvalidDestinationType(format!(
                                    "Attempting to set an Array by index to an {:?}",
                                    current
                                ))
                                .into())
                            }
                        };
                    }
                    Namespace::AppendArray => {
                        match current {
                            Value::Array(arr) => {
                                arr.push(Value::Null);
                                current = arr.last_mut().unwrap();
                            }
                            Value::Null => {
                                let arr = vec![Value::Null];
                                *current = Value::Array(arr);
                                current = current.as_array_mut().unwrap().last_mut().unwrap();
                            }
                            _ => {
                                return Err(SetterError::InvalidDestinationType(format!(
                                    "Attempting to append an {:?} to an Array",
                                    current
                                ))
                                .into())
                            }
                        };
                    }
                    Namespace::MergeObject => {
                        return match field {
                            Value::Object(mut o) => match current {
                                Value::Object(existing) => {
                                    existing.append(&mut o);
                                    Ok(None)
                                }
                                Value::Null => {
                                    let mut new = Map::new();
                                    new.append(&mut o);
                                    *current = Value::Object(new);
                                    Ok(None)
                                }
                                _ => Err(SetterError::InvalidDestinationType(format!(
                                    "Attempting to merge an Object with and {:?}",
                                    current
                                ))
                                .into()),
                            },
                            _ => Err(SetterError::InvalidDestinationType(format!(
                                "Attempting to merge {:?} with an Object",
                                field
                            ))
                            .into()),
                        };
                    }
                    Namespace::MergeArray => {
                        return match field {
                            Value::Array(arr) => match current {
                                Value::Array(existing) => {
                                    if arr.len() > existing.len() {
                                        *existing = arr;
                                        return Ok(None);
                                    }
                                    for (i, v) in arr.into_iter().enumerate() {
                                        existing[i] = v.clone();
                                    }
                                    Ok(None)
                                }
                                Value::Null => {
                                    *current = Value::Array(arr);
                                    Ok(None)
                                }
                                _ => Err(SetterError::InvalidDestinationType(format!(
                                    "Attempting to merge an Array with and {:?}",
                                    current
                                ))
                                .into()),
                            },
                            _ => Err(SetterError::InvalidDestinationType(format!(
                                "Attempting to merge {:?} with an Array",
                                field
                            ))
                            .into()),
                        };
                    }
                    Namespace::CombineArray => {
                        return match field {
                            Value::Array(mut arr) => match current {
                                Value::Array(existing) => {
                                    existing.append(&mut arr);
                                    Ok(None)
                                }
                                Value::Null => {
                                    *current = Value::Array(arr);
                                    Ok(None)
                                }
                                _ => Err(SetterError::InvalidDestinationType(format!(
                                    "Attempting to combine an Array with and {:?}",
                                    current
                                ))
                                .into()),
                            },
                            _ => Err(SetterError::InvalidDestinationType(format!(
                                "Attempting to merge {:?} with an Array",
                                field
                            ))
                            .into()),
                        };
                    }
                };
            }
            *current = field;
        }
        Ok(None)
    }
}
