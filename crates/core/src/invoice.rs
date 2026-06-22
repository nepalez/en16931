pub mod adjustment;
pub mod buyer;
pub mod classification;
pub mod contact;
pub mod credit_transfer;
pub mod delivery;
pub mod direct_debit;
pub mod electronic_address;
pub mod invoice_line;
pub mod item;
pub mod item_attribute;
pub mod item_reference;
pub mod legal_entity;
pub mod location_reference;
pub mod note;
pub mod object_reference;
pub mod operational_entity;
pub mod payee;
pub mod payment_card;
pub mod payment_instructions;
pub mod postal_address;
pub mod preceding_invoice;
pub mod price;
pub mod seller;
pub mod supporting_document;
pub mod tax_representative;
pub mod vat_treatment;

pub use adjustment::{
    Adjustment, Amount as AdjustmentAmount, LineAdjustment, Reason as AdjustmentReason,
};
pub use buyer::Buyer;
pub use classification::Classification;
pub use contact::Contact;
pub use credit_transfer::CreditTransfer;
pub use delivery::Delivery;
pub use direct_debit::DirectDebit;
pub use electronic_address::ElectronicAddress;
pub use invoice_line::InvoiceLine;
pub use item::Item;
pub use item_attribute::ItemAttribute;
pub use item_reference::ItemReference;
pub use legal_entity::LegalEntity;
pub use location_reference::LocationReference;
pub use note::Note;
pub use object_reference::ObjectReference;
pub use operational_entity::OperationalEntity;
pub use payee::Payee;
pub use payment_card::PaymentCard;
pub use payment_instructions::{Details as PaymentDetails, PaymentInstructions};
pub use postal_address::PostalAddress;
pub use preceding_invoice::PrecedingInvoice;
pub use price::Price;
pub use seller::Seller;
pub use supporting_document::SupportingDocument;
pub use tax_representative::TaxRepresentative;
pub use vat_treatment::VatTreatment;

use crate::prelude::*;
use crate::{
    Amount, Currency, Date, Decimal, InvoiceType, NonEmptyString, Period, VatCategory, VatPoint,
};

/// Rounds a derived money amount to two decimals, half away from zero.
pub(crate) fn rounded(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// This object carries business facts an invoice can describe.
///
/// Non-core fields are optional, and a profile decides on serialization which to require or forbid.
/// All amounts are in the invoice currency (`BT-5`).
/// Regulatory-flow fields (`BT-23`, `BT-24`) do not live here but belong to the transport layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invoice {
    /// Invoice number (`BT-1`).
    pub number: NonEmptyString,
    /// Issue date (`BT-2`).
    pub issue_date: Date,
    /// Type code (`BT-3`).
    pub type_code: InvoiceType,
    /// Currency (`BT-5`).
    pub currency: Currency,
    /// VAT total in the accounting currency (`BT-111`+`BT-6`).
    pub vat_accounting_total: Option<Amount>,
    /// VAT point (`BT-7` date or `BT-8` code).
    pub vat_point: Option<VatPoint>,
    /// Payment due date (`BT-9`).
    pub payment_due_date: Option<Date>,
    /// Buyer reference (`BT-10`).
    pub buyer_reference: Option<NonEmptyString>,
    /// Project reference (`BT-11`).
    pub project_reference: Option<NonEmptyString>,
    /// Contract reference (`BT-12`).
    pub contract_reference: Option<NonEmptyString>,
    /// Purchase order reference (`BT-13`).
    pub purchase_order_reference: Option<NonEmptyString>,
    /// Sales order reference (`BT-14`).
    pub sales_order_reference: Option<NonEmptyString>,
    /// Receiving advice reference (`BT-15`).
    pub receiving_advice_reference: Option<NonEmptyString>,
    /// Despatch advice reference (`BT-16`).
    pub despatch_advice_reference: Option<NonEmptyString>,
    /// Tender or lot reference (`BT-17`).
    pub tender_or_lot_reference: Option<NonEmptyString>,
    /// Invoiced object identifier (`BT-18`).
    pub object: Option<ObjectReference>,
    /// Buyer accounting reference (`BT-19`).
    pub buyer_accounting_reference: Option<NonEmptyString>,
    /// Payment terms (`BT-20`).
    pub payment_terms: Option<NonEmptyString>,
    /// Notes (`BG-1`).
    pub notes: Vec<Note>,
    /// Preceding invoice references (`BG-3`).
    pub preceding_invoices: Vec<PrecedingInvoice>,
    /// Seller (`BG-4`).
    pub seller: Seller,
    /// Buyer (`BG-7`).
    pub buyer: Buyer,
    /// Payee (`BG-10`).
    pub payee: Option<Payee>,
    /// Seller tax representative (`BG-11`).
    pub tax_representative: Option<TaxRepresentative>,
    /// Delivery information (`BG-13`).
    pub delivery: Option<Delivery>,
    /// Invoicing period (`BG-14`).
    pub invoicing_period: Option<Period>,
    /// Document-level allowances and charges (`BG-20`/`BG-21`).
    pub adjustments: Vec<Adjustment>,
    /// Rounding amount (`BT-114`).
    pub rounding: Option<Decimal>,
    /// Payment instructions (`BG-16`).
    pub payment: Option<PaymentInstructions>,
    /// Paid amount (`BT-113`).
    pub paid: Option<Decimal>,
    /// Additional supporting documents (`BG-24`).
    pub supporting_documents: Vec<SupportingDocument>,
    /// Invoice lines (`BG-25`).
    pub lines: Vec<InvoiceLine>,
}

