//! Shared fixtures and helpers of the binding and document tests of the base invoice.

#![allow(dead_code)]

use std::num::NonZeroUsize;

use en16931_cius::{
    Buyer, Contact, Delivery, Invoice, InvoiceLine, Item, Payee, Seller, TaxRepresentative,
};
use en16931_core::{
    Adjustment, AdjustmentAmount, AdjustmentReason, AllowanceReason, Binding, BusinessProcess,
    Context, CountryCode, CreditTransfer, Currency, Date, Decimal, DirectDebit, Document,
    DocumentBuilder, ElectronicAddress, ElectronicAddressScheme, Entry, Format, InvoiceReference,
    IssuingAgency, ItemAttribute, ItemClassification, ItemClassificationScheme, ItemReference,
    LegalEntity, LineAdjustment, Location, LocationReference, LocationStep, NonEmptyString, Note,
    ObjectReference, OperationalEntity, PaymentCard, PaymentDetails, PaymentInstructions,
    PaymentMeans, Percentage, Period, PostalAddress, Price, Profile, Quantity, QuantityUnit,
    RawNamespace, RawReport, Segment, Severity, SupportingDocument, VatBreakdown, VatIdentifier,
    VatPoint, VatTreatment,
};
use time::Month;

// Builds a non-empty string, panicking on an empty input.
fn text(value: &str) -> Option<NonEmptyString> {
    Some(value.parse().expect("a non-empty string"))
}

// Builds a country code from its alpha-2 code.
fn country(code: &str) -> CountryCode {
    CountryCode::for_alpha2(code).expect("a country code")
}

// Builds a calendar date.
fn date(year: i32, month: Month, day: u8) -> Date {
    Date::from_calendar_date(year, month, day).expect("a valid date")
}

// Builds a percentage rate.
fn rate(value: i64) -> Percentage {
    Percentage::try_from(Decimal::from(value)).expect("a valid rate")
}

// Builds a quantity of units (`C62`).
fn units(value: i64) -> Quantity {
    Quantity {
        unit: QuantityUnit::from_code("C62").expect("a unit"),
        value: Decimal::from(value),
    }
}

/// A rich `DocumentBuilder` under the base EN-16931 profile, shared by the tests.
///
/// The base profile forbids no term, so it serializes and parses back unchanged.
/// The fixture follows the `binding` it is written to: UBL carries no gross price
/// without a price discount, so only the CII fixture states one for such a line.
pub fn builder(binding: Binding) -> DocumentBuilder<Invoice> {
    DocumentBuilder {
        invoice: invoice(binding),
        profile: Profile::En16931,
        business_process: Some(BusinessProcess::PEPPOL_BILLING),
    }
}

/// A `DocumentBuilder` exercising the alternative branches the rich fixture omits.
///
/// It flips the mutually-exclusive choices `builder` never reaches: a direct-debit
/// payment, an event VAT point, relative and charge adjustments, an exempt line, an
/// object scheme, a price discount, and start-only or end-only periods. Every choice
/// survives the `binding` it is written to, so its codec parses the document back unchanged.
pub fn variant_builder(binding: Binding) -> DocumentBuilder<Invoice> {
    DocumentBuilder {
        invoice: variant_invoice(binding),
        ..builder(binding)
    }
}

/// A `DocumentBuilder` whose invoice is paid with a payment card (`BG-18`).
///
/// It overrides the payment of the rich invoice, so the card details reach the
/// branch neither `builder` (credit transfer) nor `variant_builder` (direct
/// debit) exercises. The document round-trips through the `binding` it is written to.
pub fn card_builder(binding: Binding) -> DocumentBuilder<Invoice> {
    DocumentBuilder {
        invoice: Invoice {
            payment: Some(card_payment()),
            ..invoice(binding)
        },
        ..builder(binding)
    }
}

/// A `DocumentBuilder` whose invoice carries nothing but the default type code.
pub fn empty_builder() -> DocumentBuilder<Invoice> {
    DocumentBuilder {
        invoice: Invoice::default(),
        profile: Profile::En16931,
        business_process: None,
    }
}

