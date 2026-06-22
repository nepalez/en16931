use crate::{Classification, CountryCode, ItemAttribute, ItemReference, NonEmptyString};

/// Item information (`BG-31`): what is invoiced on a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Name (`BT-153`).
    pub name: NonEmptyString,
    /// Description (`BT-154`).
    pub description: Option<NonEmptyString>,
    /// Seller's item identifier (`BT-155`).
    pub seller_id: Option<NonEmptyString>,
    /// Buyer's item identifier (`BT-156`).
    pub buyer_id: Option<NonEmptyString>,
    /// Standard identifier (`BT-157`).
    pub standard_id: Option<ItemReference>,
    /// Classifications (`BT-158`).
    pub classifications: Vec<Classification>,
    /// Country of origin (`BT-159`).
    pub country_of_origin: Option<CountryCode>,
    /// Attributes (`BG-32`).
    pub attributes: Vec<ItemAttribute>,
}