impl Invoice {
    /// The sum of line net amounts (`BT-106`).
    pub fn line_net_total(&self) -> Decimal {
        self.lines.iter().map(InvoiceLine::net_amount).sum()
    }

    /// The sum of document-level allowances (`BT-107`).
    pub fn allowances_total(&self) -> Decimal {
        self.adjustments
            .iter()
            .filter(|adjustment| matches!(adjustment.reason, AdjustmentReason::Allowance { .. }))
            .map(|adjustment| adjustment.amount.value())
            .sum()
    }

    /// The sum of document-level charges (`BT-108`).
    pub fn charges_total(&self) -> Decimal {
        self.adjustments
            .iter()
            .filter(|adjustment| matches!(adjustment.reason, AdjustmentReason::Charge { .. }))
            .map(|adjustment| adjustment.amount.value())
            .sum()
    }

    /// The total without VAT (`BT-109`).
    pub fn net_total(&self) -> Decimal {
        self.line_net_total() - self.allowances_total() + self.charges_total()
    }

    /// The total VAT amount (`BT-110`): the sum of the per-category taxes, each rounded.
    pub fn vat_total(&self) -> Decimal {
        self.vat_groups()
            .into_iter()
            .map(|(_, rate, taxable)| rounded(taxable * rate / Decimal::from(100)))
            .sum()
    }

    /// The total with VAT (`BT-112`).
    pub fn gross_total(&self) -> Decimal {
        self.net_total() + self.vat_total()
    }

    /// The amount due for payment (`BT-115`).
    pub fn due(&self) -> Decimal {
        self.gross_total() - self.paid.unwrap_or(Decimal::ZERO)
            + self.rounding.unwrap_or(Decimal::ZERO)
    }

