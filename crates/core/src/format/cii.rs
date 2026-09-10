mod deserialize;
mod serialize;

use crate::format::Sealed;
use crate::prelude::*;
use crate::{Abbreviations, Format};
pub(crate) use deserialize::deserialize;
pub(crate) use serialize::serialize;

/// The marker of the UN/CEFACT Cross Industry Invoice binding.
/// It carries the CII namespace set, reached as `<Cii as Format>::Namespace`.
pub struct Cii;

impl Sealed for Cii {}

impl Format for Cii {
    type Namespace = Namespace;

    fn root_namespace() -> Namespace {
        Namespace::Rsm
    }
}

/// The record-form namespace set of the CII binding.
///
/// A member renders as the abbreviation of its namespace URI,
/// so a CII path reads as `/Q{RSM}CrossIndustryInvoice[1]/Q{RAM}…`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
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

    fn from_uri(uri: &str) -> Option<Self> {
        [Self::Rsm, Self::Ram, Self::Udt, Self::Qdt]
            .into_iter()
            .find(|member| <Self as crate::Namespace>::uri(*member) == uri)
    }

    fn default_abbreviations() -> Abbreviations<Self> {
        [
            ("rsm", Self::Rsm),
            ("ram", Self::Ram),
            ("udt", Self::Udt),
            ("qdt", Self::Qdt),
        ]
        .into_iter()
        .collect()
    }
}

// The XML prefix a CII document binds to a record-form namespace.
fn prefix(namespace: Namespace) -> &'static str {
    match namespace {
        Namespace::Rsm => "rsm",
        Namespace::Ram => "ram",
        Namespace::Udt => "udt",
        Namespace::Qdt => "qdt",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::test_helpers::{builder, card_builder, pretty, variant_builder};
    use crate::{Binding, Error};

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

    #[test]
    fn detects_its_own_output_as_cii() {
        let (xml, _, _) = serialize(&builder(Binding::Cii));

        assert_eq!(Binding::detect(&xml).expect("a CII document"), Binding::Cii);
    }

    #[test]
    fn rebuilds_the_same_dictionary_on_parse() {
        let (xml, written, _) = serialize(&builder(Binding::Cii));

        let (_, read, _) = deserialize(&xml).expect("a valid CII document");

        assert_eq!(read, written);
    }

    #[test]
    fn binds_the_abbreviations_it_writes() {
        let (_, _, abbreviations) = serialize(&builder(Binding::Cii));

        assert_eq!(abbreviations.resolve("rsm"), Some(Namespace::Rsm));
        assert_eq!(abbreviations.resolve("ram"), Some(Namespace::Ram));
        assert_eq!(abbreviations.resolve("udt"), Some(Namespace::Udt));
        assert_eq!(abbreviations.resolve("qdt"), Some(Namespace::Qdt));
    }

    #[test]
    fn rebuilds_the_same_abbreviations_on_parse() {
        let (xml, _, written) = serialize(&builder(Binding::Cii));

        let (_, _, read) = deserialize(&xml).expect("a valid CII document");

        assert_eq!(read, written);
    }

    #[test]
    fn binds_an_abbreviation_of_its_own_choice() {
        // The fixture is the rich document with `ram` renamed to `bar`.
        let (_, _, abbreviations) =
            deserialize(include_str!("cii/fixtures/3.xml")).expect("a valid CII document");

        // The document's own abbreviation joins the ones the rule sets bind.
        assert_eq!(abbreviations.resolve("bar"), Some(Namespace::Ram));
        assert_eq!(abbreviations.resolve("ram"), Some(Namespace::Ram));
    }

    #[test]
    fn emit_fixtures() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/src/format/cii/fixtures");
        let (rich, _, _) = serialize(&builder(Binding::Cii));
        let (variant, _, _) = serialize(&variant_builder(Binding::Cii));
        std::fs::write(format!("{base}/1.xml"), pretty(&rich)).expect("write");
        std::fs::write(format!("{base}/2.xml"), pretty(&variant)).expect("write");
    }

    #[test]
    fn serializes_the_document_to_cii() {
        let (xml, _, _) = serialize(&builder(Binding::Cii));

        assert_eq!(pretty(&xml), include_str!("cii/fixtures/1.xml"));
    }

    #[test]
    fn deserializes_the_document_from_cii() {
        let (parsed, _, _) =
            deserialize(include_str!("cii/fixtures/1.xml")).expect("a valid CII document");

        assert_eq!(parsed, builder(Binding::Cii));
    }

    #[test]
    fn serializes_the_variant_document_to_cii() {
        let (xml, _, _) = serialize(&variant_builder(Binding::Cii));

        assert_eq!(pretty(&xml), include_str!("cii/fixtures/2.xml"));
    }

    #[test]
    fn deserializes_the_variant_document_from_cii() {
        let (parsed, _, _) =
            deserialize(include_str!("cii/fixtures/2.xml")).expect("a valid CII document");

        assert_eq!(parsed, variant_builder(Binding::Cii));
    }

    #[test]
    fn round_trips_the_card_document_through_cii() {
        let source = card_builder(Binding::Cii);
        let (xml, _, _) = serialize(&source);

        let (parsed, _, _) = deserialize(&xml).expect("a valid CII document");

        assert_eq!(parsed, source);
    }

    #[test]
    fn rejects_an_element_of_an_unknown_namespace() {
        let xml = r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:foo="urn:example:unknown"><foo:Bar/></rsm:CrossIndustryInvoice>"#;

        let outcome = deserialize(xml);

        assert!(matches!(
            outcome,
            Err(Error::MalformedXml { message, .. }) if message.starts_with("unknown namespace")
        ));
    }
}
