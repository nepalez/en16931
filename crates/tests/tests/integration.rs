//! Integration tests of the multi-profile scenario against the live validators.
//!
//! One business invoice holds the shared content. Each profile gets its own
//! serialization, validated by the service that carries the rules of that
//! profile, and the answer comes back through a `Wrapper`/`Normalizer` pair
//! into `Document::check`.
//!
//! The tests need the services of step 2. Start them with `cargo make env-up`, then run:
//!
//! ```sh
//! cargo test -p en16931-tests --test integration -- --ignored
//! ```

use en16931_core::{
    BusinessProcess, Buyer, Cii, Contact, CreditTransfer, Document, DocumentBuilder,
    ElectronicAddress, ElectronicAddressScheme, Format, InvalidDocument, Invoice, InvoiceLine,
    Item, LegalEntity, PaymentDetails, PaymentInstructions, PaymentMeans, Percentage, Period,
    PostalAddress, Price, Profile, Quantity, RawReport, Seller, Ubl, Unit, ValidDocument,
    VatBreakdown, VatIdentifier, VatTreatment, Wrapper,
};
use en16931_iso::Iso;
use en16931_kosit::Kosit;
use en16931_phive::Phive;
use iso_currency::Currency;
use isocountry::CountryCode;
use rust_decimal::Decimal;
use time::{Date, Month};

// The one invoice every profile of the scenario shares.
fn invoice() -> Invoice {
    Invoice {
        number: Some("INV-1".parse().expect("a number")),
        issue_date: Some(Date::from_calendar_date(2026, Month::January, 15).expect("a date")),
        type_code: "380".parse().expect("a type code"),
        currency: Some(Currency::EUR),
        payment_due_date: Some(
            Date::from_calendar_date(2026, Month::February, 15).expect("a date"),
        ),
        buyer_reference: Some("04011000-12345-03".parse().expect("a reference")),
        payment_terms: Some("Payable within 30 days".parse().expect("terms")),
        seller: Some(seller()),
        buyer: Some(buyer()),
        invoicing_period: Some(Period::Range {
            start: Date::from_calendar_date(2026, Month::January, 1).expect("a date"),
            end: Date::from_calendar_date(2026, Month::January, 31).expect("a date"),
        }),
        payment: Some(payment()),
        line_net_total: Some(Decimal::new(20000, 2)),
        net_total: Some(Decimal::new(20000, 2)),
        vat_total: Some(Decimal::new(3800, 2)),
        gross_total: Some(Decimal::new(23800, 2)),
        due: Some(Decimal::new(23800, 2)),
        vat_breakdown: vec![VatBreakdown {
            treatment: Some(vat()),
            taxable: Some(Decimal::new(20000, 2)),
            tax: Some(Decimal::new(3800, 2)),
        }],
        lines: vec![line()],
        ..Default::default()
    }
}

// The standard VAT rate the line and its breakdown group share.
fn vat() -> VatTreatment {
    VatTreatment::Standard {
        rate: Percentage::try_from(Decimal::from(19)).expect("a rate"),
    }
}

fn seller() -> Seller {
    Seller {
        name: Some("Seller Official Name".parse().expect("a name")),
        legal_entity: Some(LegalEntity {
            id: Some("DE123456".parse().expect("an id")),
            issuer: None,
        }),
        vat: Some(VatIdentifier::build(country("DE"), "123456789").expect("a vat id")),
        electronic_address: Some(ElectronicAddress {
            id: Some("4035811991007".parse().expect("an address")),
            scheme: Some(ElectronicAddressScheme::EanLocationCode),
        }),
        address: Some(address("DE")),
        contact: Some(Contact {
            name: Some("Anna Seller".parse().expect("a name")),
            telephone: Some("+49 30 123456".parse().expect("a phone")),
            email: Some("anna@example.de".parse().expect("an email")),
        }),
        ..Default::default()
    }
}

fn buyer() -> Buyer {
    Buyer {
        name: Some("Buyer Official Name".parse().expect("a name")),
        electronic_address: Some(ElectronicAddress {
            id: Some("4035812991006".parse().expect("an address")),
            scheme: Some(ElectronicAddressScheme::EanLocationCode),
        }),
        address: Some(address("DE")),
        ..Default::default()
    }
}

fn payment() -> PaymentInstructions {
    PaymentInstructions {
        means: Some(PaymentMeans::CreditTransfer),
        details: Some(PaymentDetails::CreditTransfers(vec![CreditTransfer {
            account: Some("DE89370400440532013000".parse().expect("an account")),
            ..Default::default()
        }])),
        ..Default::default()
    }
}