    // The taxable base per VAT category and rate: line nets plus document charges less document
    // allowances of that treatment. It underlies the per-category tax (`BT-116`/`BT-117`).
    fn vat_groups(&self) -> Vec<(VatCategory, Decimal, Decimal)> {
        let mut groups: Vec<(VatCategory, Decimal, Decimal)> = Vec::new();
        let mut accumulate = |vat: &VatTreatment, amount: Decimal| {
            let category = vat.category();
            let rate = vat.rate();
            match groups
                .iter_mut()
                .find(|(grouped, rated, _)| *grouped == category && *rated == rate)
            {
                Some((_, _, taxable)) => *taxable += amount,
                None => groups.push((category, rate, amount)),
            }
        };
        for line in &self.lines {
            accumulate(&line.vat, line.net_amount());
        }
        for adjustment in &self.adjustments {
            let signed = match adjustment.reason {
                AdjustmentReason::Charge { .. } => adjustment.amount.value(),
                AdjustmentReason::Allowance { .. } => -adjustment.amount.value(),
            };
            accumulate(&adjustment.vat, signed);
        }
        groups
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{CountryCode, Item, Percentage, Quantity, Unit};

    fn address() -> PostalAddress {
        PostalAddress {
            line1: None,
            line2: None,
            line3: None,
            city: None,
            country: CountryCode::for_alpha2("DE").expect("DE is a country"),
            country_subdivision: None,
            postal_code: None,
        }
    }

    fn line(gross: i64, rate: i64) -> InvoiceLine {
        InvoiceLine {
            id: "1".parse().expect("a valid id"),
            note: None,
            object: None,
            quantity: Quantity {
                unit: Unit::from_code("C62").expect("C62 is a unit"),
                value: Decimal::ONE,
            },
            order_line_reference: None,
            buyer_accounting_reference: None,
            period: None,
            adjustments: Vec::new(),
            price: Price {
                gross: Decimal::new(gross, 2),
                discount: None,
                base_quantity: None,
            },
            vat: VatTreatment::Standard {
                rate: Percentage::try_from(Decimal::from(rate)).expect("a valid rate"),
            },
            item: Item {
                name: "Item".parse().expect("a valid name"),
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

    fn invoice(lines: Vec<InvoiceLine>) -> Invoice {
        Invoice {
            number: "INV-1".parse().expect("a valid number"),
            issue_date: Date::from_ordinal_date(2026, 1).expect("a valid date"),
            type_code: "380".parse().expect("380 is an invoice type"),
            currency: Currency::EUR,
            vat_accounting_total: None,
            vat_point: None,
            payment_due_date: None,
            buyer_reference: None,
            project_reference: None,
            contract_reference: None,
            purchase_order_reference: None,
            sales_order_reference: None,
            receiving_advice_reference: None,
            despatch_advice_reference: None,
            tender_or_lot_reference: None,
            object: None,
            buyer_accounting_reference: None,
            payment_terms: None,
            notes: Vec::new(),
            preceding_invoices: Vec::new(),
            seller: Seller {
                name: "Seller".parse().expect("a valid name"),
                trading_name: None,
                identifiers: Vec::new(),
                legal_entity: None,
                additional_legal_information: None,
                vat: None,
                tax_registration: None,
                electronic_address: None,
                address: address(),
                contact: None,
            },
            buyer: Buyer {
                name: "Buyer".parse().expect("a valid name"),
                trading_name: None,
                identifiers: Vec::new(),
                legal_entity: None,
                vat: None,
                electronic_address: None,
                address: address(),
                contact: None,
            },
            payee: None,
            tax_representative: None,
            delivery: None,
            invoicing_period: None,
            adjustments: Vec::new(),
            rounding: None,
            payment: None,
            paid: None,
            supporting_documents: Vec::new(),
            lines,
        }
    }

    #[test]
    fn totals_the_net_vat_and_gross_across_vat_rates() {
        let invoice = invoice(vec![line(10000, 20), line(5000, 10)]);

        assert_eq!(invoice.net_total(), Decimal::new(15000, 2));
        // 100.00 * 20% + 50.00 * 10% = 20.00 + 5.00, each rounded per category
        assert_eq!(invoice.vat_total(), Decimal::new(2500, 2));
        assert_eq!(invoice.gross_total(), Decimal::new(17500, 2));
    }

    #[test]
    fn dues_the_gross_less_the_paid_plus_the_rounding() {
        let mut invoice = invoice(vec![line(10002, 20)]);
        invoice.paid = Some(Decimal::new(5000, 2));
        invoice.rounding = Some(Decimal::new(-2, 2));

        // gross 120.02 - paid 50.00 + rounding -0.02 = 70.00
        assert_eq!(invoice.due(), Decimal::new(7000, 2));
    }
}
