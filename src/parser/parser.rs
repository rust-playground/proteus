use crate::action::Action;
use crate::actions::getter::namespace::Namespace as GetterNamespace;
use crate::actions::setter::namespace::Namespace as SetterNamespace;
use crate::actions::{Getter, ParsableConst, ParsableJoin, Setter};
use crate::parser::errors::Error;
use crate::parser::ParsableAction;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

lazy_static! {
    static ref ACTION_RE: Regex = Regex::new(r#"(?P<action>[a-zA-Z]*)\((?P<value>.*)\)"#).unwrap();
    pub static ref COMMA_SEP_RE: Regex = Regex::new(r#"[^,(]*(?:\([^)]*\))*[^,]*"#).unwrap();
    pub static ref QUOTED_STR_RE: Regex = Regex::new(r#"^"(.*?[^\\])"\s*,"#).unwrap();
}

const ACTION_NAME: &str = "action";
const ACTION_VALUE: &str = "value";

/// This type represents a single transformation action to be taken containing the source and
/// destination syntax to be parsed into an [Action](action/trait.Action.html).
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Parsable<'a> {
    source: Cow<'a, str>,
    destination: Cow<'a, str>,
}

impl<'a> Parsable<'a> {
    pub fn new<S>(source: S, destination: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Parsable {
            source: source.into(),
            destination: destination.into(),
        }
    }
}

/// This type represents a set of static methods for parsing transformation syntax into
/// [Action](action/trait.Action.html)'s.
///
/// The parser is responsible for parsing the transformation action specific syntax, take the
/// following source syntax: `join(" ", const("Mr."), first_name, last_name)`
/// the parser knows how to breakdown the syntax into a `join` action which calls the `const`
/// action, and 2 getter actions and joins those actions results.
///
/// Actions currently supported include:
/// * const eg. `const(<value>)`
/// * join eg. `join(<separator, <variadic of actions>)
///
pub struct Parser {
    actions: HashMap<String, Box<dyn ParsableAction>>,
}

impl Default for Parser {
    fn default() -> Self {
        let mut actions: HashMap<String, Box<dyn ParsableAction>> = HashMap::new();
        actions.insert("const".to_string(), Box::new(ParsableConst));
        actions.insert("join".to_string(), Box::new(ParsableJoin));
        Self { actions }
    }
}

impl Parser {
    //    // is a way to create a parser from scratch with no existing actions for parsing
    //    pub fn new(actions: HashMap<String, Box<dyn Action>>) -> Self {
    //        Self { actions }
    //    }

    /// parses a single transformation action to be taken with the provided source & destination.
    pub fn parse(&self, source: &str, destination: &str) -> Result<Box<dyn Action>, Error> {
        let set = SetterNamespace::parse(destination)?;
        let action = self.get_action(source)?;
        Ok(Box::new(Setter::new(set, action)))
    }

    /// parses a set of transformation actions into [Action](action/trait.Action.html)'s.
    pub fn parse_multi(&self, parsables: &[Parsable]) -> Result<Vec<Box<dyn Action>>, Error> {
        let mut vec = Vec::new();
        for p in parsables.iter() {
            vec.push(self.parse(&p.source, &p.destination)?);
        }
        Ok(vec)
    }

    /// parses a set of transformation actions into [Action](action/trait.Action.html)'s from a JSON
    /// string of serialized [Parsable](struct.Parsable.html) structs.
    pub fn parse_multi_from_str(&self, s: &str) -> Result<Vec<Box<dyn Action>>, Error> {
        let parsables: Vec<Parsable> = serde_json::from_str(s)?;
        self.parse_multi(&parsables)
    }

    // TODO: recursive, limit recursion to a hard max. I'm sure there's a way to make it NOT recursive to avoid a stack overflow
    //       but not sure it's worth it as nobody should be making such a complex action in the first place and would likely cause a
    //       stack overflow at runtime even if it could be parsed, so not worth the extra complexity.
    pub fn get_action(&self, source: &str) -> Result<Box<dyn Action>, Error> {
        // edge case where there is no action but it looks like there's one inside of an
        // explicit key eg. '["const()"]'
        if source.starts_with(r#"[""#) {
            let get = GetterNamespace::parse(source)?;
            return Ok(Box::new(Getter::new(get)));
        }
        match ACTION_RE.captures(source) {
            Some(caps) => match caps.name(ACTION_NAME) {
                None => Err(Error::MissingActionName {}),
                Some(key) => {
                    let key = key.as_str();
                    match self.actions.get(key) {
                        Some(ac) => {
                            // unwrap safe, has value or never would have match ACTION_RE regex
                            ac.parse(self, caps.name(ACTION_VALUE).unwrap().as_str())
                        }
                        None => Err(Error::InvalidActionName {
                            name: key.to_owned(),
                        }),
                    }
                }
            },
            None => {
                let get = GetterNamespace::parse(source)?;
                Ok(Box::new(Getter::new(get)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Constant;

    #[test]
    fn direct_getter() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::default().parse("key", "new")?;
        let expected = Box::new(Setter::new(
            SetterNamespace::parse("new")?,
            Box::new(Getter::new(GetterNamespace::parse("key")?)),
        ));
        assert_eq!(format!("{:?}", action), format!("{:?}", expected));
        Ok(())
    }

    #[test]
    fn constant() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::default().parse(r#"const("value")"#, "new")?;
        let expected = Box::new(Setter::new(
            SetterNamespace::parse("new")?,
            Box::new(Constant::new("value".into())),
        ));
        assert_eq!(format!("{:?}", action), format!("{:?}", expected));
        Ok(())
    }

    #[test]
    fn parser_serialize_deserialize() -> Result<(), Box<dyn std::error::Error>> {
        let parsables = vec![
            Parsable::new(r#"const("value")"#, "new"),
            Parsable::new(r#"const("value2")"#, "new2"),
        ];
        let serialized = serde_json::to_string(&parsables)?;
        let expected = "[{\"source\":\"const(\\\"value\\\")\",\"destination\":\"new\"},{\"source\":\"const(\\\"value2\\\")\",\"destination\":\"new2\"}]";
        assert_eq!(expected, serialized);

        let deserialized: Vec<Parsable> = serde_json::from_str(&serialized)?;
        assert_eq!(parsables, deserialized);
        Ok(())
    }

    #[test]
    fn parser_from_str() -> Result<(), Box<dyn std::error::Error>> {
        let parsables = vec![
            Parsable::new(r#"const("value")"#, "new"),
            Parsable::new(r#"const("value2")"#, "new2"),
        ];
        let expected = Parser::default().parse_multi(&parsables)?;
        let deserialized = Parser::default().parse_multi_from_str("[{\"source\":\"const(\\\"value\\\")\",\"destination\":\"new\"},{\"source\":\"const(\\\"value2\\\")\",\"destination\":\"new2\"}]")?;
        assert_eq!(format!("{:?}", expected), format!("{:?}", deserialized));
        Ok(())
    }

    #[test]
    fn join() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::default().parse(
            r#"join(",_" , first_name, last_name, const("Dean Karn"))"#,
            "full_name",
        )?;
        let expected = "Setter { namespace: [Object { id: \"full_name\" }], child: Join { sep: \",_\", values: [Getter { namespace: [Object { id: \"first_name\" }] }, Getter { namespace: [Object { id: \"last_name\" }] }, Constant { value: String(\"Dean Karn\") }] } }";
        assert_eq!(format!("{:?}", action), format!("{}", expected));
        Ok(())
    }
}
