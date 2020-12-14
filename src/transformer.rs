//! builder and finalized transformer representations..

use crate::action::Action;
use crate::errors::Error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

/// This type provides the ability to create a [Transformer](struct.Transformer.html) for use.
#[derive(Debug)]
pub struct TransformBuilder {
    actions: Vec<Box<dyn Action>>,
}

impl Default for TransformBuilder {
    fn default() -> Self {
        TransformBuilder {
            actions: Vec::new(),
        }
    }
}

impl TransformBuilder {
    /// adds a single [Action](action/trait.Action.html) to be applied during the transformation.
    pub fn add_action(mut self, action: Box<dyn Action>) -> Self {
        self.actions.push(action);
        self
    }

    /// adds multiple [Action](action/trait.Action.html) to be applied during the transformation.
    pub fn add_actions(mut self, mut actions: Vec<Box<dyn Action>>) -> Self {
        self.actions.append(&mut actions);
        self
    }

    /// creates the final [Transformer](struct.Transformer.html) representation.
    pub fn build(self) -> Result<Transformer, Error> {
        // Error return value is reserved for future optimization during the build phase.
        Ok(Transformer {
            actions: self.actions,
        })
    }
}

/// This type represents a realized transformation which can be used on data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Transformer {
    actions: Vec<Box<dyn Action>>,
}

impl Transformer {
    /// directly applies the transform actions, in order, on the source and sets directly on the
    /// provided destination.
    ///
    /// The destination in question can be an existing Object and the data set on it at any level.
    #[inline]
    pub fn apply_to_destination(
        &self,
        source: &Value,
        destination: &mut Value,
    ) -> Result<(), Error> {
        for a in self.actions.iter() {
            a.apply(&source, destination)?;
        }
        Ok(())
    }

    /// applies the transform actions, in order, on the source and returns a final Value.
    pub fn apply(&self, source: &Value) -> Result<Value, Error> {
        let mut value = Value::Null;
        self.apply_to_destination(source, &mut value)?;
        Ok(value)
    }

    /// applies the transform actions, in order, on the source slice.
    ///
    /// The source string MUST be valid utf-8 JSON.
    pub fn apply_from_slice(&self, source: &[u8]) -> Result<Value, Error> {
        self.apply(&serde_json::from_slice(&source)?)
    }

