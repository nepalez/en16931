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
    let document = Document::try_from(
        DocumentBuilder::builder()
            .invoice(prepare_invoice()?)
            .profile(Profile::XRechnung30)
            .binding(Binding::Ubl)
            .business_process(BusinessProcess::PEPPOL_BILLING)
            .build(),
    )?;
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
    Ok(Invoice::builder()
        .number("INV-2026-001".parse()?)
        .issue_date(Date::from_calendar_date(2026, Month::January, 15)?)
        .type_code("380".parse()?)
        .currency(Currency::EUR)
        .payment_due_date(Date::from_calendar_date(2026, Month::February, 15)?)
        .buyer_reference("04011000-12345-03".parse()?)
        .payment_terms("Payable within 30 days".parse()?)
        .seller(
            Seller::builder()
                .name("Seller Official Name".parse()?)
                .legal_entity(LegalEntity::builder().id("DE123456".parse()?).build())
                .vat("DE123456789".parse()?)
                .electronic_address(ElectronicAddress {
                    id: "4035811991007".parse()?,
                    scheme: ElectronicAddressScheme::EanLocationCode,
                })
                .address(
                    PostalAddress::builder()
                        .line1("Main street 1".parse()?)
                        .city("Berlin".parse()?)
                        .country(CountryCode::for_alpha2("DE")?)
                        .postal_code("10115".parse()?)
                        .build(),
                )
                .contact(
                    Contact::builder()
                        .name("Anna Seller".parse()?)
                        .telephone("+49 30 123456".parse()?)
                        .email("anna@example.de".parse()?)
                        .build(),
                )
                .build(),
        )
        .buyer(
            Buyer::builder()
                .name("Buyer Official Name".parse()?)
                .electronic_address(ElectronicAddress {
                    id: "4035812991006".parse()?,
                    scheme: ElectronicAddressScheme::EanLocationCode,
                })
                .address(
                    PostalAddress::builder()
                        .line1("Main street 1".parse()?)
                        .city("Berlin".parse()?)
                        .country(CountryCode::for_alpha2("DE")?)
                        .postal_code("10115".parse()?)
                        .build(),
                )
                .build(),
        )
        .invoicing_period(Period::Range {
            start: Date::from_calendar_date(2026, Month::January, 1)?,
            end: Date::from_calendar_date(2026, Month::January, 31)?,
        })
        .payment(
            PaymentInstructions::builder()
                .means(PaymentMeans::CreditTransfer)
                .details(PaymentDetails::CreditTransfers(vec![
                    CreditTransfer::builder()
                        .account("DE89370400440532013000".parse()?)
                        .build(),
                ]))
                .build(),
        )
        .lines(vec![
            InvoiceLine::builder()
                .id("1".parse()?)
                .quantity(Quantity {
                    unit: "C62".parse()?,
                    value: Decimal::from(2),
                })
                .price(Price::builder().gross(Decimal::new(10000, 2)).build())
                .vat(VatTreatment::Standard {
                    rate: Percentage::try_from(Decimal::from(19))?,
                })
                .item(Item::builder().name("Item name".parse()?).build())
                .build(),
        ])
        .build())
}
