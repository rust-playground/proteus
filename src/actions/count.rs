use crate::action::Action;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::actions::setter::Error as SetterError;
use crate::parser::ParsableAction;
use crate::Parser;
use crate::parser::Error as ParseError;


/// This type represents an [Action](../action/trait.Action.html) which counts the number of elements
/// in the given array.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct Count {
    value: Box<dyn Action>,
}

impl Count {
    pub fn new(value: Box<dyn Action>) -> Self {
        Self { value }
    }
}

#[typetag::serde]
impl Action for Count {
    fn apply(&self, source: &Value, destination: &mut Value) -> Result<Option<Value>, Error> {
        let result = match self.value.apply(source, destination)? {
            Some(v) => {
                match v {
                    Value::Array(l) => Ok(l.len()),
                    Value::String(_) => {
                        Err(SetterError::InvalidDestinationType {
                            err: format!("Attempting get count of string, expected array. {:?}", self.value),
                        })
                    }
                    Value::Null => {
                        Err(SetterError::InvalidDestinationType {
                            err: format!("Attempting get count of null, expected array. {:?}", self.value),
                        })
                    }
                    Value::Number(_) => {
                        Err(SetterError::InvalidDestinationType {
                            err: format!("Attempting get count of number, expected array. {:?}", self.value),
                        })
                    }
                    Value::Bool(_) => {
                        Err(SetterError::InvalidDestinationType {
                            err: format!("Attempting get count of bool, expected array. {:?}", self.value),
                        })
                    }
                    Value::Object(_) => {
                        Err(SetterError::InvalidDestinationType {
                            err: format!("Attempting get count of object, expected array. {:?}", self.value),
                        })
                    }
                }
            }
            None => Ok(0 as usize),
        };

        let res = result?;
        Ok(Some(Value::Number(res.into())))
    }
}


#[derive(Debug)]
pub struct ParsableCount;

impl ParsableAction for ParsableCount {
    fn parse(&self, parser: &Parser, value: &str) -> Result<Box<dyn Action>, ParseError> {
        let action = parser.get_action(value.trim()).unwrap();
        Ok(Box::new(Count::new(action)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::actions::Getter;
    use crate::actions::getter::namespace::Namespace as GetterNamespace;

    #[test]
    fn count_array_in_namespace() -> Result<(), Box<dyn std::error::Error>> {
        let input = json!({"key":["value1", "value2"]});
        let mut value = Value::Null;
        let counter = Count::new(Box::new(Getter::new(GetterNamespace::parse("key")?)));
        let res = counter.apply(&input, &mut value)?;
        assert_eq!(res.unwrap(), 2);
        Ok(())
    }

    #[test]
    fn count_raw_array() -> Result<(), Box<dyn std::error::Error>> {
        let input = json!(["value1", "value2"]);
        let mut value = Value::Null;
        let counter = Count::new(Box::new(Getter::new(GetterNamespace::parse("")?)));
        let res = counter.apply(&input, &mut value)?;
        assert_eq!(res.unwrap(), 2);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "expected array")]
    fn count_string_error() {
        let input = json!("value1");
        let mut value = Value::Null;
        let counter = Count::new(Box::new(Getter::new(GetterNamespace::parse("").unwrap())));
        let res = counter.apply(&input, &mut value);
        res.unwrap();
    }

}
