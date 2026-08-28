//! Issues an invoice:
//! * fills the model,
//! * serializes it under a profile,
//! * sends the XML to the KoSIT validator,
//! * and reads the answer back.
//!
//! The example needs a live validator. Start the services first (`cargo make env-up`), then run:
//!
//! ```sh
//! cargo run -p en16931-examples --example send_invoice
//! ```

use en16931_core::{
    Binding, BusinessProcess, Buyer, Contact, CreditTransfer, Document, DocumentBuilder,
    ElectronicAddress, ElectronicAddressScheme, Invoice, InvoiceLine, Item, LegalEntity,
    PaymentDetails, PaymentInstructions, PaymentMeans, Percentage, Period, PostalAddress, Price,
    Profile, Quantity, RawReport, Seller, VatTreatment,
};
use en16931_iso::Iso;
use en16931_kosit::Kosit;
use iso_currency::Currency;
use isocountry::CountryCode;
use rust_decimal::Decimal;
use time::{Date, Month};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Stage the invoice for one profile and one binding.
    // The XML is rendered under the hood as part of the document.
    let document = Document::try_from(DocumentBuilder {
        invoice: prepare_invoice()?,
        profile: Profile::XRechnung30,
        binding: Binding::Ubl,
        business_process: Some(BusinessProcess::PEPPOL_BILLING),
    })?;
    let target = document.target();
    println!(
        "Sending a {:?} {:?} document under {}",
        target.kind, target.binding, target.profile
    );

    // Send the XML to the validator. The transport is yours: any HTTP client will do.
    let answer = post(document.xml())?;

    // Read the answer: the `Kosit` wrapper opens the envelope of the service,
    // and the `Iso` normalizer reads the addresses its processor writes.
    let report = RawReport::parse(&answer, &Kosit, &Iso)?;

    // Bind the findings of the validator to the fields of the model.
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

// Sends an XML body to the KoSIT deployment and returns its report.
fn post(xml: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = std::env::var("KOSIT_URL").unwrap_or_else(|_| "http://localhost:8082".to_owned());
    Ok(reqwest::blocking::Client::new()
        .post(url)
        .header("Content-Type", "application/xml")
        .body(xml.to_owned())
        .send()?
        .text()?)
}

// The business facts of the invoice.
fn prepare_invoice() -> Result<Invoice, Box<dyn std::error::Error>> {
    Ok(Invoice {
        number: "INV-2026-001".parse()?,
        issue_date: Date::from_calendar_date(2026, Month::January, 15)?,
        type_code: "380".parse()?,
        currency: Currency::EUR,
        vat_accounting_total: None,
        vat_point: None,
        payment_due_date: Some(Date::from_calendar_date(2026, Month::February, 15)?),
        buyer_reference: Some("04011000-12345-03".parse()?),
        project_reference: None,
        contract_reference: None,
        purchase_order_reference: None,
        sales_order_reference: None,
        receiving_advice_reference: None,
        despatch_advice_reference: None,
        tender_or_lot_reference: None,
        object: None,
        buyer_accounting_reference: None,
        payment_terms: Some("Payable within 30 days".parse()?),
        notes: Vec::new(),
        preceding_invoices: Vec::new(),
        seller: Seller {
            name: "Seller Official Name".parse()?,
            trading_name: None,
            identifiers: Vec::new(),
            legal_entity: Some(LegalEntity {
                id: "DE123456".parse()?,
                issuer: None,
            }),
            additional_legal_information: None,
            vat: Some("DE123456789".parse()?),
            tax_registration: None,
            electronic_address: Some(ElectronicAddress {
                id: "4035811991007".parse()?,
                scheme: ElectronicAddressScheme::EanLocationCode,
            }),
            address: PostalAddress {
                line1: Some("Main street 1".parse()?),
                line2: None,
                line3: None,
                city: Some("Berlin".parse()?),
                country: CountryCode::for_alpha2("DE")?,
                country_subdivision: None,
                postal_code: Some("10115".parse()?),
            },
            contact: Some(Contact {
                name: Some("Anna Seller".parse()?),
                telephone: Some("+49 30 123456".parse()?),
                email: Some("anna@example.de".parse()?),
            }),
        },
        buyer: Buyer {
            name: "Buyer Official Name".parse()?,
            trading_name: None,
            identifiers: Vec::new(),
            legal_entity: None,
            vat: None,
            electronic_address: Some(ElectronicAddress {
                id: "4035812991006".parse()?,
                scheme: ElectronicAddressScheme::EanLocationCode,
            }),
            address: PostalAddress {
                line1: Some("Main street 1".parse()?),
                line2: None,
                line3: None,
                city: Some("Berlin".parse()?),
                country: CountryCode::for_alpha2("DE")?,
                country_subdivision: None,
                postal_code: Some("10115".parse()?),
            },
            contact: None,
        },
        payee: None,
        tax_representative: None,
        delivery: None,
        invoicing_period: Some(Period::Range {
            start: Date::from_calendar_date(2026, Month::January, 1)?,
            end: Date::from_calendar_date(2026, Month::January, 31)?,
        }),
        adjustments: Vec::new(),
        rounding: None,
        payment: Some(PaymentInstructions {
            means: PaymentMeans::CreditTransfer,
            means_text: None,
            remittance_information: None,
            details: Some(PaymentDetails::CreditTransfers(vec![CreditTransfer {
                account: "DE89370400440532013000".parse()?,
                account_name: None,
                provider: None,
            }])),
        }),
        paid: None,
        supporting_documents: Vec::new(),
        lines: vec![InvoiceLine {
            id: "1".parse()?,
            note: None,
            object: None,
            quantity: Quantity {
                unit: "C62".parse()?,
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
                rate: Percentage::try_from(Decimal::from(19))?,
            },
            item: Item {
                name: "Item name".parse()?,
                description: None,
                seller_id: None,
                buyer_id: None,
                standard_id: None,
                classifications: Vec::new(),
                country_of_origin: None,
                attributes: Vec::new(),
            },
        }],
    })
}
