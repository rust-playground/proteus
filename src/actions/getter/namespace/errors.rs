use std::num::ParseIntError;
use thiserror::Error;

/// This type represents all possible errors that an occur while parsing transformation syntax to generate a [Namespace](enum.Namespace.html) to be used in [Getter](../struct.Getter.html).
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid '.' notation for namespace: {}. {}", ns, err)]
    InvalidDotNotation { err: String, ns: String },

    #[error(transparent)]
    InvalidNamespaceArrayIndex(#[from] ParseIntError),

    #[error("Missing end bracket ']' in array index for namespace: {0}")]
    MissingArrayIndexBracket(String),

    #[error("Invalid Explicit Key Syntax for namespace {0}. Explicit Key Syntax must start with '[\"' and end with '\"]' with any enclosed '\"' escaped.")]
    InvalidExplicitKeySyntax(String),
}
