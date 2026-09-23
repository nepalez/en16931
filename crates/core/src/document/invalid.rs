use crate::{Document, Format, Problem, Report};

/// A document a validator rejected.
///
/// It pairs the checked document with the report of that pass,
/// which holds at least one error, and possibly warnings and remarks as well.
///
/// A function that demands a rejected document takes this type.
/// The conversion back to the `Document` and `into_invoice` both drop the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidDocument<I, F: Format> {
    // The document the pass rejected.
    pub(crate) document: Document<I, F>,
    // The problems of that pass, at least one of them an error.
    pub(crate) report: Report,
}

impl<I, F: Format> InvalidDocument<I, F> {
    /// The report of the pass that rejected the document.
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The problems of the pass, at least one of them an error.
    pub fn problems(&self) -> &[Problem] {
        &self.report.problems
    }

    /// Recovers the business object, dropping the report of the pass.
    pub fn into_invoice(self) -> I {
        self.document.into_invoice()
    }
}

impl<I, F: Format> From<InvalidDocument<I, F>> for Document<I, F> {
    /// Recovers the checked document, dropping the report of the pass.
    fn from(checked: InvalidDocument<I, F>) -> Self {
        checked.document
    }
}
