//! The interface of a contact point (`BG-6`/`BG-9`).

use crate::{EmailAddress, NonEmptyString};

/// A person or department to reach at a party (`BG-6`/`BG-9`).
pub trait Contact {
    /// Contact point name (`BT-41`).
    fn name(&mut self) -> &mut Option<NonEmptyString>;
    /// Contact telephone number (`BT-42`).
    fn telephone(&mut self) -> &mut Option<NonEmptyString>;
    /// Contact email address (`BT-43`).
    fn email(&mut self) -> &mut Option<EmailAddress>;
}
