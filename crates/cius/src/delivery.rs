//! The delivery information of the base invoice (`BG-13`).

use crate::prelude::*;

/// Where and when the goods or services are delivered (`BG-13`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

impl crate::prelude::Delivery for Delivery {
    fn name(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.name
    }

    fn location(&mut self) -> &mut Option<LocationReference> {
        &mut self.location
    }

    fn date(&mut self) -> &mut Option<Date> {
        &mut self.date
    }

    fn address(&mut self) -> &mut Option<PostalAddress> {
        &mut self.address
    }
}
