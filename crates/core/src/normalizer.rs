use crate::{Error, Location};

/// Convert the validator-specific error references into the normalized form.
///
/// An extension crate implements the trait for the dialect of that processor.
/// The implementation strips the processor-specific syntax of error references
/// and converts them into the standard representation.
pub trait Normalizer {
    type Error: Into<Error>;

    /// Turns a validator-specific location into its normalized form.
    fn normalize(&self, location: &str) -> Result<Location, Self::Error>;
}
