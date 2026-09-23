mod deserialize;
mod serialize;

use super::{Format, Sealed, Token};
use crate::prelude::*;

pub use deserialize::deserialize;
pub use serialize::serialize;

/// The marker of the UN/CEFACT Cross Industry Invoice binding.
/// It carries the CII namespace set, reached as `<Cii as Format>::Namespace`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cii;

impl Sealed for Cii {}

impl Format for Cii {
    type Namespace = Namespace;

    const ROOT_ELEMENT: &'static str = "CrossIndustryInvoice";

    const BINDING: crate::Binding = crate::Binding::Cii;

    fn root_namespace() -> Namespace {
        Namespace::Rsm
    }
}

/// The record-form namespace set of the CII binding.
///
/// A member renders as the abbreviation of its namespace URI,
/// so a CII path reads as `/Q{RSM}CrossIndustryInvoice[1]/Q{RAM}…`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, VariantArray)]
pub enum Namespace {
    /// The namespace of the root `CrossIndustryInvoice` document
    /// and its top-level structural elements.
    #[display("RSM")]
    Rsm,
    /// The Reusable Aggregate Business Information Entity namespace,
    /// holding the business groups and fields.
    #[display("RAM")]
    Ram,
    /// The Unqualified Data Type namespace,
    /// holding the value carriers such as `udt:DateTimeString`.
    #[display("UDT")]
    Udt,
    /// The Qualified Data Type namespace, holding the formatted value carriers.
    #[display("QDT")]
    Qdt,
}

impl crate::Namespace for Namespace {
    fn uri(self) -> &'static str {
        match self {
            Self::Rsm => "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100",
            Self::Ram => {
                "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
            }
            Self::Udt => "urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100",
            Self::Qdt => "urn:un:unece:uncefact:data:standard:QualifiedDataType:100",
        }
    }

    fn prefix(self) -> &'static str {
        match self {
            Self::Rsm => "rsm",
            Self::Ram => "ram",
            Self::Udt => "udt",
            Self::Qdt => "qdt",
        }
    }

    fn abbreviation(self) -> &'static str {
        match self {
            Self::Rsm => "rsm",
            Self::Ram => "ram",
            Self::Udt => "udt",
            Self::Qdt => "qdt",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn names_its_root_namespace() {
        assert_eq!(Cii::root_namespace(), Namespace::Rsm);
    }

    #[test]
    fn abbreviates_each_namespace() {
        assert_eq!(Namespace::Rsm.to_string(), "RSM");
        assert_eq!(Namespace::Ram.to_string(), "RAM");
        assert_eq!(Namespace::Udt.to_string(), "UDT");
        assert_eq!(Namespace::Qdt.to_string(), "QDT");
    }

    #[test]
    fn resolves_an_abbreviation_of_a_rule_set() {
        let abbreviations = <Namespace as crate::Namespace>::default_abbreviations();

        assert_eq!(abbreviations.resolve("rsm"), Some(Namespace::Rsm));
        assert_eq!(abbreviations.resolve("ram"), Some(Namespace::Ram));
        assert_eq!(abbreviations.resolve("udt"), Some(Namespace::Udt));
        assert_eq!(abbreviations.resolve("qdt"), Some(Namespace::Qdt));
        assert_eq!(abbreviations.resolve("cac"), None);
    }
}
