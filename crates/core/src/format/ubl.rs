mod deserialize;
mod serialize;

use crate::Format;
use crate::format::Sealed;
use crate::prelude::*;

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
    use crate::format::test_helpers::{
        builder, card_builder, empty_builder, path, pretty, step, variant_builder,
    };
    use crate::{Binding, Context, Document, DocumentBuilder, Error, Invoice, Profile, Segment};

    fn written(builder: DocumentBuilder<Invoice>) -> Document<Invoice, Ubl> {
        Document::try_from(builder).expect("a serialized document")
    }

    fn read(xml: &str) -> Result<Document<Invoice, Ubl>, Error> {
        Document::parse(xml)
    }

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
        let document = written(builder());

        assert_eq!(
            Binding::detect(document.xml()).expect("a UBL document"),
            Binding::Ubl
        );
    }

    #[test]
    fn rebuilds_the_same_dictionary_on_parse() {
        let document = written(builder());

        let parsed = read(document.xml()).expect("a valid UBL document");

        assert_eq!(parsed.dictionary, document.dictionary);
    }

    #[test]
    fn binds_the_abbreviations_it_writes() {
        let abbreviations = written(builder()).abbreviations;

        // The root carries the default namespace, so its abbreviation is empty.
        assert_eq!(abbreviations.resolve(""), Some(Namespace::Inv));
        assert_eq!(abbreviations.resolve("cac"), Some(Namespace::Cac));
        assert_eq!(abbreviations.resolve("cbc"), Some(Namespace::Cbc));
    }

    #[test]
    fn rebuilds_the_same_abbreviations_on_parse() {
        let document = written(builder());

        let parsed = read(document.xml()).expect("a valid UBL document");

        assert_eq!(parsed.abbreviations, document.abbreviations);
    }

    #[test]
    fn binds_an_abbreviation_of_its_own_choice() {
        // The fixture is the rich document with `cbc` renamed to `foo`.
        let parsed = read(include_str!("ubl/fixtures/3.xml")).expect("a valid UBL document");

        // The document's own abbreviation joins the ones the rule sets bind.
        assert_eq!(parsed.abbreviations.resolve("foo"), Some(Namespace::Cbc));
        assert_eq!(parsed.abbreviations.resolve("cbc"), Some(Namespace::Cbc));
    }

    #[test]
    fn emit_fixtures() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/src/format/ubl/fixtures");
        let rich = written(builder());
        let variant = written(variant_builder());
        std::fs::write(format!("{base}/1.xml"), pretty(rich.xml())).expect("write");
        std::fs::write(format!("{base}/2.xml"), pretty(variant.xml())).expect("write");
    }

    #[test]
    fn serializes_the_document_to_ubl() {
        let document = written(builder());

        assert_eq!(pretty(document.xml()), include_str!("ubl/fixtures/1.xml"));
    }

    #[test]
    fn deserializes_the_document_from_ubl() {
        let parsed = read(include_str!("ubl/fixtures/1.xml")).expect("a valid UBL document");

        assert_eq!(parsed.builder, builder());
    }

    #[test]
    fn serializes_the_variant_document_to_ubl() {
        let document = written(variant_builder());

        assert_eq!(pretty(document.xml()), include_str!("ubl/fixtures/2.xml"));
    }

    #[test]
    fn deserializes_the_variant_document_from_ubl() {
        let parsed = read(include_str!("ubl/fixtures/2.xml")).expect("a valid UBL document");

        assert_eq!(parsed.builder, variant_builder());
    }

    #[test]
    fn round_trips_the_card_document_through_ubl() {
        let source = card_builder();
        let document = written(source.clone());

        let parsed = read(document.xml()).expect("a valid UBL document");

        assert_eq!(parsed.builder, source);
    }

    #[test]
    fn maps_nodes_to_their_contexts() {
        let dictionary = written(builder()).dictionary;

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
        let mut source = builder();
        source.profile = Profile::PeppolBisBilling30;
        let document = written(source);
        let xml = document.xml();

        assert!(xml.contains("<cbc:CustomizationID>urn:cen.eu:en16931:2017#compliant"));
        assert!(xml.contains("<cbc:Note>General note text</cbc:Note>"));
        assert!(!xml.contains("#AAB#"));
    }

    #[test]
    fn serializes_an_empty_invoice_with_zero_totals() {
        let source = empty_builder();
        let document = written(source.clone());

        assert!(
            document
                .xml()
                .contains("<cbc:InvoiceTypeCode>380</cbc:InvoiceTypeCode>")
        );
        assert!(
            document
                .xml()
                .contains("<cbc:PayableAmount>0.00</cbc:PayableAmount>")
        );
        assert!(!document.xml().contains("TaxSubtotal"));
        let parsed = read(document.xml()).expect("a valid UBL document");
        assert_eq!(parsed.builder, source);
    }

    #[test]
    fn parses_a_document_without_a_number() {
        let xml =
            include_str!("ubl/fixtures/1.xml").replacen("<cbc:ID>INV-2026-001</cbc:ID>", "", 1);

        let parsed = read(&xml).expect("a valid UBL document");

        assert_eq!(parsed.builder.invoice.number, None);
    }

    #[test]
    fn rejects_a_document_without_a_type_code() {
        let xml = r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2" xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2"><cbc:CustomizationID>urn:cen.eu:en16931:2017</cbc:CustomizationID></Invoice>"#;

        let outcome = read(xml);

        assert!(matches!(outcome, Err(Error::MalformedXml { .. })));
    }

    #[test]
    fn rejects_a_standard_rated_category_without_a_rate() {
        let xml = r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2" xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2" xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2"><cbc:CustomizationID>urn:cen.eu:en16931:2017</cbc:CustomizationID><cbc:InvoiceTypeCode>380</cbc:InvoiceTypeCode><cac:InvoiceLine><cac:Item><cac:ClassifiedTaxCategory><cbc:ID>S</cbc:ID><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:ClassifiedTaxCategory></cac:Item></cac:InvoiceLine></Invoice>"#;

        let outcome = read(xml);

        assert!(matches!(outcome, Err(Error::MalformedXml { .. })));
    }

    #[test]
    fn rejects_an_element_of_an_unknown_namespace() {
        let xml = r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2" xmlns:foo="urn:example:unknown"><foo:Bar/></Invoice>"#;

        let outcome = read(xml);

        assert!(matches!(
            outcome,
            Err(Error::MalformedXml { message, .. }) if message.starts_with("unknown namespace")
        ));
    }
}
