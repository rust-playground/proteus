//! Errors that can occur applying transformations.

use crate::actions::setter::Error as SetterError;
use serde_json::Error as SerdeJSONError;
use snafu::Snafu;

/// This type represents all possible errors that an occur while building and applying a Transformation.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", err))]
    Setter { err: SetterError },

    #[snafu(display("{}", err))]
    JSONError { err: SerdeJSONError },
}

impl From<SerdeJSONError> for Error {
    fn from(err: SerdeJSONError) -> Self {
        Error::JSONError { err }
    }
}

impl From<SetterError> for Error {
    fn from(err: SetterError) -> Self {
        Error::Setter { err }
    }
}
