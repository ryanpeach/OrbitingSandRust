//! Bevy system errors
//!

use thiserror::Error;

/// Returned from a system that is expecting an object to have a parent
#[derive(Error, Debug)]
#[error("{entity_type} needs a {necessary_parent_type} parent: {err:?}")]
pub struct MissingParentError {
    /// The error you are wrapping, if any
    pub err: Option<Box<dyn std::error::Error>>,
    /// The child
    pub entity_type: String,
    /// The parent
    pub necessary_parent_type: String,
}

/// Tells you that the query did not return any desired results
#[derive(Error, Debug)]
#[error("Empty Query Result for {parameter_name}: {err:?}")]
pub struct EmptyQueryResult {
    /// The error you are wrapping, if any
    pub err: Option<Box<dyn std::error::Error>>,
    /// Use this to put a name of a variable you were trying to calculate
    pub parameter_name: String,
}
