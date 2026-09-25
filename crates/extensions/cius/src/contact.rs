//! The contact point of the base invoice (`BG-6`/`BG-9`).

use crate::prelude::*;

/// A person or department to reach at a party (`BG-6`/`BG-9`).
///
/// Every field is optional.
/// The group exists only to carry whichever contact details are known.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contact {
    /// Contact point name (`BT-41`).
    pub name: Option<NonEmptyString>,
    /// Contact telephone number (`BT-42`).
    pub telephone: Option<NonEmptyString>,
    /// Contact email address (`BT-43`).
    pub email: Option<EmailAddress>,
}

impl crate::prelude::Contact for Contact {
    fn name(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.name
    }

    fn telephone(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.telephone
    }

    fn email(&mut self) -> &mut Option<EmailAddress> {
        &mut self.email
    }
}
