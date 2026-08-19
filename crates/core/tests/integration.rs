//! Integration tests that validate generated XML against the live validators.
//!
//! These tests require the validator services from step 2 to be running. Start
//! them with `cargo make env-up`, then run:
//!
//! ```sh
//! cargo test -p en16931-core --test integration -- --ignored
//! ```

use en16931_core::{
    Binding, BusinessProcess, Buyer, Contact, DocumentBuilder, ElectronicAddress,
    ElectronicAddressScheme, Invoice, InvoiceLine, Item, LegalEntity, PaymentDetails,
    PaymentInstructions, PaymentMeans, Percentage, Period, PostalAddress, Price, Profile, Quantity,
    Seller, Unit, VatIdentifier, VatTreatment,
};
use iso_currency::Currency;
use isocountry::CountryCode;
use rust_decimal::Decimal;
use time::{Date, Month};

// Builds a known-valid document under a profile and binding for the validators.
fn document(profile: Profile, binding: Binding) -> DocumentBuilder {
    DocumentBuilder {
        invoice: invoice(),
        profile,
        binding,
        business_process: Some(
            "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"
                .parse::<BusinessProcess>()
                .expect("a business process"),
        ),
    }
}

fn invoice() -> Invoice {
    Invoice {
        number: "INV-1".parse().expect("a number"),
        issue_date: Date::from_calendar_date(2026, Month::January, 15).expect("a date"),
        type_code: "380".parse().expect("a type code"),
        currency: Currency::EUR,
        vat_accounting_total: None,
        vat_point: None,
        payment_due_date: Some(
            Date::from_calendar_date(2026, Month::February, 15).expect("a date"),
        ),
        buyer_reference: Some("04011000-12345-03".parse().expect("a reference")),
        project_reference: None,
        contract_reference: None,
        purchase_order_reference: None,
        sales_order_reference: None,
        receiving_advice_reference: None,
        despatch_advice_reference: None,
        tender_or_lot_reference: None,
        object: None,
        buyer_accounting_reference: None,
        payment_terms: Some("Payable within 30 days".parse().expect("terms")),
        notes: Vec::new(),
        preceding_invoices: Vec::new(),
        seller: seller(),
        buyer: buyer(),
        payee: None,
        tax_representative: None,
        delivery: None,
        invoicing_period: Some(Period::Range {
            start: Date::from_calendar_date(2026, Month::January, 1).expect("a date"),
            end: Date::from_calendar_date(2026, Month::January, 31).expect("a date"),
        }),
        adjustments: Vec::new(),
        rounding: None,
        payment: Some(payment()),
        paid: None,
        supporting_documents: Vec::new(),
        lines: vec![line()],
    }
}

fn seller() -> Seller {
    Seller {
        name: "Seller Official Name".parse().expect("a name"),
        trading_name: None,
        identifiers: Vec::new(),
        legal_entity: Some(LegalEntity {
            id: "DE123456".parse().expect("an id"),
            issuer: None,
        }),
        additional_legal_information: None,
        vat: Some(VatIdentifier::build(country("DE"), "123456789").expect("a vat id")),
        tax_registration: None,
        electronic_address: Some(ElectronicAddress {
            id: "4035811991007".parse().expect("an address"),
            scheme: ElectronicAddressScheme::EanLocationCode,
        }),
        address: address("DE"),
        contact: Some(Contact {
            name: Some("Anna Seller".parse().expect("a name")),
            telephone: Some("+49 30 123456".parse().expect("a phone")),
            email: Some("anna@example.de".parse().expect("an email")),
        }),
    }
}

fn buyer() -> Buyer {
    Buyer {
        name: "Buyer Official Name".parse().expect("a name"),
        trading_name: None,
        identifiers: Vec::new(),
        legal_entity: None,
        vat: None,
        electronic_address: Some(ElectronicAddress {
            id: "4035812991006".parse().expect("an address"),
            scheme: ElectronicAddressScheme::EanLocationCode,
        }),
        address: address("DE"),
        contact: None,
    }
}

fn payment() -> PaymentInstructions {
    PaymentInstructions {
        means: PaymentMeans::CreditTransfer,
        means_text: None,
        remittance_information: None,
        details: Some(PaymentDetails::CreditTransfers(vec![
            en16931_core::CreditTransfer {
                account: "DE89370400440532013000".parse().expect("an account"),
                account_name: None,
                provider: None,
            },
        ])),
    }
}

fn line() -> InvoiceLine {
    InvoiceLine {
        id: "1".parse().expect("an id"),
        note: None,
        object: None,
        quantity: Quantity {
            unit: Unit::from_code("C62").expect("a unit"),
            value: Decimal::from(2),
        },
        order_line_reference: None,
        buyer_accounting_reference: None,
        period: None,
        adjustments: Vec::new(),
        price: Price {
            gross: Decimal::new(10000, 2),
            discount: None,
            base_quantity: None,
        },
        vat: VatTreatment::Standard {
            rate: Percentage::try_from(Decimal::from(19)).expect("a rate"),
        },
        item: Item {
            name: "Item name".parse().expect("a name"),
            description: None,
            seller_id: None,
            buyer_id: None,
            standard_id: None,
            classifications: Vec::new(),
            country_of_origin: None,
            attributes: Vec::new(),
        },
    }
}

fn address(code: &str) -> PostalAddress {
    PostalAddress {
        line1: Some("Main street 1".parse().expect("a line")),
        line2: None,
        line3: None,
        city: Some("Berlin".parse().expect("a city")),
        country: country(code),
        country_subdivision: None,
        postal_code: Some("10115".parse().expect("a code")),
    }
}

fn country(code: &str) -> CountryCode {
    CountryCode::for_alpha2(code).expect("a country code")
}

fn kosit_url() -> String {
    std::env::var("KOSIT_URL").unwrap_or_else(|_| "http://localhost:8082".to_owned())
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

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_ubl_against_kosit() {
    let (xml, _, _) = Binding::Ubl.serialize(&document(Profile::XRechnung30, Binding::Ubl));
    let body = post(&kosit_url(), &[("Content-Type", "application/xml")], xml);

    assert!(
        body.contains("<rep:accept"),
        "kosit should accept the UBL invoice"
    );
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_cii_against_kosit() {
    let (xml, _, _) = Binding::Cii.serialize(&document(Profile::XRechnung30, Binding::Cii));
    let body = post(&kosit_url(), &[("Content-Type", "application/xml")], xml);

    assert!(
        body.contains("<rep:accept"),
        "kosit should accept the CII invoice"
    );
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn validates_ubl_against_phive() {
    let (xml, _, _) = Binding::Ubl.serialize(&document(Profile::PeppolBisBilling30, Binding::Ubl));
    let url = std::env::var("PHIVE_URL").unwrap_or_else(|_| "http://localhost:8083".to_owned())
        + "/api/validate/eu.peppol.bis3:invoice:latest";
    let token = std::env::var("PHIVE_TOKEN").unwrap_or_else(|_| "phorm-dev-token".to_owned());
    let body = post(
        &url,
        &[("X-Token", &token), ("Content-Type", "application/xml")],
        xml,
    );

    assert!(
        body.contains("\"success\":true"),
        "phive should validate the UBL invoice"
    );
}
