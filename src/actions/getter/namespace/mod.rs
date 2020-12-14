mod errors;

pub use errors::Error;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Represents a single group/level of JSON structures used for traversing JSON structures.
///
/// # Example
/// ```json
/// {
///   "test" : { "value" : "my value" }
/// }
/// ```
/// `test.value` would be represented by two Namespace Object's `test` and `value` as a way to
/// traverse the JSON data to point at `my value`.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Namespace {
    /// Represents an id/location within the source data's Object
    Object { id: String },

    /// Represents an index/location within the source data's JSON Array.
    Array { index: usize },
}

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Namespace::Object { id } => write!(f, "{}", id),
            Namespace::Array { index } => write!(f, "[{}]", index),
        }
    }
}

impl Namespace {
    /// parses a transformation syntax string into an Vec of [Namespace](enum.Namespace.html)'s for
    /// use in the [Getter](../struct.Getter.html).
    ///
    /// The transformation syntax is very similar to access JSON data in Javascript.
    ///
    /// To handle special characters such as ``(blank), `[`, `]`, `"` and `.` you can use the explicit
    /// key syntax `["example[].blah"]` which would represent the key in the following JSON:
    /// ```json
    /// {
    ///   "example[].blah" : "my value"
    /// }
    /// ```
    pub fn parse(input: &str) -> Result<Vec<Namespace>, Error> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let bytes = input.as_bytes();
        let mut namespaces = Vec::new();
        let mut idx = 0;
        let mut s = Vec::with_capacity(10);

