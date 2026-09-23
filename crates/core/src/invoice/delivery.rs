//! The interface of the delivery information (`BG-13`).

use crate::{Date, LocationReference, NonEmptyString, PostalAddress};

/// Where and when the goods or services are delivered (`BG-13`).
pub trait Delivery {
    /// Deliver-to party name (`BT-70`).
    fn name(&mut self) -> &mut Option<NonEmptyString>;
    /// Deliver-to location identifier (`BT-71`).
    fn location(&mut self) -> &mut Option<LocationReference>;
    /// Actual delivery date (`BT-72`).
    fn date(&mut self) -> &mut Option<Date>;
    /// Deliver-to address (`BG-15`).
    fn address(&mut self) -> &mut Option<PostalAddress>;
}
