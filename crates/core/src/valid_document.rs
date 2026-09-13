use crate::{Document, Format, Invoice, Problem, Report};

/// A document a validator accepted.
///
/// It pairs the checked document with the report of that pass.
/// The report holds no error, and it may still hold warnings and remarks.
///
/// A function that demands an accepted document takes this type.
/// Both conversions back, to the `Document` and to the `Invoice`, drop the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidDocument<I, F: Format> {
    // The document the pass accepted.
    pub(crate) document: Document<I, F>,
    // The problems of that pass, none of them an error.
    pub(crate) report: Report,
}

impl<I, F: Format> ValidDocument<I, F> {
    /// The report of the pass that accepted the document.
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The problems of the pass, none of them an error.
    pub fn problems(&self) -> &[Problem] {
        &self.report.problems
    }
}

impl<I, F: Format> From<ValidDocument<I, F>> for Document<I, F> {
    /// Recovers the checked document, dropping the report of the pass.
    fn from(checked: ValidDocument<I, F>) -> Self {
        checked.document
    }
}

impl<F: Format> From<ValidDocument<Invoice, F>> for Invoice {
    /// Recovers the business object, dropping the report of the pass.
    fn from(checked: ValidDocument<Invoice, F>) -> Self {
        checked.document.into()
    }
}
