use snafu::Snafu;
use std::num::ParseIntError;

/// This type represents all possible errors that an occur while parsing transformation syntax to generate a [Namespace](enum.Namespace.html) to be used in [Setter](../struct.Setter.html).
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid '.' notation for namespace: {}. {}", ns, err))]
    InvalidDotNotation { err: String, ns: String },

    #[snafu(display("error: {}", err))]
    InvalidNamespaceArrayIndex { err: ParseIntError },

    #[snafu(display("Missing end bracket ']' in array index for namespace: {}", ns))]
    MissingArrayIndexBracket { ns: String },

    #[snafu(display("Invalid Merge Object Syntax for namespace: {}. Merge Object Syntax must be exactly '{{}}' and is only valid at the end of the namespace.", ns))]
    InvalidMergeObjectSyntax { ns: String },

    #[snafu(display("Invalid Merge Array Syntax for namespace: {}. Merge Array Syntax must be exactly '[-]' and is only valid at the end of the namespace.", ns))]
    InvalidMergeArraySyntax { ns: String },

    #[snafu(display("Invalid Combine Array Syntax for namespace: {}. Combine Array Syntax must be exactly '[+]' and is only valid at the end of the namespace.", ns))]
    InvalidCombineArraySyntax { ns: String },

    #[snafu(display("Invalid Explicit Key Syntax for namespace {}. Explicit Key Syntax must start with '[\"' and end with '\"]' with any enclosed '\"' escaped.", ns))]
    InvalidExplicitKeySyntax { ns: String },
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::InvalidNamespaceArrayIndex { err }
    }
}
