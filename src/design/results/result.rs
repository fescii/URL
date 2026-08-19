use crate::design::errors::Error;

/// Centrally managed Result alias for the project.
pub type Result<T, E = Error> = std::result::Result<T, E>;
