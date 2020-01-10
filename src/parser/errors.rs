use crate::actions::getter::namespace::Error as GetterNamespaceError;
use crate::actions::setter::namespace::Error as SetterNamespaceError;
use crate::errors::Error as JSONError;
use serde_json::error::Error as SerdeJSONError;
use snafu::Snafu;

/// This type represents all possible errors that an occur while parsing the transformation syntax.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", err))]
    ParseError { err: JSONError },

    #[snafu(display("Brackets: () must always be preceded by and action name."))]
    MissingActionName {},

    #[snafu(display("Action Name: '{}' is not recognized.", name))]
    InvalidActionName { name: String },

    #[snafu(display(
        "Action Value missing for key:{}. An action Value must be set in brackets eg. const(null)",
        key
    ))]
    MissingActionValue { key: String },

    #[snafu(display("Issue parsing Action Value: {}", err))]
    ValueParseError { err: SerdeJSONError },

    #[snafu(display("Invalid number of properties supplied to Action: '{}'", key))]
    InvalidNumberOfProperties { key: String },

    #[snafu(display("Invalid quoted value supplied for Action: '{}'", key))]
    InvalidQuotedValue { key: String },

    #[snafu(display("Setter namespace parsing error: {}", err))]
    GetterNamespace { err: GetterNamespaceError },

    #[snafu(display("Setter namespace parsing error: {}", err))]
    SetterNamespace { err: SetterNamespaceError },

    #[snafu(display("{}", err))]
    ParsableActionError { err: String },
}

impl From<GetterNamespaceError> for Error {
    fn from(err: GetterNamespaceError) -> Self {
        Error::GetterNamespace { err }
    }
}

impl From<SetterNamespaceError> for Error {
    fn from(err: SetterNamespaceError) -> Self {
        Error::SetterNamespace { err }
    }
}

impl From<JSONError> for Error {
    fn from(err: JSONError) -> Self {
        Error::ParseError { err }
    }
}

impl From<SerdeJSONError> for Error {
    fn from(err: SerdeJSONError) -> Self {
        Error::ValueParseError { err }
    }
}
