use crate::{CreditTransfer, DirectDebit, NonEmptyString, PaymentCard, PaymentMeans};

/// Payment instructions (`BG-16`): how the invoice is to be paid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentInstructions {
    /// Payment means type code (`BT-81`).
    pub means: PaymentMeans,
    /// Payment means text (`BT-82`).
    pub means_text: Option<NonEmptyString>,
    /// Remittance information (`BT-83`).
    pub remittance_information: Option<NonEmptyString>,
    /// Payment details (`BG-17`/`BG-18`/`BG-19`): at most one kind, keyed by the payment means.
    pub details: Option<Details>,
}

/// The means-specific payment details (`BG-17`/`BG-18`/`BG-19`):
/// exactly one applies, chosen by the payment means code (`BT-81`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Details {
    /// Credit transfer details (`BG-17`).
    CreditTransfers(Vec<CreditTransfer>),
    /// Payment card information (`BG-18`).
    Card(PaymentCard),
    /// Direct debit details (`BG-19`).
    DirectDebit(DirectDebit),
}
