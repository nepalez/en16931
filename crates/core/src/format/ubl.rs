mod deserialize;
mod serialize;

use crate::format::Sealed;
use crate::prelude::*;
use crate::{Abbreviations, Format};
pub(crate) use deserialize::deserialize;
pub(crate) use serialize::serialize;

/// The marker of the OASIS Universal Business Language binding.
/// It carries the UBL namespace set, reached as `<Ubl as Format>::Namespace`.
pub struct Ubl;

impl Sealed for Ubl {}

impl Format for Ubl {
    type Namespace = Namespace;

    fn root_namespace() -> Namespace {
        Namespace::Inv
    }
}

/// The record-form namespace set of the UBL binding.
///
/// A member renders as the abbreviation of its namespace URI,
/// so a UBL path reads as `/Q{INV}Invoice[1]/Q{CAC}InvoiceLine[2]/Q{CBC}ID[1]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
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

    fn from_uri(uri: &str) -> Option<Self> {
        [Self::Inv, Self::Cac, Self::Cbc]
            .into_iter()
            .find(|member| <Self as crate::Namespace>::uri(*member) == uri)
    }

    fn default_abbreviations() -> Abbreviations<Self> {
        [("ubl", Self::Inv), ("cac", Self::Cac), ("cbc", Self::Cbc)]
            .into_iter()
            .collect()
    }
}

