//! Golden and round-trip tests of the base invoice through the UBL binding.

mod common;

use common::{
    bound, builder, card_builder, context, empty_builder, field, instance, location, pretty,
    variant_builder,
};
use en16931_cius::{Invoice, InvoiceLine};
use en16931_core::{Binding, Decimal, Document, DocumentBuilder, Error, Price, Profile, Ubl};

fn written(builder: DocumentBuilder<Invoice>) -> Document<Invoice, Ubl> {
    Document::try_from(builder).expect("a serialized document")
}

fn read(xml: &str) -> Result<Document<Invoice, Ubl>, Error> {
    Document::parse(xml)
}

#[test]
fn detects_its_own_output_as_ubl() {
    let document = written(builder(Binding::Ubl));

    assert_eq!(
        Binding::detect(document.xml()).expect("a UBL document"),
        Binding::Ubl
    );
}

#[test]
fn binds_the_default_namespace_it_writes() {
    let document = written(builder(Binding::Ubl));

    // The root carries the default namespace, so its abbreviation is empty.
    let address = location(&[("", "Invoice", 1), ("cac", "InvoiceLine", 2)]);

    assert_eq!(
        bound(&document, address),
        Some(context(vec![instance("lines", 2)]))
    );
}

#[test]
fn emit_fixtures() {
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ubl");
    let rich = written(builder(Binding::Ubl));
    let variant = written(variant_builder(Binding::Ubl));
    std::fs::write(format!("{base}/1.xml"), pretty(rich.xml())).expect("write");
    std::fs::write(format!("{base}/2.xml"), pretty(variant.xml())).expect("write");
}

#[test]
fn serializes_the_document_to_ubl() {
    let document = written(builder(Binding::Ubl));

    assert_eq!(pretty(document.xml()), include_str!("fixtures/ubl/1.xml"));
}

#[test]
fn deserializes_the_document_from_ubl() {
    let parsed = read(include_str!("fixtures/ubl/1.xml")).expect("a valid UBL document");

    let source = builder(Binding::Ubl);
    assert_eq!(parsed.target().profile, source.profile);
    assert_eq!(parsed.into_invoice(), source.invoice);
}

#[test]
fn serializes_the_variant_document_to_ubl() {
    let document = written(variant_builder(Binding::Ubl));

    assert_eq!(pretty(document.xml()), include_str!("fixtures/ubl/2.xml"));
}

#[test]
fn deserializes_the_variant_document_from_ubl() {
    let parsed = read(include_str!("fixtures/ubl/2.xml")).expect("a valid UBL document");

    assert_eq!(parsed.into_invoice(), variant_builder(Binding::Ubl).invoice);
}

#[test]
fn round_trips_the_card_document_through_ubl() {
    let source = card_builder(Binding::Ubl);
    let document = written(source.clone());

    let parsed = read(document.xml()).expect("a valid UBL document");

    assert_eq!(parsed.into_invoice(), source.invoice);
}

#[test]
fn maps_nodes_to_their_contexts() {
    let document = written(builder(Binding::Ubl));

    let root = location(&[("ubl", "Invoice", 1)]);
    let seller_name = location(&[
        ("ubl", "Invoice", 1),
        ("cac", "AccountingSupplierParty", 1),
        ("cac", "Party", 1),
        ("cac", "PartyLegalEntity", 1),
        ("cbc", "RegistrationName", 1),
    ]);
    let second_line = location(&[("ubl", "Invoice", 1), ("cac", "InvoiceLine", 2)]);
    let payable = location(&[
        ("ubl", "Invoice", 1),
        ("cac", "LegalMonetaryTotal", 1),
        ("cbc", "PayableAmount", 1),
    ]);
    let second_line_net = location(&[
        ("ubl", "Invoice", 1),
        ("cac", "InvoiceLine", 2),
        ("cbc", "LineExtensionAmount", 1),
    ]);

    // The root, a term-less node, resolves to the root context.
    assert_eq!(bound(&document, root), Some(context(Vec::new())));
    // The registration name nests its field under the seller group.
    assert_eq!(
        bound(&document, seller_name),
        Some(context(vec![field("seller"), field("name")]))
    );
    // The second line resolves to its instance, carrying index 2.
    assert_eq!(
        bound(&document, second_line),
        Some(context(vec![instance("lines", 2)]))
    );
    // A stated total maps to its own field.
    assert_eq!(bound(&document, payable), Some(context(vec![field("due")])));
    // The net amount of a line maps to the field of its instance.
    assert_eq!(
        bound(&document, second_line_net),
        Some(context(vec![instance("lines", 2), field("net_amount")]))
    );
}

#[test]
fn drops_a_term_the_profile_forbids() {
    // Peppol BIS forbids the note subject code (BT-21).
    let mut source = builder(Binding::Ubl);
    source.profile = Profile::PeppolBisBilling30;
    let document = written(source);
    let xml = document.xml();

    assert!(xml.contains("<cbc:CustomizationID>urn:cen.eu:en16931:2017#compliant"));
    assert!(xml.contains("<cbc:Note>General note text</cbc:Note>"));
    assert!(!xml.contains("#AAB#"));
}

#[test]
fn serializes_an_empty_invoice_without_totals() {
    let source = empty_builder();
    let document = written(source.clone());

    assert!(
        document
            .xml()
            .contains("<cbc:InvoiceTypeCode>380</cbc:InvoiceTypeCode>")
    );
    assert!(!document.xml().contains("LegalMonetaryTotal"));
    assert!(!document.xml().contains("TaxTotal"));
    let parsed = read(document.xml()).expect("a valid UBL document");
    assert_eq!(parsed.into_invoice(), source.invoice);
}

#[test]
fn serializes_a_stated_zero_total() {
    let mut source = empty_builder();
    source.invoice.allowances_total = Some(Decimal::ZERO);

    let document = written(source);

    assert!(
        document
            .xml()
            .contains("<cbc:AllowanceTotalAmount>0.00</cbc:AllowanceTotalAmount>")
    );
}

#[test]
fn rounds_an_amount_to_two_decimals_on_serialization() {
    let mut source = empty_builder();
    source.invoice.due = Some(Decimal::new(10505, 3));
    source.invoice.rounding = Some(Decimal::new(-10505, 3));
    source.invoice.paid = Some(Decimal::new(105, 1));

    let document = written(source.clone());
    let xml = document.xml();

    assert!(xml.contains("<cbc:PayableAmount>10.51</cbc:PayableAmount>"));
    assert!(xml.contains("<cbc:PayableRoundingAmount>-10.51</cbc:PayableRoundingAmount>"));
    assert!(xml.contains("<cbc:PrepaidAmount>10.50</cbc:PrepaidAmount>"));
    // The model keeps the amount as the issuer stated it.
    assert_eq!(document.into_invoice().due, Some(Decimal::new(10505, 3)));
}

#[test]
fn serializes_a_price_with_its_own_scale() {
    let mut source = empty_builder();
    source.invoice.lines = vec![InvoiceLine {
        price: Some(Price {
            net: Some(Decimal::new(5, 3)),
            ..Default::default()
        }),
        ..Default::default()
    }];

    let document = written(source);

    assert!(
        document
            .xml()
            .contains("<cbc:PriceAmount>0.005</cbc:PriceAmount>")
    );
}

#[test]
fn parses_a_document_without_a_number() {
    let xml = include_str!("fixtures/ubl/1.xml").replacen("<cbc:ID>INV-2026-001</cbc:ID>", "", 1);

    let parsed = read(&xml).expect("a valid UBL document");

    assert_eq!(parsed.into_invoice().number, None);
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
