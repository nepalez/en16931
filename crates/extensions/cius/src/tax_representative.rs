//! The seller's tax representative of the base invoice (`BG-11`).

use crate::prelude::*;

/// A party that represents the seller for VAT (`BG-11`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaxRepresentative {
    /// Tax representative name (`BT-62`).
    pub name: Option<NonEmptyString>,
    /// Tax representative VAT identifier (`BT-63`).
    pub vat: Option<VatIdentifier>,
    /// Tax representative postal address (`BG-12`).
    pub address: Option<PostalAddress>,
}

impl crate::prelude::TaxRepresentative for TaxRepresentative {
    fn name(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.name
    }

    fn vat(&mut self) -> &mut Option<VatIdentifier> {
        &mut self.vat
    }

    fn address(&mut self) -> &mut Option<PostalAddress> {
        &mut self.address
    }
}
