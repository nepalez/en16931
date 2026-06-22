use crate::{EmailAddress, NonEmptyString};

/// A contact point (`BG-6`/`BG-9`): a person or department to reach at a party.
///
/// Every field is optional.
/// The group exists only to carry whichever contact details are known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    /// Contact point name (`BT-41`).
    pub name: Option<NonEmptyString>,
    /// Contact telephone number (`BT-42`).
    pub telephone: Option<NonEmptyString>,
    /// Contact email address (`BT-43`).
    pub email: Option<EmailAddress>,
}
