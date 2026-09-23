mod deserialize;
mod serialize;

use super::{Format, Token};
use crate::format::Sealed;
use crate::prelude::*;

pub use deserialize::deserialize;
pub use serialize::serialize;

/// The marker of the OASIS Universal Business Language binding.
/// It carries the UBL namespace set, reached as `<Ubl as Format>::Namespace`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ubl;

impl Sealed for Ubl {}

impl Format for Ubl {
    type Namespace = Namespace;

    const ROOT_ELEMENT: &'static str = "Invoice";

    const BINDING: crate::Binding = crate::Binding::Ubl;

    fn root_namespace() -> Namespace {
        Namespace::Inv
    }
}

/// The record-form namespace set of the UBL binding.
///
/// A member renders as the abbreviation of its namespace URI,
/// so a UBL path reads as `/Q{INV}Invoice[1]/Q{CAC}InvoiceLine[2]/Q{CBC}ID[1]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, VariantArray)]
pub enum Namespace {
    /// The namespace of the root `Invoice` document.
    #[display("INV")]
    Inv,
    /// The Common Aggregate Components namespace, holding the nested business groups.
    #[display("CAC")]
    Cac,
    /// The Common Basic Components namespace, holding the leaf fields.
    #[display("CBC")]
    Cbc,
}

impl crate::Namespace for Namespace {
    fn uri(self) -> &'static str {
        match self {
            Self::Inv => "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2",
            Self::Cac => "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2",
            Self::Cbc => "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2",
        }
    }

    // The root carries the default namespace, so it needs no prefix.
    fn prefix(self) -> &'static str {
        match self {
            Self::Inv => "",
            Self::Cac => "cac",
            Self::Cbc => "cbc",
        }
    }

    fn abbreviation(self) -> &'static str {
        match self {
            Self::Inv => "ubl",
            Self::Cac => "cac",
            Self::Cbc => "cbc",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn names_its_root_namespace() {
        assert_eq!(Ubl::root_namespace(), Namespace::Inv);
    }

    #[test]
    fn abbreviates_each_namespace() {
        assert_eq!(Namespace::Inv.to_string(), "INV");
        assert_eq!(Namespace::Cac.to_string(), "CAC");
        assert_eq!(Namespace::Cbc.to_string(), "CBC");
    }

    #[test]
    fn resolves_an_abbreviation_of_a_rule_set() {
        let abbreviations = <Namespace as crate::Namespace>::default_abbreviations();

        assert_eq!(abbreviations.resolve("ubl"), Some(Namespace::Inv));
        assert_eq!(abbreviations.resolve("cac"), Some(Namespace::Cac));
        assert_eq!(abbreviations.resolve("cbc"), Some(Namespace::Cbc));
        assert_eq!(abbreviations.resolve("rsm"), None);
    }
}
