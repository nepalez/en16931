use crate::{BusinessProcess, Profile};

/// The builder of the document to be sent.
///
/// It includes both the business terms (the corresponding `invoice`)
/// and the information needed for the document exchange only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentBuilder<I> {
    /// The business document with every fact it carries.
    pub invoice: I,
    /// Stamps the specification identifier and forbids terms (`BT-24`).
    pub profile: Profile,
    /// Business process type (`BT-23`).
    pub business_process: Option<BusinessProcess>,
}
