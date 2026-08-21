use crate::{Document, Invoice, Report};

/// A document a validator accepted.
///
/// It pairs the checked document with the report of that pass.
/// The report holds no error, and it may still hold warnings and remarks.
///
/// A function that demands an accepted document takes this type.
/// Both conversions back, to the `Document` and to the `Invoice`, drop the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidDocument {
    // The document the pass accepted.
    pub(crate) document: Document,
    // The problems of that pass, none of them an error.
    pub(crate) report: Report,
}

impl ValidDocument {
    /// The report of the pass that accepted the document.
    pub fn report(&self) -> &Report {
        &self.report
    }
}

impl From<ValidDocument> for Document {
    /// Recovers the checked document, dropping the report of the pass.
    fn from(checked: ValidDocument) -> Self {
        checked.document
    }
}

impl From<ValidDocument> for Invoice {
    /// Recovers the business object, dropping the report of the pass.
    fn from(checked: ValidDocument) -> Self {
        checked.document.into()
    }
}
