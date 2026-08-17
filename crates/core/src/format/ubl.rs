mod deserialize;
mod serialize;

use crate::Namespace;
pub(crate) use deserialize::deserialize;
pub(crate) use serialize::serialize;

// The UBL namespaces, shared by both halves.
const INV: Namespace = Namespace::Invoice;
const CAC: Namespace = Namespace::CommonAggregateComponents;
const CBC: Namespace = Namespace::CommonBasicComponents;

// The XML prefix a UBL document binds to a record-form namespace. The root
// carries the default namespace, so it needs no prefix.
fn prefix(namespace: Namespace) -> &'static str {
    match namespace {
        CAC => "cac",
        CBC => "cbc",
        INV => "",
        _other => unreachable!("a UBL document never carries the {_other} namespace"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::test_helpers::{builder, path, pretty, step, variant_builder};
    use crate::{Binding, Context, Profile, Segment};
    use std::num::NonZeroUsize;

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
    fn detects_its_own_output_as_ubl() {
        let (xml, _) = serialize(&builder(Binding::Ubl));

        assert_eq!(Binding::detect(&xml).expect("a UBL document"), Binding::Ubl);
    }

    #[test]
    fn rebuilds_the_same_dictionary_on_parse() {
        let (xml, written) = serialize(&builder(Binding::Ubl));

        let (_, read) = deserialize(&xml).expect("a valid UBL document");

        assert_eq!(read, written);
    }

    #[test]
    fn emit_fixtures() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/src/format/ubl/fixtures");
        let (rich, _) = serialize(&builder(Binding::Ubl));
        let (variant, _) = serialize(&variant_builder(Binding::Ubl));
        std::fs::write(format!("{base}/1.xml"), pretty(&rich)).expect("write");
        std::fs::write(format!("{base}/2.xml"), pretty(&variant)).expect("write");
    }

    #[test]
    fn serializes_the_document_to_ubl() {
        let (xml, _) = serialize(&builder(Binding::Ubl));

        assert_eq!(pretty(&xml), include_str!("ubl/fixtures/1.xml"));
    }

    #[test]
    fn deserializes_the_document_from_ubl() {
        let (parsed, _) =
            deserialize(include_str!("ubl/fixtures/1.xml")).expect("a valid UBL document");

        assert_eq!(parsed, builder(Binding::Ubl));
    }

    #[test]
    fn serializes_the_variant_document_to_ubl() {
        let (xml, _) = serialize(&variant_builder(Binding::Ubl));

        assert_eq!(pretty(&xml), include_str!("ubl/fixtures/2.xml"));
    }

    #[test]
    fn deserializes_the_variant_document_from_ubl() {
        let (parsed, _) =
            deserialize(include_str!("ubl/fixtures/2.xml")).expect("a valid UBL document");

        assert_eq!(parsed, variant_builder(Binding::Ubl));
    }

    #[test]
    fn maps_nodes_to_their_contexts() {
        let (_, dictionary) = serialize(&builder(Binding::Ubl));

        let root = path(vec![step(INV, "Invoice", 1)]);
        let seller_name = path(vec![
            step(INV, "Invoice", 1),
            step(CAC, "AccountingSupplierParty", 1),
            step(CAC, "Party", 1),
            step(CAC, "PartyLegalEntity", 1),
            step(CBC, "RegistrationName", 1),
        ]);
        let second_line = path(vec![step(INV, "Invoice", 1), step(CAC, "InvoiceLine", 2)]);
        let payable = path(vec![
            step(INV, "Invoice", 1),
            step(CAC, "LegalMonetaryTotal", 1),
            step(CBC, "PayableAmount", 1),
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
        let (xml, _) = serialize(&document);

        assert!(xml.contains("<cbc:CustomizationID>urn:cen.eu:en16931:2017#compliant"));
        assert!(xml.contains("<cbc:Note>General note text</cbc:Note>"));
        assert!(!xml.contains("#AAB#"));
    }
}
