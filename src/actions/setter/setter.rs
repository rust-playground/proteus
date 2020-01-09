use crate::action::Action;
use crate::actions::setter::namespace::Namespace;
use crate::actions::setter::Error as SetterError;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
    fn apply(&self, source: &Value, destination: &mut Value) -> Result<Option<Value>, Error> {
        if let Some(field) = self.child.apply(source, destination)? {
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
                                return Err(SetterError::InvalidDestinationType {
                                    err: format!(
                                        "Attempting to set an Object by id to an {:?}",
                                        current
                                    ),
                                }
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
                                return Err(SetterError::InvalidDestinationType {
                                    err: format!(
                                        "Attempting to set an Array by index to an {:?}",
                                        current
                                    ),
                                }
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
                                return Err(SetterError::InvalidDestinationType {
                                    err: format!(
                                        "Attempting to append an {:?} to an Array",
                                        current
                                    ),
                                }
                                .into())
                            }
                        };
                    }
                    Namespace::MergeObject => {
                        // serde_json::Map does not have 'append' because it's not a fixed type.
                        // it could be:
                        // - https://docs.rs/indexmap/1.3.0/indexmap/map/struct.IndexMap.html
                        // - https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.append
                        //
                        // PR has been made to expose this functionality https://github.com/serde-rs/json/pull/600

                        // mem::take is ok as getter clones all source data for safety.
                        match field {
                            Value::Object(mut o) => {
                                match current {
                                    Value::Object(existing) => {
                                        for (k, v) in std::mem::take(&mut o).into_iter() {
                                            existing.insert(k, v);
                                        }
                                        return Ok(None);
                                    }
                                    Value::Null => {
                                        let mut new = Map::new();
                                        for (k, v) in std::mem::take(&mut o).into_iter() {
                                            new.insert(k, v);
                                        }
                                        *current = Value::Object(new);
                                        return Ok(None);
                                    }
                                    _ => {
                                        return Err(SetterError::InvalidDestinationType {
                                            err: format!(
                                                "Attempting to merge an Object with and {:?}",
                                                current
                                            ),
                                        }
                                        .into())
                                    }
                                };
                            }
                            _ => {
                                return Err(SetterError::InvalidDestinationType {
                                    err: format!("Attempting to merge {:?} with an Object", field),
                                }
                                .into())
                            }
                        };
                    }
                    Namespace::MergeArray => {
                        match field {
                            Value::Array(arr) => {
                                match current {
                                    Value::Array(existing) => {
                                        if arr.len() > existing.len() {
                                            *existing = arr;
                                            return Ok(None);
                                        }
                                        for (i, v) in arr.into_iter().enumerate() {
                                            existing[i] = v;
                                        }
                                        return Ok(None);
                                    }
                                    Value::Null => {
                                        *current = Value::Array(arr);
                                        return Ok(None);
                                    }
                                    _ => {
                                        return Err(SetterError::InvalidDestinationType {
                                            err: format!(
                                                "Attempting to merge an Array with and {:?}",
                                                current
                                            ),
                                        }
                                        .into())
                                    }
                                };
                            }
                            _ => {
                                return Err(SetterError::InvalidDestinationType {
                                    err: format!("Attempting to merge {:?} with an Array", field),
                                }
                                .into())
                            }
                        };
                    }
                    Namespace::CombineArray => {
                        match field {
                            Value::Array(mut arr) => {
                                match current {
                                    Value::Array(existing) => {
                                        existing.append(&mut arr);
                                        return Ok(None);
                                    }
                                    Value::Null => {
                                        *current = Value::Array(arr);
                                        return Ok(None);
                                    }
                                    _ => {
                                        return Err(SetterError::InvalidDestinationType {
                                            err: format!(
                                                "Attempting to combine an Array with and {:?}",
                                                current
                                            ),
                                        }
                                        .into())
                                    }
                                };
                            }
                            _ => {
                                return Err(SetterError::InvalidDestinationType {
                                    err: format!("Attempting to merge {:?} with an Array", field),
                                }
                                .into())
                            }
                        };
                    }
                };
            }
            *current = field;
        }
        Ok(None)
    }
}