fn line() -> InvoiceLine {
    InvoiceLine {
        id: Some("1".parse().expect("an id")),
        quantity: Some(Quantity {
            unit: Unit::from_code("C62").expect("a unit"),
            value: Decimal::from(2),
        }),
        net_amount: Some(Decimal::new(20000, 2)),
        price: Some(Price {
            net: Some(Decimal::new(10000, 2)),
            gross: Some(Decimal::new(10000, 2)),
            ..Default::default()
        }),
        vat: Some(vat()),
        item: Some(Item {
            name: Some("Item name".parse().expect("a name")),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn address(code: &str) -> PostalAddress {
    PostalAddress {
        line1: Some("Main street 1".parse().expect("a line")),
        city: Some("Berlin".parse().expect("a city")),
        country: Some(country(code)),
        postal_code: Some("10115".parse().expect("a code")),
        ..Default::default()
    }
}

fn country(code: &str) -> CountryCode {
    CountryCode::for_alpha2(code).expect("a country code")
}

// The Peppol billing process every serialization of the scenario declares.
fn business_process() -> Option<BusinessProcess> {
    Some(
        "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
            .parse()
            .expect("a business process"),
    )
}

// Posts an XML body to a validator and returns the response text.
fn post(url: &str, headers: &[(&str, &str)], body: String) -> String {
    let mut request = reqwest::blocking::Client::new().post(url).body(body);
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    request
        .send()
        .expect("the validator request to succeed")
        .text()
        .expect("a response body")
}

// Sends the document to the phive service under the rule set its target names.
fn phive_answer<F: Format>(document: &Document<Invoice, F>) -> String {
    let base = std::env::var("PHIVE_URL").unwrap_or_else(|_| "http://localhost:8083".to_owned());
    let token = std::env::var("PHIVE_TOKEN").unwrap_or_else(|_| "phorm-dev-token".to_owned());
    let rules = Phive
        .vendor_id(document.target())
        .expect("a rule set for the target");
    post(
        &format!("{base}/api/validate/{rules}:latest"),
        &[
            ("X-Token", &token),
            ("Content-Type", "application/xml"),
            ("Accept", "application/xml"),
        ],
        document.xml().to_owned(),
    )
}

// Sends the document to the KoSIT deployment its profile routes to.
fn kosit_answer<F: Format>(document: &Document<Invoice, F>) -> String {
    let url = std::env::var("KOSIT_URL").unwrap_or_else(|_| "http://localhost:8082".to_owned());
    post(
        &url,
        &[("Content-Type", "application/xml")],
        document.xml().to_owned(),
    )
}

// Reads the answer through a wrapper paired with the ISO normalizer, and checks the document.
#[allow(clippy::result_large_err)]
fn outcome<W: Wrapper, F: Format>(
    document: Document<Invoice, F>,
    wrapper: &W,
    answer: &str,
) -> Result<ValidDocument<Invoice, F>, InvalidDocument<Invoice, F>> {
    let report = RawReport::parse(answer, wrapper, &Iso).expect("a report the pair reads");
    document.check(report).expect("every address to bind")
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_the_nlcius_serialization_against_phive() {
    let serialized = Document::<Invoice, Ubl>::try_from(DocumentBuilder {
        invoice: invoice(),
        profile: Profile::Nlcius10,
        business_process: business_process(),
    })
    .expect("a document");
    let answer = phive_answer(&serialized);

    let checked = outcome(serialized, &Phive, &answer);

    assert!(
        checked.is_ok(),
        "phive should accept the NLCIUS invoice: {:?}",
        checked.err()
    );
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_the_peppol_serialization_against_phive() {
    let serialized = Document::<Invoice, Ubl>::try_from(DocumentBuilder {
        invoice: invoice(),
        profile: Profile::PeppolBisBilling30,
        business_process: business_process(),
    })
    .expect("a document");
    let answer = phive_answer(&serialized);

    let checked = outcome(serialized, &Phive, &answer);

    assert!(
        checked.is_ok(),
        "phive should accept the Peppol invoice: {:?}",
        checked.err()
    );
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn reports_a_missing_buyer_reference_as_a_finding_of_phive() {
    let serialized = Document::<Invoice, Ubl>::try_from(DocumentBuilder {
        invoice: Invoice {
            buyer_reference: None,
            ..invoice()
        },
        profile: Profile::PeppolBisBilling30,
        business_process: business_process(),
    })
    .expect("a document");
    let answer = phive_answer(&serialized);

    let checked = outcome(serialized, &Phive, &answer);

    assert!(
        checked.is_err(),
        "phive should report the missing buyer reference as a finding"
    );
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_the_xrechnung_cii_serialization_against_kosit() {
    let serialized = Document::<Invoice, Cii>::try_from(DocumentBuilder {
        invoice: invoice(),
        profile: Profile::XRechnung30,
        business_process: business_process(),
    })
    .expect("a document");
    let answer = kosit_answer(&serialized);

    let checked = outcome(serialized, &Kosit, &answer);

    assert!(
        checked.is_ok(),
        "kosit should accept the CII invoice: {:?}",
        checked.err()
    );
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_the_en16931_cii_serialization_against_phive() {
    let serialized = Document::<Invoice, Cii>::try_from(DocumentBuilder {
        invoice: invoice(),
        profile: Profile::En16931,
        business_process: business_process(),
    })
    .expect("a document");
    let answer = phive_answer(&serialized);

    let checked = outcome(serialized, &Phive, &answer);

    assert!(
        checked.is_ok(),
        "phive should accept the CII invoice: {:?}",
        checked.err()
    );
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_the_xrechnung_serialization_against_both_services() {
    let serialized = Document::<Invoice, Ubl>::try_from(DocumentBuilder {
        invoice: invoice(),
        profile: Profile::XRechnung30,
        business_process: business_process(),
    })
    .expect("a document");
    let from_kosit = kosit_answer(&serialized);
    let from_phive = phive_answer(&serialized);

    let routed = outcome(serialized.clone(), &Kosit, &from_kosit);
    let identified = outcome(serialized, &Phive, &from_phive);

    assert!(
        routed.is_ok(),
        "kosit should accept the XRechnung invoice: {:?}",
        routed.err()
    );
    assert!(
        identified.is_ok(),
        "phive should accept the XRechnung invoice: {:?}",
        identified.err()
    );
}
