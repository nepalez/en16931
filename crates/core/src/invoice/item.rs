//! The interface of the item information (`BG-31`).

use crate::{CountryCode, ItemAttribute, ItemClassification, ItemReference, NonEmptyString};

/// What is invoiced on a line (`BG-31`).
pub trait Item {
    /// Name (`BT-153`).
    fn name(&mut self) -> &mut Option<NonEmptyString>;
    /// Description (`BT-154`).
    fn description(&mut self) -> &mut Option<NonEmptyString>;
    /// Seller's item identifier (`BT-155`).
    fn seller_id(&mut self) -> &mut Option<NonEmptyString>;
    /// Buyer's item identifier (`BT-156`).
    fn buyer_id(&mut self) -> &mut Option<NonEmptyString>;
    /// Standard identifier (`BT-157`).
    fn standard_id(&mut self) -> &mut Option<ItemReference>;
    /// Classifications (`BT-158`).
    fn classifications(&mut self) -> &mut Vec<ItemClassification>;
    /// Country of origin (`BT-159`).
    fn country_of_origin(&mut self) -> &mut Option<CountryCode>;
    /// Attributes (`BG-32`).
    fn attributes(&mut self) -> &mut Vec<ItemAttribute>;
}
