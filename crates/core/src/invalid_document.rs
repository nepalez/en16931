use crate::{Document, Invoice, Report};

/// A document a validator rejected.
///
/// It pairs the checked document with the report of that pass,
/// which holds at least one error, and possibly warnings and remarks as well.
///
/// A function that demands a rejected document takes this type.
/// Both conversions back, to the `Document` and to the `Invoice`, drop the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidDocument {
    // The document the pass rejected.
    pub(crate) document: Document,
    // The problems of that pass, at least one of them an error.
    pub(crate) report: Report,
}

impl InvalidDocument {
    /// The report of the pass that rejected the document.
    pub fn report(&self) -> &Report {
        &self.report
    }
}

impl From<InvalidDocument> for Document {
    /// Recovers the checked document, dropping the report of the pass.
    fn from(checked: InvalidDocument) -> Self {
        checked.document
    }
}

impl From<InvalidDocument> for Invoice {
    /// Recovers the business object, dropping the report of the pass.
    fn from(checked: InvalidDocument) -> Self {
        checked.document.into()
    }
}
