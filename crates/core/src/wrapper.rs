use crate::{Entry, Error, Target};

/// Extract validator report findings from its service-specific envelope.
///
/// An extension crate implements the trait for the envelope of that service.
/// The implementation reads every finding the report carries
/// and leaves its address in the syntax the processor wrote.
///
/// The trait also covers the request side of the same service.
/// A service that names its rule set by an identifier supplies that identifier here.
/// A service that selects the rule set otherwise leaves the default in place.
pub trait Wrapper {
    type Error: Into<Error>;

    /// Turns a service-specific report into the findings it carries.
    fn unwrap(&self, report: &str) -> Result<Vec<Entry>, Self::Error>;

    /// The identifier of the rule set that checks such a document.
    ///
    /// The default suits a service that takes no identifier at all.
    fn vendor_id(&self, target: Target) -> Result<&'static str, Error> {
        Err(Error::UnsupportedTarget(target))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Binding, DocumentKind, Profile};

    // A wrapper of a service that selects its rule set outside the request body.
    struct Routed;

    impl Wrapper for Routed {
        type Error = Error;

        fn unwrap(&self, _report: &str) -> Result<Vec<Entry>, Self::Error> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn names_no_rule_set_by_default() {
        let target = Target {
            profile: Profile::En16931,
            binding: Binding::Ubl,
            kind: DocumentKind::CreditNote,
        };

        let outcome = Routed.vendor_id(target);

        assert!(matches!(outcome, Err(Error::UnsupportedTarget(_))));
    }
}
