use crate::NonEmptyString;

/// Payment card information (`BG-18`): the card a payment is made with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentCard {
    /// Payment card primary account number (`BT-87`), usually masked to the last digits.
    pub primary_account_number: NonEmptyString,
    /// Payment cardholder name (`BT-88`).
    pub holder_name: Option<NonEmptyString>,
}
