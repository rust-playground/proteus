//! Errors that can occur applying transformations.

use crate::actions::setter::Error as SetterError;
use thiserror::Error;

/// This type represents all possible errors that an occur while building and applying a Transformation.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Setter(#[from] SetterError),

    #[error(transparent)]
    JSONError(#[from] serde_json::Error),
}