// The base invoice with its alternative-branch fields overridden.
fn variant_invoice(binding: Binding) -> Invoice {
    Invoice {
        payment: Some(variant_payment()),
        vat_point: Some(VatPoint::try_from(35u16).expect("a vat point event")),
        object: Some(ObjectReference {
            id: text("OBJ-200"),
            scheme: Some("AAA".parse().expect("an object type")),
        }),
        adjustments: vec![variant_allowance(), variant_charge()],
        line_net_total: Some(Decimal::new(33700, 2)),
        allowances_total: Some(Decimal::new(2000, 2)),
        charges_total: Some(Decimal::new(1500, 2)),
        net_total: Some(Decimal::new(33200, 2)),
        vat_total: Some(Decimal::new(817, 2)),
        gross_total: Some(Decimal::new(34017, 2)),
        due: Some(Decimal::new(29014, 2)),
        vat_breakdown: vec![
            VatBreakdown {
                treatment: Some(VatTreatment::Exempt {
                    code: Some("VATEX-EU-132".parse().expect("an exemption reason")),
                    text: text("Exempt supply"),
                }),
                taxable: Some(Decimal::new(28900, 2)),
                tax: Some(Decimal::new(0, 2)),
            },
            VatBreakdown {
                treatment: Some(VatTreatment::Standard { rate: rate(19) }),
                taxable: Some(Decimal::new(4300, 2)),
                tax: Some(Decimal::new(817, 2)),
            },
        ],
        invoicing_period: Some(Period::Until(date(2026, Month::January, 31))),
        lines: vec![variant_line(), line(binding, "2", 1, 5000, 4800)],
        ..invoice(binding)
    }
}

// A direct-debit payment carrying a mandate, a creditor, and a debited account.
fn variant_payment() -> PaymentInstructions {
    PaymentInstructions {
        means: Some(PaymentMeans::SepaDirectDebit),
        means_text: text("Direct debit"),
        remittance_information: text("DD-REF-9"),
        details: Some(PaymentDetails::DirectDebit(DirectDebit {
            mandate_reference: text("MANDATE-7"),
            creditor_identifier: text("DE98ZZZ09999999999"),
            debited_account: Some("DE89370400440532013000".parse().expect("an account")),
        })),
    }
}

// A card payment carrying the masked number and the cardholder name.
fn card_payment() -> PaymentInstructions {
    PaymentInstructions {
        means: Some(PaymentMeans::CreditCard),
        means_text: text("Credit card"),
        details: Some(PaymentDetails::Card(PaymentCard {
            primary_account_number: text("41234"),
            holder_name: text("Card Holder"),
        })),
        ..Default::default()
    }
}

// A relative document-level allowance, a percentage of a base.
fn variant_allowance() -> Adjustment {
    Adjustment {
        amount: Some(AdjustmentAmount::Relative {
            amount: Decimal::new(2000, 2),
            rate: rate(10),
            base: Decimal::new(20000, 2),
        }),
        vat: Some(VatTreatment::Standard { rate: rate(19) }),
        reason: Some(AdjustmentReason::Allowance {
            code: Some(AllowanceReason::Discount),
            text: text("Volume discount"),
        }),
    }
}

// A document-level charge with a reason code.
fn variant_charge() -> Adjustment {
    Adjustment {
        amount: Some(AdjustmentAmount::Absolute(Decimal::new(1500, 2))),
        vat: Some(VatTreatment::Standard { rate: rate(19) }),
        reason: Some(AdjustmentReason::Charge {
            code: Some("AA".parse().expect("a charge reason")),
            text: text("Advertising"),
        }),
    }
}

