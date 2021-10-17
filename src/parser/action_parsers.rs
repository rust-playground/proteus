use crate::action::Action;
use crate::actions::{Constant, Join, Len, Sum};
use crate::parser::Error;
use crate::{Parser, COMMA_SEP_RE, QUOTED_STR_RE};
use serde_json::Value;

pub(super) fn parse_const(val: &str) -> Result<Box<dyn Action>, Error> {
    if val.is_empty() {
        Err(Error::MissingActionValue("const".to_owned()))
    } else {
        let value: Value = serde_json::from_str(val)?;
        Ok(Box::new(Constant::new(value)))
    }
}

pub(super) fn parse_join(val: &str) -> Result<Box<dyn Action>, Error> {
    let sep_len;
    let sep = match QUOTED_STR_RE.find(val) {
        Some(cap) => {
            let s = cap.as_str();
            sep_len = s.len();
            let s = s[..s.len() - 1].trim(); // strip ',' and trim any whitespace
            s[1..s.len() - 1].to_string() // remove '"" double quotes from beginning and end.
        }
        None => {
            return Err(Error::InvalidQuotedValue(format!("join({})", val)));
        }
    };

    let sub_matches = COMMA_SEP_RE.captures_iter(&val[sep_len..]);
    let mut values = Vec::new();
    for m in sub_matches {
        match m.get(0) {
            Some(m) => values.push(Parser::parse_action(m.as_str().trim())?),
            None => continue,
        };
    }

    if values.is_empty() {
        return Err(Error::InvalidNumberOfProperties("join".to_owned()));
    }
    Ok(Box::new(Join::new(sep, values)))
}

pub(super) fn parse_len(val: &str) -> Result<Box<dyn Action>, Error> {
    let action = Parser::parse_action(val)?;
    Ok(Box::new(Len::new(action)))
}

pub(super) fn parse_sum(val: &str) -> Result<Box<dyn Action>, Error> {
    let sub_matches = COMMA_SEP_RE.captures_iter(val);
    let mut values = Vec::new();
    for m in sub_matches {
        match m.get(0) {
            Some(m) => values.push(Parser::parse_action(m.as_str().trim())?),
            None => continue,
        };
    }

    if values.is_empty() {
        return Err(Error::InvalidNumberOfProperties("sum".to_owned()));
    }
    Ok(Box::new(Sum::new(values)))
}
