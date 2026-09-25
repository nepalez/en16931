//! Tests of the document envelope over the base invoice: binding report addresses
//! to model contexts, checking a validator's answer, and yielding the parts back.

mod common;

use common::{
    abbreviated, attribute, bound, builder, context, field, finding, instance, location, resolved,
};
use en16931_cius::Invoice;
use en16931_core::{
    Binding, Cii, Document, DocumentBuilder, Error, Location, LocationStep, Namespace, Problem,
    Profile, RawReport, Severity, Target, Ubl, ubl,
};

// A serialized document of the rich UBL fixture.
fn document() -> Document<Invoice, Ubl> {
    Document::try_from(builder(Binding::Ubl)).expect("a serialized document")
}

// The same document with `cbc` renamed to `foo`, an abbreviation of its own.
fn renamed() -> Document<Invoice, Ubl> {
    Document::parse(include_str!("fixtures/ubl/3.xml")).expect("a valid UBL document")
}

// The address of the registration name of the seller, a leaf field.
fn seller_name() -> Vec<LocationStep> {
    vec![
        abbreviated("ubl", "Invoice", 1),
        abbreviated("cac", "AccountingSupplierParty", 1),
        abbreviated("cac", "Party", 1),
        abbreviated("cac", "PartyLegalEntity", 1),
        abbreviated("cbc", "RegistrationName", 1),
    ]
}

// The address of the second invoice line.
fn second_line_location() -> Location {
    location(&[("ubl", "Invoice", 1), ("cac", "InvoiceLine", 2)])
}

// The context of the second invoice line, which every finding above addresses.
fn second_line() -> Context {
    context(vec![instance("lines", 2)])
}

use en16931_core::Context;

#[test]
fn binds_an_address_of_a_leaf_to_its_field() {
    let address = Location {
        steps: seller_name(),
    };

    assert_eq!(
        bound(&document(), address),
        Some(context(vec![field("seller"), field("name")]))
    );
}

#[test]
fn binds_an_address_of_a_group_to_its_instance() {
    assert_eq!(
        bound(&document(), second_line_location()),
        Some(second_line())
    );
}

#[test]
fn binds_an_address_of_a_stated_amount_to_its_field() {
    let address = location(&[
        ("ubl", "Invoice", 1),
        ("cac", "LegalMonetaryTotal", 1),
        ("cbc", "PayableAmount", 1),
    ]);

    assert_eq!(
        bound(&document(), address),
        Some(context(vec![field("due")]))
    );
}

#[test]
fn binds_an_address_of_a_term_less_node_to_the_root() {
    let address = location(&[("ubl", "Invoice", 1), ("cac", "LegalMonetaryTotal", 1)]);

    assert_eq!(bound(&document(), address), Some(context(Vec::new())));
}

#[test]
fn binds_an_address_of_an_attribute_to_the_element() {
    let mut steps = seller_name();
    steps.push(attribute("languageID"));

    assert_eq!(
        bound(&document(), Location { steps }),
        Some(context(vec![field("seller"), field("name")]))
    );
}

#[test]
fn binds_an_address_deeper_than_the_document_to_the_nearest_node() {
    let mut steps = seller_name();
    steps.push(abbreviated("cbc", "Absent", 1));

    assert_eq!(
        bound(&document(), Location { steps }),
        Some(context(vec![field("seller"), field("name")]))
    );
}

#[test]
fn resolves_a_namespace_the_dialect_wrote_in_full() {
    let address = Location {
        steps: vec![
            resolved(ubl::Namespace::Inv.uri(), "Invoice", 1),
            resolved(ubl::Namespace::Cbc.uri(), "ID", 1),
        ],
    };

    assert_eq!(
        bound(&document(), address),
        Some(context(vec![field("number")]))
    );
}

#[test]
fn resolves_an_abbreviation_of_the_document() {
    let address = location(&[("ubl", "Invoice", 1), ("foo", "ID", 1)]);

    assert_eq!(
        bound(&renamed(), address),
        Some(context(vec![field("number")]))
    );
}

#[test]
fn binds_no_address_of_another_binding() {
    let address = location(&[("rsm", "CrossIndustryInvoice", 1)]);

    assert_eq!(bound(&document(), address), None);
}

