//! The interfaces of the semantic model: the invoice and every group of the standard.
//!
//! Each trait names the fields of one group. Each method hands out a unique reference
//! to one field, so the same method serves both the writing walk and the parsing walk.
//! The concrete types of the base standard live in the `en16931-cius` crate,
//! and an extension may implement the traits on types of its own.

use crate::{
    Adjustment, Amount, Currency, Date, Decimal, InvoiceKind, InvoiceReference, InvoiceType,
    NonEmptyString, Note, ObjectReference, PaymentInstructions, Period, SupportingDocument,
    VatBreakdown, VatPoint,
};

mod buyer;
mod contact;
mod delivery;
mod item;
mod line;
mod payee;
mod seller;
mod tax_representative;

pub use buyer::Buyer;
pub use contact::Contact;
pub use delivery::Delivery;
pub use item::Item;
pub use line::Line;
pub use payee::Payee;
pub use seller::Seller;
pub use tax_representative::TaxRepresentative;

/// The business facts a document can describe.
///
/// Every field but the type code is optional: the model checks the types of the values,
/// and the external validator checks the completeness of the document.
/// A profile decides on serialization which terms to forbid.
/// All amounts are in the invoice currency (`BT-5`).
/// The issuer states every amount: the library computes none of them,
/// and the external validator checks their consistency.
/// Regulatory-flow fields (`BT-23`, `BT-24`) do not live here but belong to the transport layer.
///
/// Reading through these methods needs exclusive access: a consumer takes the invoice
/// out of a `Document` by value and works on it as its own.
pub trait Invoice {
    /// The type of the seller (`BG-4`).
    type Seller: Seller;
    /// The type of the buyer (`BG-7`).
    type Buyer: Buyer;
    /// The type of the payee (`BG-10`).
    type Payee: Payee;
    /// The type of the seller tax representative (`BG-11`).
    type TaxRepresentative: TaxRepresentative;
    /// The type of the delivery information (`BG-13`).
    type Delivery: Delivery;
    /// The type of an invoice line (`BG-25`).
    type Line: Line;

    /// The kind of the invoice, a claim for a payment by default.
    /// The issuer states it by the business event, apart from the type code.
    fn kind(&mut self) -> &mut InvoiceKind;
    /// Invoice number (`BT-1`).
    fn number(&mut self) -> &mut Option<NonEmptyString>;
    /// Issue date (`BT-2`).
    fn issue_date(&mut self) -> &mut Option<Date>;
    /// Type code (`BT-3`), the commercial invoice by default.
    fn type_code(&mut self) -> &mut InvoiceType;
    /// Currency (`BT-5`).
    fn currency(&mut self) -> &mut Option<Currency>;
    /// VAT total in the accounting currency (`BT-111`+`BT-6`).
    fn vat_accounting_total(&mut self) -> &mut Option<Amount>;
    /// VAT point (`BT-7` date or `BT-8` code).
    fn vat_point(&mut self) -> &mut Option<VatPoint>;
    /// Payment due date (`BT-9`).
    fn payment_due_date(&mut self) -> &mut Option<Date>;
    /// Buyer reference (`BT-10`).
    fn buyer_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Project reference (`BT-11`).
    fn project_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Contract reference (`BT-12`).
    fn contract_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Purchase order reference (`BT-13`).
    fn purchase_order_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Sales order reference (`BT-14`).
    fn sales_order_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Receiving advice reference (`BT-15`).
    fn receiving_advice_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Despatch advice reference (`BT-16`).
    fn despatch_advice_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Tender or lot reference (`BT-17`).
    fn tender_or_lot_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Invoiced object identifier (`BT-18`).
    fn object(&mut self) -> &mut Option<ObjectReference>;
    /// Buyer accounting reference (`BT-19`).
    fn buyer_accounting_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Payment terms (`BT-20`).
    fn payment_terms(&mut self) -> &mut Option<NonEmptyString>;
    /// Invoicing period (`BG-14`).
    fn invoicing_period(&mut self) -> &mut Option<Period>;

    /// Sum of line net amounts (`BT-106`).
    fn line_net_total(&mut self) -> &mut Option<Decimal>;
    /// Sum of document-level allowances (`BT-107`).
    fn allowances_total(&mut self) -> &mut Option<Decimal>;
    /// Sum of document-level charges (`BT-108`).
    fn charges_total(&mut self) -> &mut Option<Decimal>;
    /// Total without VAT (`BT-109`).
    fn net_total(&mut self) -> &mut Option<Decimal>;
    /// Total VAT amount (`BT-110`).
    fn vat_total(&mut self) -> &mut Option<Decimal>;
    /// Total with VAT (`BT-112`).
    fn gross_total(&mut self) -> &mut Option<Decimal>;
    /// Paid amount (`BT-113`).
    fn paid(&mut self) -> &mut Option<Decimal>;
    /// Rounding amount (`BT-114`).
    fn rounding(&mut self) -> &mut Option<Decimal>;
    /// Amount due for payment (`BT-115`).
    fn due(&mut self) -> &mut Option<Decimal>;

    /// Notes (`BG-1`).
    fn notes(&mut self) -> &mut Vec<Note>;
    /// Preceding invoice references (`BG-3`).
    fn preceding_invoices(&mut self) -> &mut Vec<InvoiceReference>;
    /// Seller (`BG-4`).
    fn seller(&mut self) -> &mut Option<Self::Seller>;
    /// Buyer (`BG-7`).
    fn buyer(&mut self) -> &mut Option<Self::Buyer>;
    /// Payee (`BG-10`).
    fn payee(&mut self) -> &mut Option<Self::Payee>;
    /// Seller tax representative (`BG-11`).
    fn tax_representative(&mut self) -> &mut Option<Self::TaxRepresentative>;
    /// Delivery information (`BG-13`).
    fn delivery(&mut self) -> &mut Option<Self::Delivery>;
    /// Payment instructions (`BG-16`).
    fn payment(&mut self) -> &mut Option<PaymentInstructions>;
    /// Document-level allowances and charges (`BG-20`/`BG-21`).
    fn adjustments(&mut self) -> &mut Vec<Adjustment>;
    /// VAT breakdown (`BG-23`).
    fn vat_breakdown(&mut self) -> &mut Vec<VatBreakdown>;
    /// Additional supporting documents (`BG-24`).
    fn supporting_documents(&mut self) -> &mut Vec<SupportingDocument>;
    /// Invoice lines (`BG-25`).
    fn lines(&mut self) -> &mut Vec<Self::Line>;
}
