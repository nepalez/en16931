//! The closed choice of the means-specific payment details (`BG-17`/`BG-18`/`BG-19`).

use crate::{CreditTransfer, DirectDebit, PaymentCard};

/// Exactly one kind of payment details, chosen by the payment means code (`BT-81`)
/// (`BG-17`/`BG-18`/`BG-19`).
///
/// The choice of kinds is closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentDetails {
    /// Credit transfer details (`BG-17`).
    CreditTransfers(Vec<CreditTransfer>),
    /// Payment card information (`BG-18`).
    Card(PaymentCard),
    /// Direct debit details (`BG-19`).
    DirectDebit(DirectDebit),
}
