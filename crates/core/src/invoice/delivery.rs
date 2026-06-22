use crate::{Date, LocationReference, NonEmptyString, PostalAddress};

/// Delivery information (`BG-13`): where and when the goods or services are delivered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    /// Deliver-to party name (`BT-70`).
    pub name: Option<NonEmptyString>,
    /// Deliver-to location identifier (`BT-71`).
    pub location: Option<LocationReference>,
    /// Actual delivery date (`BT-72`).
    pub date: Option<Date>,
    /// Deliver-to address (`BG-15`).
    pub address: Option<PostalAddress>,
}
