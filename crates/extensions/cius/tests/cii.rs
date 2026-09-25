//! Golden and round-trip tests of the base invoice through the CII binding.

mod common;

use common::{
    bound, builder, card_builder, context, empty_builder, field, location, pretty, variant_builder,
};
use en16931_cius::{Invoice, InvoiceLine};
use en16931_core::{Binding, Cii, Decimal, Document, DocumentBuilder, Error, Price};

fn written(builder: DocumentBuilder<Invoice>) -> Document<Invoice, Cii> {
    Document::try_from(builder).expect("a serialized document")
}

fn read(xml: &str) -> Result<Document<Invoice, Cii>, Error> {
    Document::parse(xml)
}

// A document of the given body under the base profile, with every CII namespace declared.
fn document(body: &str) -> String {
    format!(
        r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100" xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100"><rsm:ExchangedDocumentContext><ram:GuidelineSpecifiedDocumentContextParameter><ram:ID>urn:cen.eu:en16931:2017</ram:ID></ram:GuidelineSpecifiedDocumentContextParameter></rsm:ExchangedDocumentContext>{body}</rsm:CrossIndustryInvoice>"#
    )
}

#[test]
fn detects_its_own_output_as_cii() {
    let document = written(builder(Binding::Cii));

    assert_eq!(
        Binding::detect(document.xml()).expect("a CII document"),
        Binding::Cii
    );
}

#[test]
fn binds_the_abbreviations_it_writes() {
    let document = written(builder(Binding::Cii));

    let address = location(&[
        ("rsm", "CrossIndustryInvoice", 1),
        ("rsm", "ExchangedDocument", 1),
        ("ram", "ID", 1),
    ]);

    assert_eq!(
        bound(&document, address),
        Some(context(vec![field("number")]))
    );
}

#[test]
fn binds_an_abbreviation_of_its_own_choice() {
    // The fixture is the rich document with `ram` renamed to `bar`.
    let parsed = read(include_str!("fixtures/cii/3.xml")).expect("a valid CII document");

    // The document's own abbreviation joins the ones the rule sets bind.
    let own = location(&[
        ("rsm", "CrossIndustryInvoice", 1),
        ("rsm", "ExchangedDocument", 1),
        ("bar", "ID", 1),
    ]);
    let standard = location(&[
        ("rsm", "CrossIndustryInvoice", 1),
        ("rsm", "ExchangedDocument", 1),
        ("ram", "ID", 1),
    ]);

    assert_eq!(bound(&parsed, own), Some(context(vec![field("number")])));
    assert_eq!(
        bound(&parsed, standard),
        Some(context(vec![field("number")]))
    );
}

#[test]
fn emit_fixtures() {
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cii");
    let rich = written(builder(Binding::Cii));
    let variant = written(variant_builder(Binding::Cii));
    std::fs::write(format!("{base}/1.xml"), pretty(rich.xml())).expect("write");
    std::fs::write(format!("{base}/2.xml"), pretty(variant.xml())).expect("write");
}

#[test]
fn serializes_the_document_to_cii() {
    let document = written(builder(Binding::Cii));

    assert_eq!(pretty(document.xml()), include_str!("fixtures/cii/1.xml"));
}

#[test]
fn deserializes_the_document_from_cii() {
    let parsed = read(include_str!("fixtures/cii/1.xml")).expect("a valid CII document");

    let source = builder(Binding::Cii);
    assert_eq!(parsed.target().profile, source.profile);
    assert_eq!(parsed.into_invoice(), source.invoice);
}

#[test]
fn serializes_the_variant_document_to_cii() {
    let document = written(variant_builder(Binding::Cii));

    assert_eq!(pretty(document.xml()), include_str!("fixtures/cii/2.xml"));
}

#[test]
fn deserializes_the_variant_document_from_cii() {
    let parsed = read(include_str!("fixtures/cii/2.xml")).expect("a valid CII document");

    assert_eq!(parsed.into_invoice(), variant_builder(Binding::Cii).invoice);
}

#[test]
fn round_trips_the_card_document_through_cii() {
    let source = card_builder(Binding::Cii);
    let document = written(source.clone());

    let parsed = read(document.xml()).expect("a valid CII document");

    assert_eq!(parsed.into_invoice(), source.invoice);
}

#[test]
fn serializes_an_empty_invoice_without_totals() {
    let source = empty_builder();
    let document = written(source.clone());

    assert!(document.xml().contains("<ram:TypeCode>380</ram:TypeCode>"));
    assert!(!document.xml().contains("MonetarySummation"));
    assert!(!document.xml().contains("ApplicableTradeTax"));
    let parsed = read(document.xml()).expect("a valid CII document");
    assert_eq!(parsed.into_invoice(), source.invoice);
}

#[test]
fn rounds_an_amount_to_two_decimals_on_serialization() {
    let mut source = empty_builder();
    source.invoice.due = Some(Decimal::new(10505, 3));
    source.invoice.rounding = Some(Decimal::new(-10505, 3));
    source.invoice.paid = Some(Decimal::new(105, 1));

    let document = written(source);
    let xml = document.xml();

    assert!(xml.contains("<ram:DuePayableAmount>10.51</ram:DuePayableAmount>"));
    assert!(xml.contains("<ram:RoundingAmount>-10.51</ram:RoundingAmount>"));
    assert!(xml.contains("<ram:TotalPrepaidAmount>10.50</ram:TotalPrepaidAmount>"));
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
            .contains("<ram:ChargeAmount>0.005</ram:ChargeAmount>")
    );
}

#[test]
fn parses_a_document_without_a_number() {
    let xml = include_str!("fixtures/cii/1.xml").replacen("<ram:ID>INV-2026-001</ram:ID>", "", 1);

    let parsed = read(&xml).expect("a valid CII document");

    assert_eq!(parsed.into_invoice().number, None);
}

#[test]
fn rejects_a_document_without_a_type_code() {
    let xml = document("<rsm:ExchangedDocument></rsm:ExchangedDocument>");

    let outcome = read(&xml);

    assert!(matches!(outcome, Err(Error::MalformedXml { .. })));
}

#[test]
fn rejects_a_standard_rated_category_without_a_rate() {
    let xml = document(
        "<rsm:ExchangedDocument><ram:TypeCode>380</ram:TypeCode></rsm:ExchangedDocument><rsm:SupplyChainTradeTransaction><ram:IncludedSupplyChainTradeLineItem><ram:SpecifiedLineTradeSettlement><ram:ApplicableTradeTax><ram:TypeCode>VAT</ram:TypeCode><ram:CategoryCode>S</ram:CategoryCode></ram:ApplicableTradeTax></ram:SpecifiedLineTradeSettlement></ram:IncludedSupplyChainTradeLineItem></rsm:SupplyChainTradeTransaction>",
    );

    let outcome = read(&xml);

    assert!(matches!(outcome, Err(Error::MalformedXml { .. })));
}

#[test]
fn rejects_an_element_of_an_unknown_namespace() {
    let xml = r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:foo="urn:example:unknown"><foo:Bar/></rsm:CrossIndustryInvoice>"#;

    let outcome = read(xml);

    assert!(matches!(
        outcome,
        Err(Error::MalformedXml { message, .. }) if message.starts_with("unknown namespace")
    ));
}
