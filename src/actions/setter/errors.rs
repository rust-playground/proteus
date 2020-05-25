use thiserror::Error;

/// This type represents all possible errors that an occur while applying a transformation.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid destination type. {0}")]
    InvalidDestinationType(String),
}
