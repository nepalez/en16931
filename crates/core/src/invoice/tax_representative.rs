use crate::{NonEmptyString, PostalAddress, VatIdentifier};

/// The seller's tax representative (`BG-11`): a party that represents the seller for VAT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxRepresentative {
    /// Tax representative name (`BT-62`).
    pub name: NonEmptyString,
    /// Tax representative VAT identifier (`BT-63`).
    pub vat: VatIdentifier,
    /// Tax representative postal address (`BG-12`).
    pub address: PostalAddress,
}
