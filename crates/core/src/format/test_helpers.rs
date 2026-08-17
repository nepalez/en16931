//! Shared document fixtures and helpers for the binding tests.

use crate::prelude::*;
use crate::{
    Adjustment, AdjustmentAmount, AdjustmentReason, AllowanceReason, Binding, BusinessProcess,
    Buyer, Classification, Contact, CreditTransfer, Delivery, DirectDebit, DocumentBuilder,
    ElectronicAddress, ElectronicAddressScheme, Invoice, InvoiceLine, IssuingAgency, Item,
    ItemAttribute, ItemClassification, ItemReference, LegalEntity, LineAdjustment,
    LocationReference, Namespace, Note, ObjectReference, OperationalEntity, Path, Payee,
    PaymentDetails, PaymentInstructions, PaymentMeans, Percentage, Period, PostalAddress,
    PrecedingInvoice, Price, Profile, Quantity, Seller, Step, SupportingDocument,
    TaxRepresentative, Unit, VatIdentifier, VatPoint, VatTreatment,
};

// Builds a non-empty string, panicking on an empty input.
fn text(value: &str) -> crate::NonEmptyString {
    value.parse().expect("a non-empty string")
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

/// A rich `DocumentBuilder` under the base EN-16931 profile, shared by the tests.
///
/// The base profile forbids no term, so it serializes and parses back unchanged.
/// The `binding` is a parameter so each binding test compares against a coherent
/// fixture, since a parsed document carries the binding of the XML it was read from.
pub(crate) fn builder(binding: Binding) -> DocumentBuilder {
    DocumentBuilder {
        invoice: invoice(),
        profile: Profile::En16931,
        binding,
        business_process: Some(BusinessProcess::PEPPOL_BILLING),
    }
}

/// A `DocumentBuilder` exercising the alternative branches the rich fixture omits.
///
/// It flips the mutually-exclusive choices `builder` never reaches: a direct-debit
/// payment, an event VAT point, relative and charge adjustments, an exempt line, an
/// object scheme, a price discount, and start-only or end-only periods. Every choice
/// survives both bindings, so each codec parses the document back unchanged.
pub(crate) fn variant_builder(binding: Binding) -> DocumentBuilder {
    DocumentBuilder {
        invoice: variant_invoice(),
        profile: Profile::En16931,
        binding,
        business_process: Some(BusinessProcess::PEPPOL_BILLING),
    }
}

// The base invoice with its alternative-branch fields overridden.
fn variant_invoice() -> Invoice {
    Invoice {
        payment: Some(variant_payment()),
        vat_point: Some(VatPoint::try_from(35u16).expect("a vat point event")),
        object: Some(ObjectReference {
            id: text("OBJ-200"),
            scheme: Some("AAA".parse().expect("an object type")),
        }),
        adjustments: vec![variant_allowance(), variant_charge()],
        invoicing_period: Some(Period::Until(date(2026, Month::January, 31))),
        lines: vec![variant_line(), line("2", 1, 5000)],
        ..invoice()
    }
}

// A direct-debit payment carrying a mandate, a creditor, and a debited account.
fn variant_payment() -> PaymentInstructions {
    PaymentInstructions {
        means: PaymentMeans::SepaDirectDebit,
        means_text: Some(text("Direct debit")),
        remittance_information: Some(text("DD-REF-9")),
        details: Some(PaymentDetails::DirectDebit(DirectDebit {
            mandate_reference: Some(text("MANDATE-7")),
            creditor_identifier: Some(text("DE98ZZZ09999999999")),
            debited_account: Some("DE89370400440532013000".parse().expect("an account")),
        })),
    }
}

// A relative document-level allowance, a percentage of a base.
fn variant_allowance() -> Adjustment {
    Adjustment {
        amount: AdjustmentAmount::Relative {
            rate: rate(10),
            base: Decimal::new(20000, 2),
        },
        vat: VatTreatment::Standard { rate: rate(19) },
        reason: AdjustmentReason::Allowance {
            code: Some(AllowanceReason::Discount),
            text: Some(text("Volume discount")),
        },
    }
}

// A document-level charge with a reason code.
fn variant_charge() -> Adjustment {
    Adjustment {
        amount: AdjustmentAmount::Absolute(Decimal::new(1500, 2)),
        vat: VatTreatment::Standard { rate: rate(19) },
        reason: AdjustmentReason::Charge {
            code: Some("AA".parse().expect("a charge reason")),
            text: Some(text("Advertising")),
        },
    }
}

// An exempt line carrying a price discount, a base quantity, a line object,
// and both a relative allowance and a charge at the line level.
fn variant_line() -> InvoiceLine {
    InvoiceLine {
        id: text("1"),
        note: None,
        object: Some(ObjectReference {
            id: text("LINE-OBJ-1"),
            scheme: Some("AAB".parse().expect("an object type")),
        }),
        quantity: Quantity {
            unit: Unit::from_code("C62").expect("a unit"),
            value: Decimal::from(3),
        },
        order_line_reference: None,
        buyer_accounting_reference: None,
        period: Some(Period::From(date(2026, Month::January, 1))),
        adjustments: vec![
            LineAdjustment {
                amount: AdjustmentAmount::Relative {
                    rate: rate(5),
                    base: Decimal::new(30000, 2),
                },
                reason: AdjustmentReason::Allowance {
                    code: None,
                    text: Some(text("line rebate")),
                },
            },
            LineAdjustment {
                amount: AdjustmentAmount::Absolute(Decimal::new(400, 2)),
                reason: AdjustmentReason::Charge {
                    code: Some("AA".parse().expect("a charge reason")),
                    text: Some(text("line handling")),
                },
            },
        ],
        price: Price {
            gross: Decimal::new(12000, 2),
            discount: Some(Decimal::new(2000, 2)),
            base_quantity: Some(Quantity {
                unit: Unit::from_code("C62").expect("a unit"),
                value: Decimal::from(1),
            }),
        },
        vat: VatTreatment::Exempt {
            code: Some("VATEX-EU-132".parse().expect("an exemption reason")),
            text: Some(text("Exempt supply")),
        },
        item: Item {
            name: text("Exempt item"),
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

// The invoice the fixture carries.
fn invoice() -> Invoice {
    Invoice {
        number: text("INV-2026-001"),
        issue_date: date(2026, Month::January, 15),
        type_code: "380".parse().expect("a type code"),
        currency: Currency::EUR,
        vat_accounting_total: None,
        vat_point: None,
        payment_due_date: Some(date(2026, Month::February, 15)),
        buyer_reference: Some(text("BUYER-REF-01")),
        project_reference: Some(text("PROJECT-42")),
        contract_reference: Some(text("CONTRACT-7")),
        purchase_order_reference: Some(text("PO-2026-9")),
        sales_order_reference: Some(text("SO-2026-3")),
        receiving_advice_reference: Some(text("RECADV-2")),
        despatch_advice_reference: Some(text("DESADV-2")),
        tender_or_lot_reference: Some(text("TENDER-1")),
        object: Some(ObjectReference {
            id: text("OBJ-100"),
            scheme: None,
        }),
        buyer_accounting_reference: Some(text("ACCOUNT-500")),
        payment_terms: Some(text("Net 30 days")),
        notes: vec![Note {
            subject_code: Some(text("AAB")),
            text: text("General note text"),
        }],
        preceding_invoices: vec![PrecedingInvoice {
            number: text("INV-2025-900"),
            issue_date: Some(date(2025, Month::December, 1)),
        }],
        seller: seller(),
        buyer: buyer(),
        payee: Some(payee()),
        tax_representative: Some(tax_representative()),
        delivery: Some(delivery()),
        invoicing_period: Some(Period::Range {
            start: date(2026, Month::January, 1),
            end: date(2026, Month::January, 31),
        }),
        adjustments: vec![allowance()],
        rounding: Some(Decimal::new(-3, 2)),
        payment: Some(payment()),
        paid: Some(Decimal::new(5000, 2)),
        supporting_documents: vec![supporting_document()],
        lines: vec![line("1", 2, 10000), line("2", 1, 5000)],
    }
}

fn seller() -> Seller {
    Seller {
        name: text("Seller Official Name Ltd"),
        trading_name: Some(text("SellerTrading")),
        identifiers: vec![OperationalEntity {
            id: text("SELLER-ID-1"),
            issuer: Some("0088".parse::<IssuingAgency>().expect("an agency")),
        }],
        legal_entity: Some(LegalEntity {
            id: text("DE12345"),
            issuer: None,
        }),
        additional_legal_information: Some(text("Registered in Berlin")),
        vat: Some(VatIdentifier::build(country("DE"), "123456789").expect("a vat id")),
        tax_registration: Some(text("TAX-REG-9")),
        electronic_address: Some(ElectronicAddress {
            id: text("seller@example.de"),
            scheme: ElectronicAddressScheme::Email,
        }),
        address: address("DE", "Main street 1"),
        contact: Some(Contact {
            name: Some(text("Anna Seller")),
            telephone: Some(text("+49 30 1234")),
            email: Some("anna@example.de".parse().expect("an email")),
        }),
    }
}

fn buyer() -> Buyer {
    Buyer {
        name: text("Buyer Official Name"),
        trading_name: Some(text("BuyerTrading")),
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
            scheme: ElectronicAddressScheme::Email,
        }),
        address: address("FR", "Rue centrale 2"),
        contact: Some(Contact {
            name: Some(text("Bob Buyer")),
            telephone: Some(text("+33 1 9876")),
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
        vat: VatIdentifier::build(country("DE"), "999888777").expect("a vat id"),
        address: address("DE", "Rep street 3"),
    }
}

fn delivery() -> Delivery {
    Delivery {
        name: Some(text("Delivery Party")),
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
        means: PaymentMeans::CreditTransfer,
        means_text: Some(text("Credit transfer")),
        remittance_information: Some(text("PAY-REF-1")),
        details: Some(PaymentDetails::CreditTransfers(vec![CreditTransfer {
            account: "DE89370400440532013000".parse().expect("an account"),
            account_name: Some(text("Seller Account")),
            provider: Some("DEUTDEFF".parse().expect("a bic")),
        }])),
    }
}

fn allowance() -> Adjustment {
    Adjustment {
        amount: AdjustmentAmount::Absolute(Decimal::new(1000, 2)),
        vat: VatTreatment::Standard { rate: rate(19) },
        reason: AdjustmentReason::Allowance {
            code: Some(AllowanceReason::Discount),
            text: Some(text("Loyal customer")),
        },
    }
}

fn supporting_document() -> SupportingDocument {
    SupportingDocument {
        reference: text("DOC-REF-1"),
        description: Some(text("Supporting document")),
        external_location: Some("https://example.com/doc.pdf".parse().expect("a url")),
        attachment: None,
    }
}

// A standard-rated line of `quantity` units at `price` cents each, identified by `id`.
fn line(id: &str, quantity: i64, price: i64) -> InvoiceLine {
    InvoiceLine {
        id: text(id),
        note: Some(text("line note")),
        object: None,
        quantity: Quantity {
            unit: Unit::from_code("C62").expect("a unit"),
            value: Decimal::from(quantity),
        },
        order_line_reference: Some(text("OL-1")),
        buyer_accounting_reference: Some(text("LINE-ACC-1")),
        period: Some(Period::Range {
            start: date(2026, Month::January, 1),
            end: date(2026, Month::January, 31),
        }),
        adjustments: vec![LineAdjustment {
            amount: AdjustmentAmount::Absolute(Decimal::new(200, 2)),
            reason: AdjustmentReason::Allowance {
                code: None,
                text: Some(text("line discount")),
            },
        }],
        price: Price {
            gross: Decimal::new(price, 2),
            discount: None,
            base_quantity: None,
        },
        vat: VatTreatment::Standard { rate: rate(19) },
        item: Item {
            name: text("Item name"),
            description: Some(text("Item description")),
            seller_id: Some(text("SELLER-ITEM-1")),
            buyer_id: Some(text("BUYER-ITEM-1")),
            standard_id: Some(ItemReference {
                id: text("1234567890128"),
                issuer: "0088".parse::<IssuingAgency>().expect("an agency"),
            }),
            classifications: vec![Classification {
                id: text("65434"),
                scheme: ItemClassification::MutuallyDefined,
                version: None,
            }],
            country_of_origin: Some(country("DE")),
            attributes: vec![ItemAttribute {
                name: text("Color"),
                value: text("Blue"),
            }],
        },
    }
}

fn address(code: &str, street: &str) -> PostalAddress {
    PostalAddress {
        line1: Some(text(street)),
        line2: Some(text("Building A")),
        line3: Some(text("Floor 2")),
        city: Some(text("Berlin")),
        country: country(code),
        country_subdivision: Some(text("Berlin region")),
        postal_code: Some(text("10115")),
    }
}

/// A record-form step with a 1-based positional index, for path assertions.
pub(crate) fn step(namespace: Namespace, name: &str, index: usize) -> Step {
    Step {
        namespace,
        name: name.to_owned(),
        index: NonZeroUsize::new(index).expect("a positive index"),
    }
}

/// A record-form path from its steps, for dictionary lookups in tests.
pub(crate) fn path(steps: Vec<Step>) -> Path {
    Path { steps }
}

/// Indents compact serializer output for readable golden fixtures.
///
/// Newlines and indentation go only between tags, so leaf text stays on its own
/// line and the parser (which drops whitespace-only nodes) reads the result back
/// unchanged. It is the exact form each committed `*.xml` fixture is stored in.
pub(crate) fn pretty(xml: &str) -> String {
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
