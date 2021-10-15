use proteus::action::Action;
use proteus::parser::Error;
use proteus::{actions, Parser, TransformBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::ops::Deref;

// This example shows how to create, register and use a custom Action
fn main() -> Result<(), Box<dyn std::error::Error>> {
    proteus::Parser::add_action_parser("custom", &parse_custom)?;

    let input = get_input();
    let trans = TransformBuilder::default()
        .add_actions(actions!((r#"custom(id)"#, "custom_id"))?)
        .build()?;
    let res = trans.apply_from_str(input)?;
    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}

fn get_input() -> &'static str {
    r#"{"id": "01234"}"#
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomAction {
    action: Box<dyn Action>,
}

impl CustomAction {
    pub fn new(action: Box<dyn Action>) -> Self {
        Self { action }
    }
}

#[typetag::serde]
impl Action for CustomAction {
    fn apply<'a>(
        &self,
        _source: &'a Value,
        _destination: &mut Value,
    ) -> Result<Option<Cow<'a, Value>>, proteus::Error> {
        match self.action.apply(_source, _destination) {
            Ok(v) => match v {
                None => Ok(None),
                Some(v) => match v.deref() {
                    Value::String(s) => Ok(Some(Cow::Owned(Value::String(
                        s.to_owned() + " from my custom function",
                    )))),
                    _ => Ok(Some(Cow::Owned(Value::String(
                        v.to_string() + " from my custom function",
                    )))),
                },
            },
            Err(e) => Err(e),
        }
    }
}

fn parse_custom(val: &str) -> Result<Box<dyn Action>, Error> {
    if val.is_empty() {
        Err(Error::MissingActionValue("custom".to_owned()))
    } else {
        let inner_action = Parser::parse_action(val)?;
        Ok(Box::new(CustomAction::new(inner_action)))
    }
}
