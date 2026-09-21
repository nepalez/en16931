//! Issues an invoice:
//! * fills the model,
//! * serializes it under a profile and a binding,
//! * sends the XML to the KoSIT validator,
//! * and binds the answer back to the fields of the model.
//!
//! The example needs a live validator. Start the services first (`cargo make env-up`), then run:
//!
//! ```sh
//! cargo run -p en16931-examples --example send_invoice
//! ```

use en16931_core::{
    BusinessProcess, Buyer, Contact, CreditTransfer, Document, DocumentBuilder, ElectronicAddress,
    ElectronicAddressScheme, Invoice, InvoiceLine, Item, LegalEntity, PaymentDetails,
    PaymentInstructions, PaymentMeans, Percentage, Period, PostalAddress, Price, Profile, Quantity,
    RawReport, Seller, Ubl, VatBreakdown, VatTreatment,
};
use en16931_iso::Iso;
use en16931_kosit::Kosit;
use iso_currency::Currency;
use isocountry::CountryCode;
use rust_decimal::Decimal;
use time::{Date, Month};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The profile stamps `BT-24` and drops the terms it forbids,
    // while the binding decides the vocabulary of the XML.
    let document = Document::<Invoice, Ubl>::try_from(DocumentBuilder {
        invoice: prepare_invoice()?,
        profile: Profile::XRechnung30,
        business_process: Some(BusinessProcess::PEPPOL_BILLING),
    })?;
    let target = document.target();
    println!(
        "Sending a {:?} {:?} document under {}",
        target.kind, target.binding, target.profile
    );

    // The library produces the request parts, while the application owns the transport.
    let answer = post(document.xml())?;

    // The wrapper opens the envelope of the service,
    // and the normalizer reads the addresses its processor wrote.
    let report = RawReport::parse(&answer, &Kosit, &Iso)?;

    // The dictionary binds every address of the report to the model field it points at.
    match document.check(report)? {
        Ok(valid) => {
            println!("The validator accepted the invoice.");
            for problem in valid.problems() {
                println!("{problem}");
            }
        }
        Err(invalid) => {
            println!("The validator rejected the invoice.");
            for problem in invalid.problems() {
                println!(
                    "{} at {}: {}",
                    problem.severity, problem.context, problem.text
                );
            }
        }
    }

    Ok(())
}

// Sends an XML body to the KoSIT deployment, which routes it by the image it runs.
// Unlike phive, the service takes no rule set identifier in the request.
fn post(xml: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = std::env::var("KOSIT_URL").unwrap_or_else(|_| "http://localhost:8082".to_owned());
    Ok(reqwest::blocking::Client::new()
        .post(url)
        .header("Content-Type", "application/xml")
        .body(xml.to_owned())
        .send()?
        .text()?)
}

// The business facts of the invoice, free of anything a profile or a binding adds.
fn prepare_invoice() -> Result<Invoice, Box<dyn std::error::Error>> {
    Ok(Invoice {
        number: Some("INV-2026-001".parse()?),
        issue_date: Some(Date::from_calendar_date(2026, Month::January, 15)?),
        type_code: "380".parse()?,
        currency: Some(Currency::EUR),
        payment_due_date: Some(Date::from_calendar_date(2026, Month::February, 15)?),
        buyer_reference: Some("04011000-12345-03".parse()?),
        payment_terms: Some("Payable within 30 days".parse()?),
        seller: Some(Seller {
            name: Some("Seller Official Name".parse()?),
            legal_entity: Some(LegalEntity {
                id: Some("DE123456".parse()?),
                issuer: None,
            }),
            vat: Some("DE123456789".parse()?),
            electronic_address: Some(ElectronicAddress {
                id: Some("4035811991007".parse()?),
                scheme: Some(ElectronicAddressScheme::EanLocationCode),
            }),
            address: Some(address()?),
            contact: Some(Contact {
                name: Some("Anna Seller".parse()?),
                telephone: Some("+49 30 123456".parse()?),
                email: Some("anna@example.de".parse()?),
            }),
            ..Default::default()
        }),
        buyer: Some(Buyer {
            name: Some("Buyer Official Name".parse()?),
            electronic_address: Some(ElectronicAddress {
                id: Some("4035812991006".parse()?),
                scheme: Some(ElectronicAddressScheme::EanLocationCode),
            }),
            address: Some(address()?),
            ..Default::default()
        }),
        invoicing_period: Some(Period::Range {
            start: Date::from_calendar_date(2026, Month::January, 1)?,
            end: Date::from_calendar_date(2026, Month::January, 31)?,
        }),
        payment: Some(PaymentInstructions {
            means: Some(PaymentMeans::CreditTransfer),
            details: Some(PaymentDetails::CreditTransfers(vec![CreditTransfer {
                account: Some("DE89370400440532013000".parse()?),
                ..Default::default()
            }])),
            ..Default::default()
        }),
        // The issuer states every amount: the library computes none of them.
        line_net_total: Some(Decimal::new(20000, 2)),
        net_total: Some(Decimal::new(20000, 2)),
        vat_total: Some(Decimal::new(3800, 2)),
        gross_total: Some(Decimal::new(23800, 2)),
        due: Some(Decimal::new(23800, 2)),
        vat_breakdown: vec![VatBreakdown {
            treatment: Some(VatTreatment::Standard {
                rate: Percentage::try_from(Decimal::from(19))?,
            }),
            taxable: Some(Decimal::new(20000, 2)),
            tax: Some(Decimal::new(3800, 2)),
        }],
        lines: vec![InvoiceLine {
            id: Some("1".parse()?),
            quantity: Some(Quantity {
                unit: "C62".parse()?,
                value: Decimal::from(2),
            }),
            net_amount: Some(Decimal::new(20000, 2)),
            price: Some(Price {
                net: Some(Decimal::new(10000, 2)),
                ..Default::default()
            }),
            vat: Some(VatTreatment::Standard {
                rate: Percentage::try_from(Decimal::from(19))?,
            }),
            item: Some(Item {
                name: Some("Item name".parse()?),
                ..Default::default()
            }),
            ..Default::default()
        }],
        ..Default::default()
    })
}

// The postal address both parties share in the example.
fn address() -> Result<PostalAddress, Box<dyn std::error::Error>> {
    Ok(PostalAddress {
        line1: Some("Main street 1".parse()?),
        city: Some("Berlin".parse()?),
        country: Some(CountryCode::for_alpha2("DE")?),
        postal_code: Some("10115".parse()?),
        ..Default::default()
    })
}