// The XML prefix a UBL document binds to a record-form namespace. The root
// carries the default namespace, so it needs no prefix.
fn prefix(namespace: Namespace) -> &'static str {
    match namespace {
        Namespace::Cac => "cac",
        Namespace::Cbc => "cbc",
        Namespace::Inv => "",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::test_helpers::{builder, card_builder, path, pretty, step, variant_builder};
    use crate::{Binding, Context, Error, Profile, Segment};

    // A context of the given model segments.
    fn context(segments: Vec<Segment>) -> Context {
        Context { segments }
    }

    // A single non-indexed field segment.
    fn field(name: &'static str) -> Segment {
        Segment {
            field: name,
            index: None,
        }
    }

    // A repeatable-group instance segment.
    fn instance(name: &'static str, index: usize) -> Segment {
        Segment {
            field: name,
            index: NonZeroUsize::new(index),
        }
    }

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

    #[test]
    fn detects_its_own_output_as_ubl() {
        let (xml, _, _) = serialize(&builder(Binding::Ubl));

        assert_eq!(Binding::detect(&xml).expect("a UBL document"), Binding::Ubl);
    }

    #[test]
    fn rebuilds_the_same_dictionary_on_parse() {
        let (xml, written, _) = serialize(&builder(Binding::Ubl));

        let (_, read, _) = deserialize(&xml).expect("a valid UBL document");

        assert_eq!(read, written);
    }

    #[test]
    fn binds_the_abbreviations_it_writes() {
        let (_, _, abbreviations) = serialize(&builder(Binding::Ubl));

        // The root carries the default namespace, so its abbreviation is empty.
        assert_eq!(abbreviations.resolve(""), Some(Namespace::Inv));
        assert_eq!(abbreviations.resolve("cac"), Some(Namespace::Cac));
        assert_eq!(abbreviations.resolve("cbc"), Some(Namespace::Cbc));
    }

    #[test]
    fn rebuilds_the_same_abbreviations_on_parse() {
        let (xml, _, written) = serialize(&builder(Binding::Ubl));

        let (_, _, read) = deserialize(&xml).expect("a valid UBL document");

        assert_eq!(read, written);
    }

    #[test]
    fn binds_an_abbreviation_of_its_own_choice() {
        // The fixture is the rich document with `cbc` renamed to `foo`.
        let (_, _, abbreviations) =
            deserialize(include_str!("ubl/fixtures/3.xml")).expect("a valid UBL document");

        // The document's own abbreviation joins the ones the rule sets bind.
        assert_eq!(abbreviations.resolve("foo"), Some(Namespace::Cbc));
        assert_eq!(abbreviations.resolve("cbc"), Some(Namespace::Cbc));
    }

    #[test]
    fn emit_fixtures() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/src/format/ubl/fixtures");
        let (rich, _, _) = serialize(&builder(Binding::Ubl));
        let (variant, _, _) = serialize(&variant_builder(Binding::Ubl));
        std::fs::write(format!("{base}/1.xml"), pretty(&rich)).expect("write");
        std::fs::write(format!("{base}/2.xml"), pretty(&variant)).expect("write");
    }

    #[test]
    fn serializes_the_document_to_ubl() {
        let (xml, _, _) = serialize(&builder(Binding::Ubl));

        assert_eq!(pretty(&xml), include_str!("ubl/fixtures/1.xml"));
    }

    #[test]
    fn deserializes_the_document_from_ubl() {
        let (parsed, _, _) =
            deserialize(include_str!("ubl/fixtures/1.xml")).expect("a valid UBL document");

        assert_eq!(parsed, builder(Binding::Ubl));
    }

    #[test]
    fn serializes_the_variant_document_to_ubl() {
        let (xml, _, _) = serialize(&variant_builder(Binding::Ubl));

        assert_eq!(pretty(&xml), include_str!("ubl/fixtures/2.xml"));
    }

    #[test]
    fn deserializes_the_variant_document_from_ubl() {
        let (parsed, _, _) =
            deserialize(include_str!("ubl/fixtures/2.xml")).expect("a valid UBL document");

        assert_eq!(parsed, variant_builder(Binding::Ubl));
    }

    #[test]
    fn round_trips_the_card_document_through_ubl() {
        let source = card_builder(Binding::Ubl);
        let (xml, _, _) = serialize(&source);

        let (parsed, _, _) = deserialize(&xml).expect("a valid UBL document");

        assert_eq!(parsed, source);
    }

    #[test]
    fn maps_nodes_to_their_contexts() {
        let (_, dictionary, _) = serialize(&builder(Binding::Ubl));

        let root = path(vec![step(Namespace::Inv, "Invoice", 1)]);
        let seller_name = path(vec![
            step(Namespace::Inv, "Invoice", 1),
            step(Namespace::Cac, "AccountingSupplierParty", 1),
            step(Namespace::Cac, "Party", 1),
            step(Namespace::Cac, "PartyLegalEntity", 1),
            step(Namespace::Cbc, "RegistrationName", 1),
        ]);
        let second_line = path(vec![
            step(Namespace::Inv, "Invoice", 1),
            step(Namespace::Cac, "InvoiceLine", 2),
        ]);
        let payable = path(vec![
            step(Namespace::Inv, "Invoice", 1),
            step(Namespace::Cac, "LegalMonetaryTotal", 1),
            step(Namespace::Cbc, "PayableAmount", 1),
        ]);

        // The root, a term-less node, resolves to the root context.
        assert_eq!(dictionary.get(&root), Some(&context(Vec::new())));
        // The registration name nests its field under the seller group.
        assert_eq!(
            dictionary.get(&seller_name),
            Some(&context(vec![field("seller"), field("name")]))
        );
        // The second line resolves to its instance, carrying index 2.
        assert_eq!(
            dictionary.get(&second_line),
            Some(&context(vec![instance("lines", 2)]))
        );
        // A derived total maps to the root, like every term-less node.
        assert_eq!(dictionary.get(&payable), Some(&context(Vec::new())));
    }

    #[test]
    fn drops_a_term_the_profile_forbids() {
        // Peppol BIS forbids the note subject code (BT-21).
        let mut document = builder(Binding::Ubl);
        document.profile = Profile::PeppolBisBilling30;
        let (xml, _, _) = serialize(&document);

        assert!(xml.contains("<cbc:CustomizationID>urn:cen.eu:en16931:2017#compliant"));
        assert!(xml.contains("<cbc:Note>General note text</cbc:Note>"));
        assert!(!xml.contains("#AAB#"));
    }

    #[test]
    fn rejects_an_element_of_an_unknown_namespace() {
        let xml = r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2" xmlns:foo="urn:example:unknown"><foo:Bar/></Invoice>"#;

        let outcome = deserialize(xml);

        assert!(matches!(
            outcome,
            Err(Error::MalformedXml { message, .. }) if message.starts_with("unknown namespace")
        ));
    }
}
