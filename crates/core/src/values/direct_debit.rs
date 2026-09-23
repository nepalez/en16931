//! The direct debit details (`BG-19`).

use crate::{AccountNumber, NonEmptyString};

/// The mandate under which a direct-debit payment is collected (`BG-19`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectDebit {
    /// Mandate reference identifier (`BT-89`).
    pub mandate_reference: Option<NonEmptyString>,
    /// Bank assigned creditor identifier (`BT-90`).
    pub creditor_identifier: Option<NonEmptyString>,
    /// Debited account identifier (`BT-91`).
    pub debited_account: Option<AccountNumber>,
}