    /// applies the transform actions, in order, on the source string.
    ///
    /// The source string MUST be valid JSON.
    pub fn apply_from_str<'a, S>(&self, source: S) -> Result<Value, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        self.apply(&serde_json::from_str(&source.into())?)
    }

    /// applies the transform actions, in order, on the source string and returns the type
    /// represented by D.
    ///
    /// The source string MUST be valid JSON.
    pub fn apply_from_str_to<'a, S, D>(&self, source: S) -> Result<D, Error>
    where
        S: Into<Cow<'a, str>>,
        D: DeserializeOwned,
    {
        let value = self.apply(&serde_json::from_str(&source.into())?)?;
        Ok(serde_json::from_value::<D>(value)?)
    }

    /// applies the transform actions, in order, on the serializable source and returns the type
    /// represented by D.
    pub fn apply_to<S, D>(&self, source: S) -> Result<D, Error>
    where
        S: Serialize,
        D: DeserializeOwned,
    {
        let value = self.apply(&serde_json::to_value(source)?)?;
        Ok(serde_json::from_value::<D>(value)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Parsable, Parser, TransformBuilder};
    use serde_json::{json, Value};

    #[test]
    fn constant() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(r#"const("Dean Karn")"#, "full_name")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let destination = trans.apply(&source)?;
        let expected = json!({"full_name":"Dean Karn"});
        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn array_of_array_to_array() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(r#"const("Dean Karn")"#, "[2][1]")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let destination = trans.apply(&source)?;
        assert!(destination.is_array());

        let expected = json!([null, null, [null, "Dean Karn"]]);

        assert_eq!(expected, destination);

        let action = Parser::parse(r#"const("Dean Karn")"#, "[2][1].name")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let destination = trans.apply(&source)?;
        assert!(destination.is_array());

        let expected = json!([null, null, [null, {"name":"Dean Karn"}]]);
        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn push_array() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(r#"const("Dean Karn")"#, "[2][]")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let destination = trans.apply(&source)?;
        assert!(destination.is_array());

        let expected = json!([null, null, ["Dean Karn"]]);

        assert_eq!(expected, destination);

        let action = Parser::parse(r#"const("Dean Karn")"#, "[2][]")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let mut destination = json!([null, null, [null]]);

        let res = trans.apply_to_destination(&source, &mut destination);
        assert!(!res.is_err());
        assert!(destination.is_array());

        let expected = json!([null, null, [null, "Dean Karn"]]);

        assert_eq!(expected, destination);

        let action = Parser::parse(r#"const("Dean Karn")"#, "[2]")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let destination = trans.apply(&source)?;
        assert!(destination.is_array());

        let expected = json!([null, null, "Dean Karn"]);

        assert_eq!(expected, destination);

        // testing replace
        let action = Parser::parse(r#"const("Dean Karn")"#, "[2]")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let mut destination = json!([null, null, {"id":"id"}]);
        let res = trans.apply_to_destination(&source, &mut destination);
        assert!(!res.is_err());
        assert!(destination.is_array());

        let expected = json!([null, null, "Dean Karn"]);

        assert_eq!(expected, destination);

        let action = Parser::parse(r#"const("Dean Karn")"#, "[1].key.key2")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let destination = trans.apply(&source)?;
        assert!(destination.is_array());

        let expected = json!([null, {"key": {"key2":"Dean Karn"}}]);

        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn append_array_top_level() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(r#"const([null,"Dean Karn"])"#, "[]")?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = "".into();
        let mut destination = Value::Array(vec!["test".into()]);
        let res = trans.apply_to_destination(&source, &mut destination);
        assert!(!res.is_err());
        assert!(destination.is_array());

        let expected = json!(["test", [null, "Dean Karn"]]);

        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn test_top_level() -> Result<(), Box<dyn std::error::Error>> {
        let actions = Parser::parse_multi(&[
            Parsable::new("existing_key", "rename_from_existing_key"),
            Parsable::new("my_array[0]", "used_to_be_array"),
            Parsable::new(r#"const("consant_value")"#, "const"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let input = json!({
            "existing_key":"my_val1",
            "my_array":["idx_0_value"]
        });
        let expected = json!({"const":"consant_value","rename_from_existing_key":"my_val1","used_to_be_array":"idx_0_value"});
        let output = trans.apply(&input)?;
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_10_top_level() -> Result<(), Box<dyn std::error::Error>> {
        let actions = Parser::parse_multi(&[
            Parsable::new("top1", "new1"),
            Parsable::new("top2", "new2"),
            Parsable::new("top3", "new3"),
            Parsable::new("top4", "new4"),
            Parsable::new("top5", "new5"),
            Parsable::new("top6", "new6"),
            Parsable::new("top7", "new7"),
            Parsable::new("top8", "new8"),
            Parsable::new("top9", "new9"),
            Parsable::new("top10", "new10"),
        ])?;

        let trans = TransformBuilder::default().add_actions(actions).build()?;

        let input = json!({
            "top1": "value",
            "top2": "value",
            "top3": "value",
            "top4": "value",
            "top5": "value",
            "top6": "value",
            "top7": "value",
            "top8": "value",
            "top9": "value",
            "top10": "value"
        });
        let expected = json!({"new1":"value","new10":"value","new2":"value","new3":"value","new4":"value","new5":"value","new6":"value","new7":"value","new8":"value","new9":"value"});
        let output = trans.apply(&input)?;
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_join() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(
            r#"join(" ", const("Mr."), first_name, meta.middle_name, last_name)"#,
            "full_name",
        )?;
        let trans = TransformBuilder::default().add_action(action).build()?;

        let input = json!({
            "first_name": "Dean",
            "last_name": "Karn",
            "meta": {
                "middle_name":"Peter"
            }
        });
        let expected = json!({"full_name":"Mr. Dean Peter Karn"});
        let output = trans.apply(&input)?;
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_explicit_key() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(r#"["name(1)"]"#, r#"["my name is ([2][])"]"#)?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = json!({"name(1)":"Dean Karn"});
        let destination = trans.apply(&source)?;
        assert!(destination.is_object());

        let expected = json!({"my name is ([2][])": "Dean Karn"});

        assert_eq!(expected, destination);

        let action = Parser::parse(r#"["name(1)"].name"#, r#"["my name is ([2][])"]"#)?;
        let trans = TransformBuilder::default().add_action(action).build()?;
        let source = json!({"name(1)":{"name":"Dean Karn"}});
        let destination = trans.apply(&source)?;
        assert!(destination.is_object());

        let expected = json!({"my name is ([2][])": "Dean Karn"});
        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn merge_object() -> Result<(), Box<dyn std::error::Error>> {
        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "person.full_name"),
            Parsable::new("person.metadata", "person{}"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let source = json!({"person":{"name":"Dean Karn", "metadata":{"age":1}}});
        let destination = trans.apply(&source)?;
        let expected = json!({"person":{"full_name":"Dean Karn", "age":1}});
        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn combine_array() -> Result<(), Box<dyn std::error::Error>> {
        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "person[0]"),
            Parsable::new("person.metadata", "person[+]"), // CombineArray = [+], MergeArray = [-]
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let source = json!({"person":{"name":"Dean Karn", "metadata":[1]}});
        let destination = trans.apply(&source)?;
        let expected = json!({"person":["Dean Karn", 1]});
        assert_eq!(expected, destination);

        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "[0]"),
            Parsable::new("person.metadata", "[+]"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let source = json!({"person":{"name":"Dean Karn", "metadata":[1]}});
        let mut destination = Value::Array(vec![1.into()]);
        let _ = trans.apply_to_destination(&source, &mut destination);
        let expected = json!(["Dean Karn", 1]);
        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn replace_array() -> Result<(), Box<dyn std::error::Error>> {
        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "person[0]"),
            Parsable::new("person.metadata", "person[0]"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let source = json!({"person":{"name":"Dean Karn", "metadata":[1]}});
        let destination = trans.apply(&source)?;
        let expected = json!({"person":[[1]]});
        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn merge_array() -> Result<(), Box<dyn std::error::Error>> {
        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "person[0]"),
            Parsable::new("person.metadata", "person[-]"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let source = json!({"person":{"name":"Dean Karn", "metadata":[1]}});
        let destination = trans.apply(&source)?;
        let expected = json!({"person":[1]});
        assert_eq!(expected, destination);

        // test source len > existing
        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "person[0]"),
            Parsable::new("person.metadata", "person[-]"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let source = json!({"person":{"name":"Dean Karn", "metadata":[1, "blah", 45.6]}});
        let destination = trans.apply(&source)?;
        let expected = json!({"person":[1,"blah",45.6]});
        assert_eq!(expected, destination);

        // test source len < existing
        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "person[5]"),
            Parsable::new("person.metadata", "person[-]"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let source = json!({"person":{"name":"Dean Karn", "metadata":[1, "blah", 45.6]}});
        let destination = trans.apply(&source)?;
        let expected = json!({"person":[1, "blah", 45.6, null, null, "Dean Karn"]});
        assert_eq!(expected, destination);
        Ok(())
    }

    #[test]
    fn transformer_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let actions = Parser::parse_multi(&[
            Parsable::new("person.name", "person[0]"),
            Parsable::new("person.metadata", "person[0]"),
        ])?;
        let trans = TransformBuilder::default().add_actions(actions).build()?;
        let res = serde_json::to_string(&trans)?;
        assert_eq!(res, "{\"actions\":[{\"Setter\":{\"namespace\":[{\"Object\":{\"id\":\"person\"}},{\"Array\":{\"index\":0}}],\"child\":{\"Getter\":{\"namespace\":[{\"Object\":{\"id\":\"person\"}},{\"Object\":{\"id\":\"name\"}}]}}}},{\"Setter\":{\"namespace\":[{\"Object\":{\"id\":\"person\"}},{\"Array\":{\"index\":0}}],\"child\":{\"Getter\":{\"namespace\":[{\"Object\":{\"id\":\"person\"}},{\"Object\":{\"id\":\"metadata\"}}]}}}}]}");
        Ok(())
    }
}
