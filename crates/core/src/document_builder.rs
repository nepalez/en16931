use crate::{Binding, BusinessProcess, Invoice, Profile};

/// The builder of the document to be sent.
///
/// It includes both the business terms (the corresponding `invoice`)
/// and the information needed for the document exchange only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentBuilder {
    /// The business document with every fact it carries.
    pub invoice: Invoice,
    /// The target profile (`BT-24`): stamps the specification identifier and forbids terms.
    pub profile: Profile,
    /// The target XML binding to serialize into.
    pub binding: Binding,
    /// Business process type (`BT-23`).
    pub business_process: Option<BusinessProcess>,
}
