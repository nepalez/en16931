//! The payment card information (`BG-18`).

use crate::NonEmptyString;

/// The card a payment is made with (`BG-18`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaymentCard {
    /// Payment card primary account number (`BT-87`), usually masked to the last digits.
    pub primary_account_number: Option<NonEmptyString>,
    /// Payment cardholder name (`BT-88`).
    pub holder_name: Option<NonEmptyString>,
}
