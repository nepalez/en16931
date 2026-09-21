use crate::{
    Adjustment, Amount, Buyer, Currency, Date, Decimal, Delivery, InvoiceLine, InvoiceType,
    NonEmptyString, Note, ObjectReference, Payee, PaymentInstructions, Period, PrecedingInvoice,
    Seller, SupportingDocument, TaxRepresentative, VatBreakdown, VatPoint,
};

/// This object carries business facts an invoice can describe.
///
/// Every field but the type code is optional: the model checks the types of the values,
/// and the external validator checks the completeness of the document.
/// A profile decides on serialization which terms to forbid.
/// All amounts are in the invoice currency (`BT-5`).
/// The issuer states every amount: the library computes none of them,
/// and the external validator checks their consistency.
/// Regulatory-flow fields (`BT-23`, `BT-24`) do not live here but belong to the transport layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Invoice {
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
    pub preceding_invoices: Vec<PrecedingInvoice>,
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
