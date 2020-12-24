use crate::actions::getter::namespace::Error as GetterNamespaceError;
use crate::actions::setter::namespace::Error as SetterNamespaceError;
use crate::errors::Error as JSONError;
use thiserror::Error;

/// This type represents all possible errors that an occur while parsing the transformation syntax.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] JSONError),

    #[error("Brackets: () must always be preceded by and action name.")]
    MissingActionName,

    #[error("Action Name: '{0}' is not recognized.")]
    InvalidActionName(String),

    #[error(
        "Action Value missing for key:{0}. An action Value must be set in brackets eg. const(null)"
    )]
    MissingActionValue(String),

    #[error("Issue parsing Action Value: {0}")]
    ValueParseError(#[from] serde_json::Error),

    #[error("Invalid number of properties supplied to Action: '{0}'")]
    InvalidNumberOfProperties(String),

    #[error("Invalid quoted value supplied for Action: '{0}'")]
    InvalidQuotedValue(String),

    #[error("Setter namespace parsing error: {0}")]
    GetterNamespace(#[from] GetterNamespaceError),

    #[error("Setter namespace parsing error: {0}")]
    SetterNamespace(#[from] SetterNamespaceError),

    #[error("{0}")]
    CustomActionParseError(String),
}
