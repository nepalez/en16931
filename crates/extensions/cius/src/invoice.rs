//! The base invoice: the root of the semantic model.

use crate::prelude::*;
use crate::{Buyer, Delivery, InvoiceLine, Payee, Seller, TaxRepresentative};

/// This object carries business facts an invoice can describe.
///
/// Every field but the type code is optional: the model checks the types of the values,
/// and the external validator checks the completeness of the document.
///
/// All amounts are in the invoice currency (`BT-5`).
/// Regulatory-flow fields (`BT-23`, `BT-24`) do not live here but belong to the transport layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Invoice {
    /// The kind of the invoice, a claim for a payment by default.
    pub kind: InvoiceKind,
    /// Invoice number (`BT-1`).
    pub number: Option<NonEmptyString>,
    /// Issue date (`BT-2`).
    pub issue_date: Option<Date>,
    /// Type code (`BT-3`), the commercial invoice by default.
    pub type_code: InvoiceType,
    /// Currency (`BT-5`).
    pub currency: Option<Currency>,
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
    pub preceding_invoices: Vec<InvoiceReference>,
    /// Seller (`BG-4`).
    pub seller: Option<Seller>,
    /// Buyer (`BG-7`).
    pub buyer: Option<Buyer>,
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
    /// Sum of line net amounts (`BT-106`).
    pub line_net_total: Option<Decimal>,
    /// Sum of document-level allowances (`BT-107`).
    pub allowances_total: Option<Decimal>,
    /// Sum of document-level charges (`BT-108`).
    pub charges_total: Option<Decimal>,
    /// Total without VAT (`BT-109`).
    pub net_total: Option<Decimal>,
    /// Total VAT amount (`BT-110`).
    pub vat_total: Option<Decimal>,
    /// Total with VAT (`BT-112`).
    pub gross_total: Option<Decimal>,
    /// Rounding amount (`BT-114`).
    pub rounding: Option<Decimal>,
    /// Payment instructions (`BG-16`).
    pub payment: Option<PaymentInstructions>,
    /// Paid amount (`BT-113`).
    pub paid: Option<Decimal>,
    /// Amount due for payment (`BT-115`).
    pub due: Option<Decimal>,
    /// VAT breakdown (`BG-23`).
    pub vat_breakdown: Vec<VatBreakdown>,
    /// Additional supporting documents (`BG-24`).
    pub supporting_documents: Vec<SupportingDocument>,
    /// Invoice lines (`BG-25`).
    pub lines: Vec<InvoiceLine>,
}

impl crate::prelude::Invoice for Invoice {
    type Seller = Seller;
    type Buyer = Buyer;
    type Payee = Payee;
    type TaxRepresentative = TaxRepresentative;
    type Delivery = Delivery;
    type Line = InvoiceLine;

    fn kind(&mut self) -> &mut InvoiceKind {
        &mut self.kind
    }

    fn number(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.number
    }

    fn issue_date(&mut self) -> &mut Option<Date> {
        &mut self.issue_date
    }

    fn type_code(&mut self) -> &mut InvoiceType {
        &mut self.type_code
    }

    fn currency(&mut self) -> &mut Option<Currency> {
        &mut self.currency
    }

    fn vat_accounting_total(&mut self) -> &mut Option<Amount> {
        &mut self.vat_accounting_total
    }

    fn vat_point(&mut self) -> &mut Option<VatPoint> {
        &mut self.vat_point
    }

    fn payment_due_date(&mut self) -> &mut Option<Date> {
        &mut self.payment_due_date
    }

    fn buyer_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.buyer_reference
    }

    fn project_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.project_reference
    }

    fn contract_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.contract_reference
    }

    fn purchase_order_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.purchase_order_reference
    }

    fn sales_order_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.sales_order_reference
    }

    fn receiving_advice_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.receiving_advice_reference
    }

    fn despatch_advice_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.despatch_advice_reference
    }

    fn tender_or_lot_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.tender_or_lot_reference
    }

    fn object(&mut self) -> &mut Option<ObjectReference> {
        &mut self.object
    }

    fn buyer_accounting_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.buyer_accounting_reference
    }

    fn payment_terms(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.payment_terms
    }

    fn invoicing_period(&mut self) -> &mut Option<Period> {
        &mut self.invoicing_period
    }

    fn line_net_total(&mut self) -> &mut Option<Decimal> {
        &mut self.line_net_total
    }

    fn allowances_total(&mut self) -> &mut Option<Decimal> {
        &mut self.allowances_total
    }

    fn charges_total(&mut self) -> &mut Option<Decimal> {
        &mut self.charges_total
    }

    fn net_total(&mut self) -> &mut Option<Decimal> {
        &mut self.net_total
    }

    fn vat_total(&mut self) -> &mut Option<Decimal> {
        &mut self.vat_total
    }

    fn gross_total(&mut self) -> &mut Option<Decimal> {
        &mut self.gross_total
    }

    fn paid(&mut self) -> &mut Option<Decimal> {
        &mut self.paid
    }

    fn rounding(&mut self) -> &mut Option<Decimal> {
        &mut self.rounding
    }

    fn due(&mut self) -> &mut Option<Decimal> {
        &mut self.due
    }

    fn notes(&mut self) -> &mut Vec<Note> {
        &mut self.notes
    }

    fn preceding_invoices(&mut self) -> &mut Vec<InvoiceReference> {
        &mut self.preceding_invoices
    }

    fn seller(&mut self) -> &mut Option<Seller> {
        &mut self.seller
    }

    fn buyer(&mut self) -> &mut Option<Buyer> {
        &mut self.buyer
    }

    fn payee(&mut self) -> &mut Option<Payee> {
        &mut self.payee
    }

    fn tax_representative(&mut self) -> &mut Option<TaxRepresentative> {
        &mut self.tax_representative
    }

    fn delivery(&mut self) -> &mut Option<Delivery> {
        &mut self.delivery
    }

    fn payment(&mut self) -> &mut Option<PaymentInstructions> {
        &mut self.payment
    }

    fn adjustments(&mut self) -> &mut Vec<Adjustment> {
        &mut self.adjustments
    }

    fn vat_breakdown(&mut self) -> &mut Vec<VatBreakdown> {
        &mut self.vat_breakdown
    }

    fn supporting_documents(&mut self) -> &mut Vec<SupportingDocument> {
        &mut self.supporting_documents
    }

    fn lines(&mut self) -> &mut Vec<InvoiceLine> {
        &mut self.lines
    }
}
