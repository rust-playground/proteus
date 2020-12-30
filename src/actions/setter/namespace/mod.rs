mod errors;

pub use errors::Error;

use crate::actions::setter::namespace::Error as SetterErr;
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
    /// Represents an id/location for an Object within the destination data.
    Object { id: String },

    /// Represents that the [Setter](../struct.Setter.html) should merge the source and destination
    /// JSON Objects.
    MergeObject,

    /// Represents an index/location for an Array within the destination data.
    Array { index: usize },

    /// Represents that the [Setter](../struct.Setter.html) should append the source data to the
    /// destination JSON Array.
    AppendArray,

    /// Represents that the [Setter](../struct.Setter.html) should merge the source and destination
    /// JSON Arrays.
    MergeArray,

    /// Represents that the [Setter](../struct.Setter.html) should combine the source JSON Array to
    /// the destination JSON Array by appending all array elements from the source Array to the
    /// destinations.
    CombineArray,
}

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Namespace::Object { id } => write!(f, "{}", id),
            Namespace::MergeObject => write!(f, "{{}}"),
            Namespace::AppendArray => write!(f, "[]"),
            Namespace::MergeArray => write!(f, "[-]"),
            Namespace::CombineArray => write!(f, "[+]"),
            Namespace::Array { index } => write!(f, "[{}]", index),
        }
    }
}

impl Namespace {
    /// parses a transformation syntax string into an Vec of [Namespace](enum.Namespace.html)'s for
    /// use in the [Setter](../struct.Setter.html).
    ///
    /// The transformation syntax is very similar to access JSON data in Javascript with a few additions:
    /// * `{}` eg. test.value{} which denotes that the source Object and destination Object `value` should merge their data instead of the source replace the destination value
    /// * `[]` eg. test.value[] which denotes that the source data should be appended to the Array `value` rather than replacing the destination value.
    /// * `[+]` eg. test.value[+] which denotes that the source Array should append all of it's values onto the destination Array.
    /// * `[-]` eg. test.value[-] which denotes that the source Array values should replace the destination Array's values at the overlapping indexes.
    /// NOTE: `{}`, `[+]` and `[-]` can only be used on the last element of the Namespace syntax.
    ///
    /// To handle special characters such as ``(blank), `[`, `]`, `"` and `.` you can use the explicit
    /// key syntax `["example[].blah"]` which would represent the key in the following JSON:
    /// ```json
    /// {
    ///   "example[].blah" : "my value"
    /// }
    /// ```
    pub fn parse(input: &str) -> Result<Vec<Namespace>, SetterErr> {
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
                b'{' => {
                    if !s.is_empty() {
                        namespaces.push(Namespace::Object {
                            id: unsafe { String::from_utf8_unchecked(s.clone()) },
                        });
                        s.clear();
                    }
                    // merge object syntax
                    idx += 1;
                    if idx < bytes.len() && bytes[idx] != b'}' {
                        // error invalid merge object syntax
                        return Err(Error::InvalidMergeObjectSyntax(input.to_owned()));
                    }
                    idx += 1;
                    if idx != bytes.len() {
                        // error merge object must be the last part in the namespace.
                        return Err(Error::InvalidMergeObjectSyntax(input.to_owned()));
                    }
                    namespaces.push(Namespace::MergeObject);
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
                    match bytes[idx] {
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
                            return Err(Error::InvalidExplicitKeySyntax(input.to_owned()));
                        }
                        b']' => {
                            // append array index
                            namespaces.push(Namespace::AppendArray);
                            idx += 1;
                            continue 'outer;
                        }
                        b'-' => {
                            // merge array
                            idx += 1;
                            if idx < bytes.len() && bytes[idx] != b']' {
                                // error invalid merge object syntax
                                return Err(Error::InvalidMergeArraySyntax(input.to_owned()));
                            }
                            idx += 1;
                            if idx != bytes.len() {
                                // error merge object must be the last part in the namespace.
                                return Err(Error::InvalidMergeArraySyntax(input.to_owned()));
                            }
                            namespaces.push(Namespace::MergeArray);
                        }
                        b'+' => {
                            // merge array
                            idx += 1;
                            if idx < bytes.len() && bytes[idx] != b']' {
                                // error invalid merge object syntax
                                return Err(Error::InvalidCombineArraySyntax(input.to_owned()));
                            }
                            idx += 1;
                            if idx != bytes.len() {
                                // error merge object must be the last part in the namespace.
                                return Err(Error::InvalidCombineArraySyntax(input.to_owned()));
                            }
                            namespaces.push(Namespace::CombineArray);
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
                            return Err(Error::MissingArrayIndexBracket(input.to_owned()));
                        }
                    }
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
    fn test_direct_set() {
        let ns = "";
        let results = Namespace::parse(ns).unwrap();
        let expected: Vec<Namespace> = Vec::new();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_object_merge() {
        let ns = "person{}";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "person".into(),
            },
            Namespace::MergeObject,
        ];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_array_merge() {
        let ns = "person[-]";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "person".into(),
            },
            Namespace::MergeArray,
        ];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_array_combine() {
        let ns = "person[+]";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "person".into(),
            },
            Namespace::CombineArray,
        ];
        assert_eq!(expected, results);
    }

    #[test]
    fn test_append_array() {
        let ns = "person[]";
        let results = Namespace::parse(ns).unwrap();
        let expected = vec![
            Namespace::Object {
                id: "person".into(),
            },
            Namespace::AppendArray,
        ];
        assert_eq!(expected, results);
    }
}
