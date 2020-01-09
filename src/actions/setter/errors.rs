use snafu::Snafu;

/// This type represents all possible errors that an occur while applying a transformation.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid destination type. {}", err))]
    InvalidDestinationType { err: String },
}