// An exempt line carrying a price discount, a base quantity, a line object,
// and both a relative allowance and a charge at the line level.
fn variant_line() -> InvoiceLine {
    InvoiceLine {
        id: text("1"),
        object: Some(ObjectReference {
            id: text("LINE-OBJ-1"),
            scheme: Some("AAB".parse().expect("an object type")),
        }),
        quantity: Some(units(3)),
        net_amount: Some(Decimal::new(28900, 2)),
        period: Some(Period::From(date(2026, Month::January, 1))),
        adjustments: vec![
            LineAdjustment {
                amount: Some(AdjustmentAmount::Relative {
                    amount: Decimal::new(1500, 2),
                    rate: rate(5),
                    base: Decimal::new(30000, 2),
                }),
                reason: Some(AdjustmentReason::Allowance {
                    code: None,
                    text: text("line rebate"),
                }),
            },
            LineAdjustment {
                amount: Some(AdjustmentAmount::Absolute(Decimal::new(400, 2))),
                reason: Some(AdjustmentReason::Charge {
                    code: Some("AA".parse().expect("a charge reason")),
                    text: text("line handling"),
                }),
            },
        ],
        price: Some(Price {
            net: Some(Decimal::new(10000, 2)),
            gross: Some(Decimal::new(12000, 2)),
            discount: Some(Decimal::new(2000, 2)),
            base_quantity: Some(units(1)),
        }),
        vat: Some(VatTreatment::Exempt {
            code: Some("VATEX-EU-132".parse().expect("an exemption reason")),
            text: text("Exempt supply"),
        }),
        item: Some(Item {
            name: text("Exempt item"),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// The invoice the fixture carries.
fn invoice(binding: Binding) -> Invoice {
    Invoice {
        number: text("INV-2026-001"),
        issue_date: Some(date(2026, Month::January, 15)),
        type_code: "380".parse().expect("a type code"),
        currency: Some(Currency::EUR),
        payment_due_date: Some(date(2026, Month::February, 15)),
        buyer_reference: text("BUYER-REF-01"),
        project_reference: text("PROJECT-42"),
        contract_reference: text("CONTRACT-7"),
        purchase_order_reference: text("PO-2026-9"),
        sales_order_reference: text("SO-2026-3"),
        receiving_advice_reference: text("RECADV-2"),
        despatch_advice_reference: text("DESADV-2"),
        tender_or_lot_reference: text("TENDER-1"),
        object: Some(ObjectReference {
            id: text("OBJ-100"),
            scheme: None,
        }),
        buyer_accounting_reference: text("ACCOUNT-500"),
        payment_terms: text("Net 30 days"),
        notes: vec![Note {
            subject_code: text("AAB"),
            text: text("General note text"),
        }],
        preceding_invoices: vec![InvoiceReference {
            number: text("INV-2025-900"),
            issue_date: Some(date(2025, Month::December, 1)),
        }],
        seller: Some(seller()),
        buyer: Some(buyer()),
        payee: Some(payee()),
        tax_representative: Some(tax_representative()),
        delivery: Some(delivery()),
        invoicing_period: Some(Period::Range {
            start: date(2026, Month::January, 1),
            end: date(2026, Month::January, 31),
        }),
        adjustments: vec![allowance()],
        line_net_total: Some(Decimal::new(24600, 2)),
        allowances_total: Some(Decimal::new(1000, 2)),
        net_total: Some(Decimal::new(23600, 2)),
        vat_total: Some(Decimal::new(4484, 2)),
        gross_total: Some(Decimal::new(28084, 2)),
        rounding: Some(Decimal::new(-3, 2)),
        payment: Some(payment()),
        paid: Some(Decimal::new(5000, 2)),
        due: Some(Decimal::new(23081, 2)),
        vat_breakdown: vec![VatBreakdown {
            treatment: Some(VatTreatment::Standard { rate: rate(19) }),
            taxable: Some(Decimal::new(23600, 2)),
            tax: Some(Decimal::new(4484, 2)),
        }],
        supporting_documents: vec![supporting_document()],
        lines: vec![
            line(binding, "1", 2, 10000, 19800),
            line(binding, "2", 1, 5000, 4800),
        ],
        ..Default::default()
    }
}

fn seller() -> Seller {
    Seller {
        name: text("Seller Official Name Ltd"),
        trading_name: text("SellerTrading"),
        identifiers: vec![OperationalEntity {
            id: text("SELLER-ID-1"),
            issuer: Some("0088".parse::<IssuingAgency>().expect("an agency")),
        }],
        legal_entity: Some(LegalEntity {
            id: text("DE12345"),
            issuer: None,
        }),
        additional_legal_information: text("Registered in Berlin"),
        vat: Some(VatIdentifier::build(country("DE"), "123456789").expect("a vat id")),
        tax_registration: text("TAX-REG-9"),
        electronic_address: Some(ElectronicAddress {
            id: text("seller@example.de"),
            scheme: Some(ElectronicAddressScheme::Email),
        }),
        address: Some(address("DE", "Main street 1")),
        contact: Some(Contact {
            name: text("Anna Seller"),
            telephone: text("+49 30 1234"),
            email: Some("anna@example.de".parse().expect("an email")),
        }),
    }
}

fn buyer() -> Buyer {
    Buyer {
        name: text("Buyer Official Name"),
        trading_name: text("BuyerTrading"),
        identifiers: vec![OperationalEntity {
            id: text("BUYER-ID-1"),
            issuer: None,
        }],
        legal_entity: Some(LegalEntity {
            id: text("FR98765"),
            issuer: None,
        }),
        vat: Some(VatIdentifier::build(country("FR"), "12345678901").expect("a vat id")),
        electronic_address: Some(ElectronicAddress {
            id: text("buyer@example.fr"),
            scheme: Some(ElectronicAddressScheme::Email),
        }),
        address: Some(address("FR", "Rue centrale 2")),
        contact: Some(Contact {
            name: text("Bob Buyer"),
            telephone: text("+33 1 9876"),
            email: Some("bob@example.fr".parse().expect("an email")),
        }),
    }
}

fn payee() -> Payee {
    Payee {
        name: text("Payee Name"),
        identifiers: vec![OperationalEntity {
            id: text("PAYEE-ID"),
            issuer: None,
        }],
        legal_entity: Some(LegalEntity {
            id: text("PAYEE-LE"),
            issuer: None,
        }),
    }
}

fn tax_representative() -> TaxRepresentative {
    TaxRepresentative {
        name: text("Tax Rep Name"),
        vat: Some(VatIdentifier::build(country("DE"), "999888777").expect("a vat id")),
        address: Some(address("DE", "Rep street 3")),
    }
}

fn delivery() -> Delivery {
    Delivery {
        name: text("Delivery Party"),
        location: Some(LocationReference {
            id: text("LOC-1"),
            issuer: None,
        }),
        date: Some(date(2026, Month::January, 20)),
        address: Some(address("DE", "Delivery street 4")),
    }
}

fn payment() -> PaymentInstructions {
    PaymentInstructions {
        means: Some(PaymentMeans::CreditTransfer),
        means_text: text("Credit transfer"),
        remittance_information: text("PAY-REF-1"),
        details: Some(PaymentDetails::CreditTransfers(vec![CreditTransfer {
            account: Some("DE89370400440532013000".parse().expect("an account")),
            account_name: text("Seller Account"),
            provider: Some("DEUTDEFF".parse().expect("a bic")),
        }])),
    }
}

fn allowance() -> Adjustment {
    Adjustment {
        amount: Some(AdjustmentAmount::Absolute(Decimal::new(1000, 2))),
        vat: Some(VatTreatment::Standard { rate: rate(19) }),
        reason: Some(AdjustmentReason::Allowance {
            code: Some(AllowanceReason::Discount),
            text: text("Loyal customer"),
        }),
    }
}

fn supporting_document() -> SupportingDocument {
    SupportingDocument {
        reference: text("DOC-REF-1"),
        description: text("Supporting document"),
        external_location: Some("https://example.com/doc.pdf".parse().expect("a url")),
        attachment: None,
    }
}

// A standard-rated line of `quantity` units at `price` cents each, identified by `id`
// and stating a net amount of `net` cents.
// Only CII carries a gross price without a price discount, so only its fixture states one.
fn line(binding: Binding, id: &str, quantity: i64, price: i64, net: i64) -> InvoiceLine {
    InvoiceLine {
        id: text(id),
        note: text("line note"),
        object: None,
        quantity: Some(units(quantity)),
        net_amount: Some(Decimal::new(net, 2)),
        order_line_reference: text("OL-1"),
        buyer_accounting_reference: text("LINE-ACC-1"),
        period: Some(Period::Range {
            start: date(2026, Month::January, 1),
            end: date(2026, Month::January, 31),
        }),
        adjustments: vec![LineAdjustment {
            amount: Some(AdjustmentAmount::Absolute(Decimal::new(200, 2))),
            reason: Some(AdjustmentReason::Allowance {
                code: None,
                text: text("line discount"),
            }),
        }],
        price: Some(Price {
            net: Some(Decimal::new(price, 2)),
            gross: match binding {
                Binding::Cii => Some(Decimal::new(price, 2)),
                Binding::Ubl => None,
            },
            ..Default::default()
        }),
        vat: Some(VatTreatment::Standard { rate: rate(19) }),
        item: Some(Item {
            name: text("Item name"),
            description: text("Item description"),
            seller_id: text("SELLER-ITEM-1"),
            buyer_id: text("BUYER-ITEM-1"),
            standard_id: Some(ItemReference {
                id: text("1234567890128"),
                issuer: Some("0088".parse::<IssuingAgency>().expect("an agency")),
            }),
            classifications: vec![ItemClassification {
                id: text("65434"),
                scheme: Some(ItemClassificationScheme::MutuallyDefined),
                version: None,
            }],
            country_of_origin: Some(country("DE")),
            attributes: vec![ItemAttribute {
                name: text("Color"),
                value: text("Blue"),
            }],
        }),
    }
}

fn address(code: &str, street: &str) -> PostalAddress {
    PostalAddress {
        line1: text(street),
        line2: text("Building A"),
        line3: text("Floor 2"),
        city: text("Berlin"),
        country: Some(country(code)),
        country_subdivision: text("Berlin region"),
        postal_code: text("10115"),
    }
}

/// Indents compact serializer output for readable golden fixtures.
///
/// Newlines and indentation go only between tags, so leaf text stays on its own
/// line and the parser (which drops whitespace-only nodes) reads the result back
/// unchanged. It is the exact form each committed `*.xml` fixture is stored in.
pub fn pretty(xml: &str) -> String {
    #[derive(PartialEq)]
    enum Prev {
        None,
        Open,
        Text,
        Close,
    }
    let mut out = String::new();
    let mut depth: usize = 0;
    let mut prev = Prev::None;
    let mut rest = xml;
    while !rest.is_empty() {
        if rest.starts_with('<') {
            let end = rest.find('>').expect("a closed tag");
            let tag = &rest[..=end];
            if tag.starts_with("</") {
                depth -= 1;
                if prev == Prev::Text {
                    out.push_str(tag);
                } else {
                    out.push('\n');
                    out.push_str(&"  ".repeat(depth));
                    out.push_str(tag);
                }
                prev = Prev::Close;
            } else {
                if prev != Prev::None {
                    out.push('\n');
                    out.push_str(&"  ".repeat(depth));
                }
                out.push_str(tag);
                if tag.ends_with("/>") {
                    prev = Prev::Close;
                } else {
                    depth += 1;
                    prev = Prev::Open;
                }
            }
            rest = &rest[end + 1..];
        } else {
            let next = rest.find('<').unwrap_or(rest.len());
            out.push_str(&rest[..next]);
            prev = Prev::Text;
            rest = &rest[next..];
        }
    }
    out.push('\n');
    out
}

// ---- report locations and their bindings --------------------------------

/// An address of the steps a dialect abbreviated.
pub fn location(steps: &[(&str, &str, usize)]) -> Location {
    Location {
        steps: steps
            .iter()
            .map(|(abbreviation, name, index)| abbreviated(abbreviation, name, *index))
            .collect(),
    }
}

/// A step whose namespace a dialect abbreviated.
pub fn abbreviated(abbreviation: &str, name: &str, index: usize) -> LocationStep {
    LocationStep {
        namespace: Some(RawNamespace::Abbreviation(abbreviation.to_owned())),
        name: name.to_owned(),
        index: NonZeroUsize::new(index).expect("a positive index"),
    }
}

/// A step whose namespace a dialect wrote in full.
pub fn resolved(uri: &str, name: &str, index: usize) -> LocationStep {
    LocationStep {
        namespace: Some(RawNamespace::Uri(uri.to_owned())),
        name: name.to_owned(),
        index: NonZeroUsize::new(index).expect("a positive index"),
    }
}

/// A trailing attribute step, which belongs to no namespace.
pub fn attribute(name: &str) -> LocationStep {
    LocationStep {
        namespace: None,
        name: name.to_owned(),
        index: NonZeroUsize::new(1).expect("a positive index"),
    }
}

/// A context of the given model segments.
pub fn context(segments: Vec<Segment>) -> Context {
    Context { segments }
}

/// A single non-indexed field segment.
pub fn field(name: &'static str) -> Segment {
    Segment {
        field: name,
        index: None,
    }
}

/// A repeatable-group instance segment.
pub fn instance(name: &'static str, index: usize) -> Segment {
    Segment {
        field: name,
        index: NonZeroUsize::new(index),
    }
}

/// A finding of the given weight at the given address.
pub fn finding(severity: Severity, location: Location) -> Entry {
    Entry {
        severity,
        code: Some("BR-21".to_owned()),
        text: "each line needs an identifier".to_owned(),
        original_location: "/ubl:Invoice/cac:InvoiceLine[2]".to_owned(),
        normalized_location: Some(location),
    }
}

/// The context the document binds an address to, through the check of a single finding,
/// or `None` when no node answers the address.
pub fn bound<F: Format + Clone>(
    document: &Document<Invoice, F>,
    location: Location,
) -> Option<Context> {
    let report = RawReport {
        findings: vec![finding(Severity::Error, location)],
    };
    match document.clone().check(report) {
        Ok(Err(rejected)) => rejected
            .problems()
            .first()
            .map(|problem| problem.context.clone()),
        _ => None,
    }
}
