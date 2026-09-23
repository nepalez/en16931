//! The interface of an invoice line (`BG-25`).

use crate::{
    Decimal, Item, LineAdjustment, NonEmptyString, ObjectReference, Period, Price, Quantity,
    VatTreatment,
};

/// One charged position of the invoice (`BG-25`).
/// All amounts here are in the invoice currency (`BT-5`).
pub trait Line {
    /// The type of the item information (`BG-31`).
    type Item: Item;

    /// Line identifier (`BT-126`).
    fn id(&mut self) -> &mut Option<NonEmptyString>;
    /// Line note (`BT-127`).
    fn note(&mut self) -> &mut Option<NonEmptyString>;
    /// Object identifier (`BT-128`).
    fn object(&mut self) -> &mut Option<ObjectReference>;
    /// Invoiced quantity (`BT-129`+`BT-130`).
    fn quantity(&mut self) -> &mut Option<Quantity>;
    /// Line net amount (`BT-131`).
    fn net_amount(&mut self) -> &mut Option<Decimal>;
    /// Referenced purchase order line reference (`BT-132`).
    fn order_line_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Buyer accounting reference (`BT-133`).
    fn buyer_accounting_reference(&mut self) -> &mut Option<NonEmptyString>;
    /// Line period (`BG-26`).
    fn period(&mut self) -> &mut Option<Period>;
    /// Line allowances and charges (`BG-27`/`BG-28`).
    fn adjustments(&mut self) -> &mut Vec<LineAdjustment>;
    /// Price details (`BG-29`).
    fn price(&mut self) -> &mut Option<Price>;
    /// Line VAT treatment (`BG-30`).
    fn vat(&mut self) -> &mut Option<VatTreatment>;
    /// Item information (`BG-31`).
    fn item(&mut self) -> &mut Option<Self::Item>;
}
