//! Public interface of all the possible errors.

use crate::signal::AddEdgeError;

impl From<AddEdgeError> for Error {
    fn from(error: AddEdgeError) -> Self {
        Self::AddEdgeError(error)
    }
}

/// Convenience enumeration of all the errors that could be returned from
/// libraries' public interfaces.
#[derive(Debug)]
pub enum Error {
    AddEdgeError(AddEdgeError),
}
