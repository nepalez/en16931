//! The credit transfer details (`BG-17`).

use crate::{AccountNumber, Bic, NonEmptyString};

/// An account a credit-transfer payment is made to (`BG-17`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreditTransfer {
    /// Payment account identifier (`BT-84`).
    pub account: Option<AccountNumber>,
    /// Payment account name (`BT-85`).
    pub account_name: Option<NonEmptyString>,
    /// Payment service provider identifier (`BT-86`).
    pub provider: Option<Bic>,
}
