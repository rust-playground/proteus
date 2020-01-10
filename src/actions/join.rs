use crate::action::Action;
use crate::errors::Error;
use crate::parser::Error as ParseError;
use crate::parser::{ParsableAction, COMMA_SEP_RE, QUOTED_STR_RE};
use crate::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    fn apply(&self, source: &Value, destination: &mut Value) -> Result<Option<Value>, Error> {
        let l = self.values.len() - 1;
        let mut result = String::new();
        for (i, v) in self.values.iter().enumerate() {
            match v.apply(source, destination)? {
                Some(v) => {
                    match v {
                        Value::String(s) => {
                            if s.is_empty() {
                                continue;
                            }
                            result.push_str(&s);
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
        Ok(Some(Value::String(result)))
    }
}

#[derive(Debug)]
pub struct ParsableJoin;

impl ParsableAction for ParsableJoin {
    fn parse(&self, parser: &Parser, value: &str) -> Result<Box<dyn Action>, ParseError> {
        let sep_len;
        let sep = match QUOTED_STR_RE.find(value) {
            Some(cap) => {
                let s = cap.as_str();
                sep_len = s.len();
                let s = s[..s.len() - 1].trim(); // strip ',' and trim any whitespace
                s[1..s.len() - 1].to_string() // remove '"" double quotes from beginning and end.
            }
            None => {
                return Err(ParseError::InvalidQuotedValue {
                    key: format!("join({})", value),
                })
            }
        };

        let sub_matches = COMMA_SEP_RE.captures_iter(&value[sep_len..]);
        let mut values = Vec::new();
        for m in sub_matches {
            match m.get(0) {
                Some(m) => values.push(parser.get_action(m.as_str().trim())?),
                None => continue,
            };
        }

        if values.is_empty() {
            return Err(ParseError::InvalidNumberOfProperties {
                key: "join".to_owned(),
            });
        }
        Ok(Box::new(Join::new(sep, values)))
    }
}
