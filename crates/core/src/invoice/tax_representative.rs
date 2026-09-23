//! The interface of the seller's tax representative (`BG-11`).

use crate::{NonEmptyString, PostalAddress, VatIdentifier};

/// A party that represents the seller for VAT (`BG-11`).
pub trait TaxRepresentative {
    /// Tax representative name (`BT-62`).
    fn name(&mut self) -> &mut Option<NonEmptyString>;
    /// Tax representative VAT identifier (`BT-63`).
    fn vat(&mut self) -> &mut Option<VatIdentifier>;
    /// Tax representative postal address (`BG-12`).
    fn address(&mut self) -> &mut Option<PostalAddress>;
}
