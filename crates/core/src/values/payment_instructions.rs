//! The payment instructions (`BG-16`).

use crate::{NonEmptyString, PaymentDetails, PaymentMeans};

/// How the invoice is to be paid (`BG-16`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaymentInstructions {
    /// Payment means type code (`BT-81`).
    pub means: Option<PaymentMeans>,
    /// Payment means text (`BT-82`).
    pub means_text: Option<NonEmptyString>,
    /// Remittance information (`BT-83`).
    pub remittance_information: Option<NonEmptyString>,
    /// At most one kind of payment details, keyed by the payment means (`BG-17`/`BG-18`/`BG-19`).
    pub details: Option<PaymentDetails>,
}