#[test]
fn binds_no_address_of_an_unknown_abbreviation() {
    let address = location(&[("xsi", "Invoice", 1)]);

    assert_eq!(bound(&document(), address), None);
}

#[test]
fn rejects_a_document_of_an_error() {
    let source = document();

    let outcome = source.check(RawReport {
        findings: vec![finding(Severity::Error, second_line_location())],
    });

    let Ok(Err(rejected)) = outcome else {
        panic!("an error should reject the document");
    };
    assert_eq!(
        rejected.problems(),
        vec![Problem {
            severity: Severity::Error,
            code: Some("BR-21".to_owned()),
            text: "each line needs an identifier".to_owned(),
            context: second_line(),
        }]
    );
}

#[test]
fn accepts_a_document_of_no_error() {
    let source = document();

    let outcome = source.check(RawReport {
        findings: vec![
            finding(Severity::Warning, second_line_location()),
            finding(Severity::Information, second_line_location()),
        ],
    });

    let Ok(Ok(accepted)) = outcome else {
        panic!("a report of no error should accept the document");
    };
    assert_eq!(accepted.problems().len(), 2);
    assert_eq!(accepted.problems()[0].context, second_line());
}

#[test]
fn yields_the_checked_document_and_its_invoice_back() {
    let source = document();

    let outcome = source.clone().check(RawReport {
        findings: vec![finding(Severity::Warning, second_line_location())],
    });

    let Ok(Ok(accepted)) = outcome else {
        panic!("a report of no error should accept the document");
    };
    assert_eq!(
        accepted.clone().into_invoice(),
        builder(Binding::Ubl).invoice
    );
    assert_eq!(Document::from(accepted), source);
}

#[test]
fn fails_the_pass_of_an_unbound_location() {
    let source = document();
    let stray = en16931_core::Entry {
        original_location: "/rsm:CrossIndustryInvoice".to_owned(),
        ..finding(
            Severity::Error,
            location(&[("rsm", "CrossIndustryInvoice", 1)]),
        )
    };

    let outcome = source.check(RawReport {
        findings: vec![finding(Severity::Error, second_line_location()), stray],
    });

    let Err(error) = outcome else {
        panic!("an unbound location should fail the pass");
    };
    assert!(
        matches!(error, Error::UnboundLocation(address) if address == "/rsm:CrossIndustryInvoice")
    );
}

#[test]
fn fails_the_pass_of_a_location_no_dialect_read() {
    let source = document();
    let unread = en16931_core::Entry {
        normalized_location: None,
        ..finding(Severity::Error, second_line_location())
    };

    let outcome = source.check(RawReport {
        findings: vec![unread],
    });

    assert!(matches!(outcome, Err(Error::UnboundLocation(_))));
}

#[test]
fn round_trips_a_builder_through_the_ubl_document() {
    let document = document();

    let parsed = Document::<Invoice, Ubl>::parse(document.xml()).expect("a parsed document");

    assert_eq!(parsed, document);
}

#[test]
fn round_trips_a_builder_through_the_cii_document() {
    let document =
        Document::<Invoice, Cii>::try_from(builder(Binding::Cii)).expect("a serialized document");

    let parsed = Document::<Invoice, Cii>::parse(document.xml()).expect("a parsed document");

    assert_eq!(parsed, document);
}

#[test]
fn yields_the_request_parts() {
    let source = builder(Binding::Ubl);
    let document = document();

    let target = Target {
        profile: source.profile,
        binding: Binding::Ubl,
        kind: source.invoice.kind,
    };

    assert!(document.xml().starts_with("<Invoice"));
    assert_eq!(document.target(), target);
}

#[test]
fn yields_the_request_parts_of_each_profile_of_one_invoice() {
    let source = builder(Binding::Ubl);

    for profile in [Profile::Nlcius10, Profile::PeppolBisBilling30] {
        let document = Document::<Invoice, Ubl>::try_from(DocumentBuilder {
            profile,
            ..source.clone()
        })
        .expect("a serialized document");
        let target = Target {
            profile,
            binding: Binding::Ubl,
            kind: source.invoice.kind,
        };

        assert_eq!(document.target(), target);
        assert!(document.xml().contains(&profile.to_string()));
    }
}

#[test]
fn yields_its_invoice_by_value() {
    let source = builder(Binding::Ubl);

    assert_eq!(document().into_invoice(), source.invoice);
}
