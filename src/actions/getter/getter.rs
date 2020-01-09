use crate::action::Action;
use crate::actions::getter::namespace::Namespace;
use crate::errors::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// This type represents an [Action](../action/trait.Action.html) which extracts data from the
/// source JSON Value.
#[derive(Debug, Serialize, Deserialize)]
pub struct Getter {
    namespace: Vec<Namespace>,
}

impl Getter {
    pub fn new(namespace: Vec<Namespace>) -> Self {
        Self { namespace }
    }
}

#[typetag::serde]
impl Action for Getter {
    fn apply(&self, source: &Value, _destination: &mut Value) -> Result<Option<Value>, Error> {
        let mut current = source;
        for ns in &self.namespace {
            current = match expand(ns, current)? {
                Some(value) => value,
                None => return Ok(None),
            };
        }
        Ok(Some(current.clone()))
    }
}

#[inline]
fn expand<'a>(ns: &Namespace, current: &'a Value) -> Result<Option<&'a Value>, Error> {
    match current {
        Value::Object(o) => match ns {
            Namespace::Object { id } => Ok(o.get(id)),
            _ => Ok(None),
        },
        Value::Array(arr) => match ns {
            Namespace::Array { index } => Ok(arr.get(*index)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map};

    #[test]
    fn key_value() -> Result<(), Box<dyn std::error::Error>> {
        let ns = Namespace::parse("key")?;
        let input = json!({"key":"value"});
        let mut output = Value::Object(Map::new().into());
        let getter = Getter::new(ns);
        let res = getter.apply(&input, &mut output)?;
        assert_eq!(res, Some(Value::String("value".into())));
        Ok(())
    }

    #[test]
    fn array_value_in_object() -> Result<(), Box<dyn std::error::Error>> {
        let ns = Namespace::parse("my_array[0]")?;
        let input = json!({
            "existing_key":"my_val1",
            "my_array":["value"]
        });
        let mut output = Value::Object(Map::new().into());
        let getter = Getter::new(ns);
        let res = getter.apply(&input, &mut output)?;
        assert_eq!(res, Some(Value::String("value".into())));
        Ok(())
    }

    #[test]
    fn array_value_in_array() -> Result<(), Box<dyn std::error::Error>> {
        let ns = Namespace::parse("[0][0]")?;
        let input = json!([["value"]]);
        let mut output = Value::Object(Map::new().into());
        let getter = Getter::new(ns);
        let res = getter.apply(&input, &mut output)?;
        assert_eq!(res, Some(Value::String("value".into())));
        Ok(())
    }

    #[test]
    fn array_in_array() -> Result<(), Box<dyn std::error::Error>> {
        let ns = Namespace::parse("[0]")?;
        let input = json!([["value"]]);
        let mut output = Value::Object(Map::new().into());
        let getter = Getter::new(ns);
        let res = getter.apply(&input, &mut output)?;
        assert_eq!(res, Some(json!(["value"])));
        Ok(())
    }

    #[test]
    fn object_value_in_array() -> Result<(), Box<dyn std::error::Error>> {
        let ns = Namespace::parse("[0].key")?;
        let input = json!([{"key":"value"}]);
        let mut output = Value::Object(Map::new().into());
        let getter = Getter::new(ns);
        let res = getter.apply(&input, &mut output)?;
        assert_eq!(res, Some(json!("value")));
        Ok(())
    }

    #[test]
    fn array_value_in_object_in_array() -> Result<(), Box<dyn std::error::Error>> {
        let ns = Namespace::parse("[0].key[1]")?;
        let input = json!([{"key":[null,"value"]}]);
        let mut output = Value::Object(Map::new().into());
        let getter = Getter::new(ns);
        let res = getter.apply(&input, &mut output)?;
        assert_eq!(res, Some(json!("value")));
        Ok(())
    }
}
