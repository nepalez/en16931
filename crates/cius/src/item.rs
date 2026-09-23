//! The item information of the base invoice (`BG-31`).

use crate::prelude::*;

/// What is invoiced on a line (`BG-31`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    /// Name (`BT-153`).
    pub name: Option<NonEmptyString>,
    /// Description (`BT-154`).
    pub description: Option<NonEmptyString>,
    /// Seller's item identifier (`BT-155`).
    pub seller_id: Option<NonEmptyString>,
    /// Buyer's item identifier (`BT-156`).
    pub buyer_id: Option<NonEmptyString>,
    /// Standard identifier (`BT-157`).
    pub standard_id: Option<ItemReference>,
    /// Classifications (`BT-158`).
    pub classifications: Vec<ItemClassification>,
    /// Country of origin (`BT-159`).
    pub country_of_origin: Option<CountryCode>,
    /// Attributes (`BG-32`).
    pub attributes: Vec<ItemAttribute>,
}

impl crate::prelude::Item for Item {
    fn name(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.name
    }

    fn description(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.description
    }

    fn seller_id(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.seller_id
    }

    fn buyer_id(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.buyer_id
    }

    fn standard_id(&mut self) -> &mut Option<ItemReference> {
        &mut self.standard_id
    }

    fn classifications(&mut self) -> &mut Vec<ItemClassification> {
        &mut self.classifications
    }

    fn country_of_origin(&mut self) -> &mut Option<CountryCode> {
        &mut self.country_of_origin
    }

    fn attributes(&mut self) -> &mut Vec<ItemAttribute> {
        &mut self.attributes
    }
}
