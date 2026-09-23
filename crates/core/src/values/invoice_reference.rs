//! A preceding invoice reference (`BG-3`).

use crate::{Date, NonEmptyString};

/// An invoice that this one corrects or relates to (`BG-3`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvoiceReference {
    /// Preceding invoice number (`BT-25`).
    pub number: Option<NonEmptyString>,
    /// Preceding invoice issue date (`BT-26`).
    pub issue_date: Option<Date>,
}