        'outer: while idx < bytes.len() {
            let b = bytes[idx];
            match b {
                b'.' => {
                    if s.is_empty() {
                        // empty values must be via explicit key
                        // might also be ending to other types eg. array.
                        if idx == 0 || idx + 1 == bytes.len() {
                            // cannot start with '.', if want a blank key must use explicit key syntax
                            return Err(Error::InvalidDotNotation {
                                ns: input.to_owned(),
                                err: r#"Namespace cannot start or end with '.', explicit key syntax of '[""]' must be used to denote a blank key."#.to_owned(),
                            });
                        }
                        idx += 1;
                        continue;
                    }
                    namespaces.push(Namespace::Object {
                        id: unsafe { String::from_utf8_unchecked(s.clone()) },
                    });
                    s.clear();
                    idx += 1;
                    continue;
                }
                b'[' => {
                    if !s.is_empty() {
                        // this syntax named[..] lets create the object
                        namespaces.push(Namespace::Object {
                            id: unsafe { String::from_utf8_unchecked(s.clone()) },
                        });
                        s.clear();
                    }
                    idx += 1;
                    if idx >= bytes.len() {
                        // error incomplete namespace
                        return Err(Error::MissingArrayIndexBracket(input.to_owned()));
                    }
                    return match bytes[idx] {
                        b'"' => {
                            // parse explicit key
                            idx += 1;
                            while idx < bytes.len() {
                                let b = bytes[idx];
                                match b {
                                    b'"' if bytes[idx - 1] != b'\\' => {
                                        idx += 1;
                                        if bytes[idx] != b']' {
                                            // error invalid explicit key syntax
                                            return Err(Error::InvalidExplicitKeySyntax(
                                                input.to_owned(),
                                            ));
                                        }
                                        namespaces.push(Namespace::Object {
                                            id: unsafe { String::from_utf8_unchecked(s.clone()) }
                                                .replace("\\", ""), // unescape required escaped double quotes
                                        });
                                        s.clear();
                                        idx += 1;
                                        continue 'outer;
                                    }
                                    _ => {
                                        idx += 1;
                                        s.push(b)
                                    }
                                };
                            }
                            // error never reached the end bracket of explicit key
                            Err(Error::InvalidExplicitKeySyntax(input.to_owned()))
                        }
                        _ => {
                            // parse array index
                            while idx < bytes.len() {
                                let b = bytes[idx];
                                match b {
                                    b']' => {
                                        namespaces.push(Namespace::Array {
                                            index: unsafe {
                                                String::from_utf8_unchecked(s.clone())
                                            }
                                            .parse()?,
                                        });
                                        s.clear();
                                        idx += 1;
                                        continue 'outer;
                                    }
                                    _ => {
                                        idx += 1;
                                        s.push(b)
                                    }
                                };
                            }
                            // error no end bracket
                            Err(Error::MissingArrayIndexBracket(input.to_owned()))
                        }
                    };
                }
                _ => {
                    s.push(b);
                    idx += 1;
                }
            };
        }

        if !s.is_empty() {
            namespaces.push(Namespace::Object {
                id: unsafe { String::from_utf8_unchecked(s) },
            });
        }
        Ok(namespaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace() {
        let ns = "embedded.array[0][1]";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: String::from("embedded"),
            },
            Namespace::Object {
                id: String::from("array"),
            },
            Namespace::Array { index: 0 },
            Namespace::Array { index: 1 },
        ];
        assert_eq!(results, expected);
    }

    #[test]
    fn test_simple() {
        let ns = "field";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![Namespace::Object {
            id: String::from("field"),
        }];
        assert_eq!(results, expected);

        let ns = "array-field[0]";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: String::from("array-field"),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(results, expected);
    }

    #[test]
    fn test_blank() {
        let ns = "";
        let results = Namespace::parse(ns).unwrap();
        let expected: Vec<Namespace> = Vec::new();
        assert_eq!(results, expected);
    }

    #[test]
    fn test_blank_field() {
        let ns = ".field";
        let results = Namespace::parse(ns);
        assert!(results.is_err());
        let actual = matches!(results.err().unwrap(), Error::InvalidDotNotation { .. });
        assert!(actual);

        let ns = r#"[""].field"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: String::from(""),
            },
            Namespace::Object {
                id: String::from("field"),
            },
        ];
        assert_eq!(results, expected);
    }

    #[test]
    fn test_blank_array() {
        let ns = ".[0]";
        let results = Namespace::parse(ns);
        assert!(results.is_err());
        let actual = matches!(results.err().unwrap(), Error::InvalidDotNotation { .. });
        assert!(actual);

        let ns = r#"[""].[0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: String::from(""),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);

        let ns = r#"[""][0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: String::from(""),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);

        let ns = ".named[0]";
        let results = Namespace::parse(ns);
        assert!(results.is_err());
        let actual = matches!(results.err().unwrap(), Error::InvalidDotNotation { .. });
        assert!(actual);

        let ns = r#"[""].named[0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: String::from(""),
            },
            Namespace::Object {
                id: String::from("named"),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_array_blank() {
        let ns = "[0].";
        let results = Namespace::parse(ns);
        assert!(results.is_err());
        let actual = matches!(results.err().unwrap(), Error::InvalidDotNotation { .. });
        assert!(actual);

        let ns = "[0]";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![Namespace::Array { index: 0 }];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_array_named() {
        let ns = "[0].named";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Array { index: 0 },
            Namespace::Object {
                id: String::from("named"),
            },
        ];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_explicit_key() {
        let ns = r#"["embedded.array[0][1]"]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![Namespace::Object {
            id: String::from("embedded.array[0][1]"),
        }];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_explicit_key_array() {
        let ns = r#"["embedded.array[0][1]"][0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: String::from("embedded.array[0][1]"),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_explicit_key_nested() {
        let ns = r#"name.["embedded.array[0][1]"]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "name".to_owned(),
            },
            Namespace::Object {
                id: String::from("embedded.array[0][1]"),
            },
        ];
        assert_eq!(expected, results);

        let ns = r#"name.["embedded.array[0][1]"][0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "name".to_owned(),
            },
            Namespace::Object {
                id: "embedded.array[0][1]".to_owned(),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);

        let ns = r#"["embedded.array[0][1]"][0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "embedded.array[0][1]".to_owned(),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);

        let ns = r#"[1].["embedded.array[0][1]"][0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Array { index: 1 },
            Namespace::Object {
                id: "embedded.array[0][1]".to_owned(),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);

        let ns = r#"named[1].["embedded.array[0][1]"][0]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "named".to_owned(),
            },
            Namespace::Array { index: 1 },
            Namespace::Object {
                id: "embedded.array[0][1]".to_owned(),
            },
            Namespace::Array { index: 0 },
        ];
        assert_eq!(expected, results);

        let ns = r#"named[1].["embedded.array[0][1]"]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "named".to_owned(),
            },
            Namespace::Array { index: 1 },
            Namespace::Object {
                id: "embedded.array[0][1]".to_owned(),
            },
        ];
        assert_eq!(expected, results);

        let ns = r#"["name()"].name"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "name()".to_owned(),
            },
            Namespace::Object {
                id: String::from("name"),
            },
        ];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_explicit_key_quotes() {
        let ns = r#"["""]"#;
        let results = Namespace::parse(ns);
        assert!(results.is_err());
        let actual = matches!(results.err().unwrap(), Error::InvalidExplicitKeySyntax { .. });
        assert!(actual);

        let ns = r#"["\""]"#;
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![Namespace::Object {
            id: r#"""#.to_owned(),
        }];
        assert_eq!(expected, results);
    }
}
