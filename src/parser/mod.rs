//! Parser of transformation syntax into [Action(s)](action/trait.Action.html).

mod action_parsers;
mod errors;

pub use errors::Error;

use crate::action::Action;
use crate::actions::getter::namespace::Namespace as GetterNamespace;
use crate::actions::setter::namespace::Namespace as SetterNamespace;
use crate::actions::{Getter, Setter};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// This is a Regex used to parse comma separated values and is used as a helper within custom
/// Action Parsers.
pub static COMMA_SEP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[^,(]*(?:\([^)]*\))*[^,]*"#).unwrap());

/// This is a Regex used to get content within quoted strings and is used as a helper within custom
/// Action Parsers.
pub static QUOTED_STR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^"(.*?[^\\])"\s*,"#).unwrap());

static ACTION_RE: Lazy<Regex> = Lazy::new(|| {
    let r = format!(r#"(?P<action>{})\((?P<value>.*)\)"#, ACTION_NAME_BASE_REGEX);
    Regex::new(&r).unwrap()
});

static ACTION_PARSERS: Lazy<Mutex<HashMap<String, Arc<ActionParserFn>>>> = Lazy::new(|| {
    let mut m: HashMap<String, Arc<ActionParserFn>> = HashMap::new();
    m.insert("join".to_string(), Arc::new(action_parsers::parse_join));
    m.insert("const".to_string(), Arc::new(action_parsers::parse_const));
    m.insert("len".to_string(), Arc::new(action_parsers::parse_len));
    m.insert("sum".to_string(), Arc::new(action_parsers::parse_sum));
    m.insert("trim".to_string(), Arc::new(action_parsers::parse_trim));
    m.insert(
        "trim_start".to_string(),
        Arc::new(action_parsers::parse_trim_start),
    );
    m.insert(
        "trim_end".to_string(),
        Arc::new(action_parsers::parse_trim_end),
    );
    m.insert(
        "strip_prefix".to_string(),
        Arc::new(action_parsers::parse_strip_prefix),
    );
    m.insert(
        "strip_suffix".to_string(),
        Arc::new(action_parsers::parse_strip_suffix),
    );
    Mutex::new(m)
});

static ACTION_NAME_RE: Lazy<Regex> = Lazy::new(|| {
    let r = format!("^{}$", ACTION_NAME_BASE_REGEX);
    Regex::new(&r).unwrap()
});

const ACTION_NAME_BASE_REGEX: &str = "[a-zA-Z0-9_]+";
const ACTION_NAME: &str = "action";
const ACTION_VALUE: &str = "value";

/// ActionParserFn is function signature used for adding dynamic actions to the parser
pub type ActionParserFn = dyn Fn(&str) -> Result<Box<dyn Action>, Error> + 'static + Send + Sync;

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
pub struct Parser {}

impl Parser {
    /// add_action_parser adds an Action parsing function to dynamically be parsed.
    /// NOTE: this WILL overwrite any pre-existing functions with the same name.
    ///
    /// name only accepts ASCII letters, numbers and _ equivalent to [a-zA-Z0-9_].
    pub fn add_action_parser(name: &str, f: &'static ActionParserFn) -> Result<(), Error> {
        if !ACTION_NAME_RE.is_match(name) {
            return Err(Error::InvalidActionName(name.to_owned()));
        }
        ACTION_PARSERS
            .lock()
            .unwrap()
            .insert(name.to_owned(), Arc::new(f));
        Ok(())
    }

    /// parses a single transformation action to be taken with the provided source & destination.
    pub fn parse(source: &str, destination: &str) -> Result<Box<dyn Action>, Error> {
        let set = SetterNamespace::parse(destination)?;
        let action = Parser::parse_action(source)?;
        Ok(Box::new(Setter::new(set, action)))
    }

    /// parses a set of transformation actions into [Action](action/trait.Action.html)'s.
    pub fn parse_multi(parsables: &[Parsable]) -> Result<Vec<Box<dyn Action>>, Error> {
        let mut vec = Vec::new();
        for p in parsables.iter() {
            vec.push(Parser::parse(&p.source, &p.destination)?);
        }
        Ok(vec)
    }

    /// parses a set of transformation actions into [Action](action/trait.Action.html)'s from a JSON
    /// string of serialized [Parsable](struct.Parsable.html) structs.
    pub fn parse_multi_from_str(s: &str) -> Result<Vec<Box<dyn Action>>, Error> {
        let parsables: Vec<Parsable> = serde_json::from_str(s)?;
        Parser::parse_multi(&parsables)
    }

    /// parses an [Action](action/trait.Action.html) given the provided str. This is primarily used
    /// as a helper in custom Action Parsers.
    pub fn parse_action(source: &str) -> Result<Box<dyn Action>, Error> {
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
                    let parse_fn;
                    match ACTION_PARSERS.lock().unwrap().get(key) {
                        None => return Err(Error::InvalidActionName(key.to_owned())),
                        Some(f) => {
                            parse_fn = f.clone();
                        }
                    };
                    parse_fn(caps.name(ACTION_VALUE).unwrap().as_str()) // unwrap safe, has value or never would have match ACTION_RE regex
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
        let action = Parser::parse("key", "new")?;
        let expected = Box::new(Setter::new(
            SetterNamespace::parse("new")?,
            Box::new(Getter::new(GetterNamespace::parse("key")?)),
        ));
        assert_eq!(format!("{:?}", action), format!("{:?}", expected));
        Ok(())
    }

    #[test]
    fn constant() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(r#"const("value")"#, "new")?;
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
        let expected = Parser::parse_multi(&parsables)?;
        let deserialized = Parser::parse_multi_from_str("[{\"source\":\"const(\\\"value\\\")\",\"destination\":\"new\"},{\"source\":\"const(\\\"value2\\\")\",\"destination\":\"new2\"}]")?;
        assert_eq!(format!("{:?}", expected), format!("{:?}", deserialized));
        Ok(())
    }

    #[test]
    fn join() -> Result<(), Box<dyn std::error::Error>> {
        let action = Parser::parse(
            r#"join(",_" , first_name, last_name, const("Dean Karn"))"#,
            "full_name",
        )?;
        let expected = "Setter { namespace: [Object { id: \"full_name\" }], child: Join { sep: \",_\", values: [Getter { namespace: [Object { id: \"first_name\" }] }, Getter { namespace: [Object { id: \"last_name\" }] }, Constant { value: String(\"Dean Karn\") }] } }";
        assert_eq!(format!("{:?}", action), expected.to_string());
        Ok(())
    }
}
