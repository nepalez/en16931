mod deserialize;
mod serialize;

use super::{Format, Sealed, Token};
use crate::prelude::*;

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
    use crate::format::test_helpers::{
        builder, card_builder, empty_builder, pretty, variant_builder,
    };
    use crate::{Binding, Document, DocumentBuilder, Error, Invoice, InvoiceLine, Price};

    fn written(builder: DocumentBuilder<Invoice>) -> Document<Invoice, Cii> {
        Document::try_from(builder).expect("a serialized document")
    }

    fn read(xml: &str) -> Result<Document<Invoice, Cii>, Error> {
        Document::parse(xml)
    }

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
        let document = written(builder(Binding::Cii));

        assert_eq!(
            Binding::detect(document.xml()).expect("a CII document"),
            Binding::Cii
        );
    }

    #[test]
    fn rebuilds_the_same_dictionary_on_parse() {
        let document = written(builder(Binding::Cii));

        let parsed = read(document.xml()).expect("a valid CII document");

        assert_eq!(parsed.dictionary, document.dictionary);
    }

    #[test]
    fn binds_the_abbreviations_it_writes() {
        let abbreviations = written(builder(Binding::Cii)).abbreviations;

        assert_eq!(abbreviations.resolve("rsm"), Some(Namespace::Rsm));
        assert_eq!(abbreviations.resolve("ram"), Some(Namespace::Ram));
        assert_eq!(abbreviations.resolve("udt"), Some(Namespace::Udt));
        assert_eq!(abbreviations.resolve("qdt"), Some(Namespace::Qdt));
    }

    #[test]
    fn rebuilds_the_same_abbreviations_on_parse() {
        let document = written(builder(Binding::Cii));

        let parsed = read(document.xml()).expect("a valid CII document");

        assert_eq!(parsed.abbreviations, document.abbreviations);
    }

    #[test]
    fn binds_an_abbreviation_of_its_own_choice() {
        // The fixture is the rich document with `ram` renamed to `bar`.
        let parsed = read(include_str!("cii/fixtures/3.xml")).expect("a valid CII document");

        // The document's own abbreviation joins the ones the rule sets bind.
        assert_eq!(parsed.abbreviations.resolve("bar"), Some(Namespace::Ram));
        assert_eq!(parsed.abbreviations.resolve("ram"), Some(Namespace::Ram));
    }

    #[test]
    fn emit_fixtures() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/src/format/cii/fixtures");
        let rich = written(builder(Binding::Cii));
        let variant = written(variant_builder(Binding::Cii));
        std::fs::write(format!("{base}/1.xml"), pretty(rich.xml())).expect("write");
        std::fs::write(format!("{base}/2.xml"), pretty(variant.xml())).expect("write");
    }

    #[test]
    fn serializes_the_document_to_cii() {
        let document = written(builder(Binding::Cii));

        assert_eq!(pretty(document.xml()), include_str!("cii/fixtures/1.xml"));
    }

    #[test]
    fn deserializes_the_document_from_cii() {
        let parsed = read(include_str!("cii/fixtures/1.xml")).expect("a valid CII document");

        assert_eq!(parsed.builder, builder(Binding::Cii));
    }

    #[test]
    fn serializes_the_variant_document_to_cii() {
        let document = written(variant_builder(Binding::Cii));

        assert_eq!(pretty(document.xml()), include_str!("cii/fixtures/2.xml"));
    }

    #[test]
    fn deserializes_the_variant_document_from_cii() {
        let parsed = read(include_str!("cii/fixtures/2.xml")).expect("a valid CII document");

        assert_eq!(parsed.builder, variant_builder(Binding::Cii));
    }

    #[test]
    fn round_trips_the_card_document_through_cii() {
        let source = card_builder(Binding::Cii);
        let document = written(source.clone());

        let parsed = read(document.xml()).expect("a valid CII document");

        assert_eq!(parsed.builder, source);
    }

    // A document of the given body under the base profile, with every CII namespace declared.
    fn document(body: &str) -> String {
        format!(
            r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100" xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100"><rsm:ExchangedDocumentContext><ram:GuidelineSpecifiedDocumentContextParameter><ram:ID>urn:cen.eu:en16931:2017</ram:ID></ram:GuidelineSpecifiedDocumentContextParameter></rsm:ExchangedDocumentContext>{body}</rsm:CrossIndustryInvoice>"#
        )
    }

    #[test]
    fn serializes_an_empty_invoice_without_totals() {
        let source = empty_builder();
        let document = written(source.clone());

        assert!(document.xml().contains("<ram:TypeCode>380</ram:TypeCode>"));
        assert!(!document.xml().contains("MonetarySummation"));
        assert!(!document.xml().contains("ApplicableTradeTax"));
        let parsed = read(document.xml()).expect("a valid CII document");
        assert_eq!(parsed.builder, source);
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
        let xml =
            include_str!("cii/fixtures/1.xml").replacen("<ram:ID>INV-2026-001</ram:ID>", "", 1);

        let parsed = read(&xml).expect("a valid CII document");

        assert_eq!(parsed.builder.invoice.number, None);
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
}
