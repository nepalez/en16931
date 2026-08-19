use crate::{Entry, Error};

/// Extract validator report findings from its service-specific envelope.
///
/// An extension crate implements the trait for the envelope of that service.
/// The implementation reads every finding the report carries
/// and leaves its address in the syntax the processor wrote.
pub trait Wrapper {
    type Error: Into<Error>;

    /// Turns a service-specific report into the findings it carries.
    fn unwrap(&self, report: &str) -> Result<Vec<Entry>, Self::Error>;
}
