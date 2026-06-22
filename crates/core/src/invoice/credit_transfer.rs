use crate::{AccountNumber, Bic, NonEmptyString};

/// Credit transfer details (`BG-17`): an account a credit-transfer payment is made to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditTransfer {
    /// Payment account identifier (`BT-84`).
    pub account: AccountNumber,
    /// Payment account name (`BT-85`).
    pub account_name: Option<NonEmptyString>,
    /// Payment service provider identifier (`BT-86`).
    pub provider: Option<Bic>,
}
