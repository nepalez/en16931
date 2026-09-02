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
    Binding, BusinessProcess, Buyer, Contact, CreditTransfer, Document, DocumentBuilder,
    ElectronicAddress, ElectronicAddressScheme, InvalidDocument, Invoice, InvoiceLine, Item,
    LegalEntity, PaymentDetails, PaymentInstructions, PaymentMeans, Percentage, Period,
    PostalAddress, Price, Profile, Quantity, RawReport, Seller, Unit, ValidDocument, VatIdentifier,
    VatTreatment, Wrapper,
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
    Invoice::builder()
        .number("INV-1".parse().expect("a number"))
        .issue_date(Date::from_calendar_date(2026, Month::January, 15).expect("a date"))
        .type_code("380".parse().expect("a type code"))
        .currency(Currency::EUR)
        .payment_due_date(Date::from_calendar_date(2026, Month::February, 15).expect("a date"))
        .buyer_reference("04011000-12345-03".parse().expect("a reference"))
        .payment_terms("Payable within 30 days".parse().expect("terms"))
        .seller(seller())
        .buyer(buyer())
        .invoicing_period(Period::Range {
            start: Date::from_calendar_date(2026, Month::January, 1).expect("a date"),
            end: Date::from_calendar_date(2026, Month::January, 31).expect("a date"),
        })
        .payment(payment())
        .lines(vec![line()])
        .build()
}

fn seller() -> Seller {
    Seller::builder()
        .name("Seller Official Name".parse().expect("a name"))
        .legal_entity(
            LegalEntity::builder()
                .id("DE123456".parse().expect("an id"))
                .build(),
        )
        .vat(VatIdentifier::build(country("DE"), "123456789").expect("a vat id"))
        .electronic_address(ElectronicAddress {
            id: "4035811991007".parse().expect("an address"),
            scheme: ElectronicAddressScheme::EanLocationCode,
        })
        .address(address("DE"))
        .contact(
            Contact::builder()
                .name("Anna Seller".parse().expect("a name"))
                .telephone("+49 30 123456".parse().expect("a phone"))
                .email("anna@example.de".parse().expect("an email"))
                .build(),
        )
        .build()
}

fn buyer() -> Buyer {
    Buyer::builder()
        .name("Buyer Official Name".parse().expect("a name"))
        .electronic_address(ElectronicAddress {
            id: "4035812991006".parse().expect("an address"),
            scheme: ElectronicAddressScheme::EanLocationCode,
        })
        .address(address("DE"))
        .build()
}

fn payment() -> PaymentInstructions {
    PaymentInstructions::builder()
        .means(PaymentMeans::CreditTransfer)
        .details(PaymentDetails::CreditTransfers(vec![
            CreditTransfer::builder()
                .account("DE89370400440532013000".parse().expect("an account"))
                .build(),
        ]))
        .build()
}

fn line() -> InvoiceLine {
    InvoiceLine::builder()
        .id("1".parse().expect("an id"))
        .quantity(Quantity {
            unit: Unit::from_code("C62").expect("a unit"),
            value: Decimal::from(2),
        })
        .price(Price::builder().gross(Decimal::new(10000, 2)).build())
        .vat(VatTreatment::Standard {
            rate: Percentage::try_from(Decimal::from(19)).expect("a rate"),
        })
        .item(
            Item::builder()
                .name("Item name".parse().expect("a name"))
                .build(),
        )
        .build()
}

fn address(code: &str) -> PostalAddress {
    PostalAddress::builder()
        .line1("Main street 1".parse().expect("a line"))
        .city("Berlin".parse().expect("a city"))
        .country(country(code))
        .postal_code("10115".parse().expect("a code"))
        .build()
}

fn country(code: &str) -> CountryCode {
    CountryCode::for_alpha2(code).expect("a country code")
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
fn phive_answer(document: &Document) -> String {
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
fn kosit_answer(document: &Document) -> String {
    let url = std::env::var("KOSIT_URL").unwrap_or_else(|_| "http://localhost:8082".to_owned());
    post(
        &url,
        &[("Content-Type", "application/xml")],
        document.xml().to_owned(),
    )
}

// Reads the answer through a wrapper paired with the ISO normalizer, and checks the document.
#[allow(clippy::result_large_err)]
fn outcome<W: Wrapper>(
    document: Document,
    wrapper: &W,
    answer: &str,
) -> Result<ValidDocument, InvalidDocument> {
    let report = RawReport::parse(answer, wrapper, &Iso).expect("a report the pair reads");
    document.check(report).expect("every address to bind")
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_the_nlcius_serialization_against_phive() {
    let serialized = Document::try_from(
        DocumentBuilder::builder()
            .invoice(invoice())
            .profile(Profile::Nlcius10)
            .binding(Binding::Ubl)
            .business_process(
                "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
                    .parse::<BusinessProcess>()
                    .expect("a business process"),
            )
            .build(),
    )
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
    let serialized = Document::try_from(
        DocumentBuilder::builder()
            .invoice(invoice())
            .profile(Profile::PeppolBisBilling30)
            .binding(Binding::Ubl)
            .business_process(
                "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
                    .parse::<BusinessProcess>()
                    .expect("a business process"),
            )
            .build(),
    )
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
fn validates_the_xrechnung_cii_serialization_against_kosit() {
    let serialized = Document::try_from(
        DocumentBuilder::builder()
            .invoice(invoice())
            .profile(Profile::XRechnung30)
            .binding(Binding::Cii)
            .business_process(
                "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
                    .parse::<BusinessProcess>()
                    .expect("a business process"),
            )
            .build(),
    )
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
    let serialized = Document::try_from(
        DocumentBuilder::builder()
            .invoice(invoice())
            .profile(Profile::En16931)
            .binding(Binding::Cii)
            .business_process(
                "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
                    .parse::<BusinessProcess>()
                    .expect("a business process"),
            )
            .build(),
    )
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
    let serialized = Document::try_from(
        DocumentBuilder::builder()
            .invoice(invoice())
            .profile(Profile::XRechnung30)
            .binding(Binding::Ubl)
            .business_process(
                "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
                    .parse::<BusinessProcess>()
                    .expect("a business process"),
            )
            .build(),
    )
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
