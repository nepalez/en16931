use crate::prelude::*;
use crate::{Date, NonEmptyString};

/// A preceding invoice reference (`BG-3`): an invoice that this one corrects or relates to.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct PrecedingInvoice {
    /// Preceding invoice number (`BT-25`).
    pub number: NonEmptyString,
    /// Preceding invoice issue date (`BT-26`).
    pub issue_date: Option<Date>,
}
