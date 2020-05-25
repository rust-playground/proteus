use std::num::ParseIntError;
use thiserror::Error;

/// This type represents all possible errors that an occur while parsing transformation syntax to generate a [Namespace](enum.Namespace.html) to be used in [Setter](../struct.Setter.html).
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid '.' notation for namespace: {}. {}", ns, err)]
    InvalidDotNotation { err: String, ns: String },

    #[error(transparent)]
    InvalidNamespaceArrayIndex(#[from] ParseIntError),

    #[error("Missing end bracket ']' in array index for namespace: {0}")]
    MissingArrayIndexBracket(String),

    #[error("Invalid Merge Object Syntax for namespace: {0}. Merge Object Syntax must be exactly '{{}}' and is only valid at the end of the namespace.")]
    InvalidMergeObjectSyntax(String),

    #[error("Invalid Merge Array Syntax for namespace: {0}. Merge Array Syntax must be exactly '[-]' and is only valid at the end of the namespace.")]
    InvalidMergeArraySyntax(String),

    #[error("Invalid Combine Array Syntax for namespace: {0}. Combine Array Syntax must be exactly '[+]' and is only valid at the end of the namespace.")]
    InvalidCombineArraySyntax(String),

    #[error("Invalid Explicit Key Syntax for namespace {0}. Explicit Key Syntax must start with '[\"' and end with '\"]' with any enclosed '\"' escaped.")]
    InvalidExplicitKeySyntax(String),
}
