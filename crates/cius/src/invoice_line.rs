//! The invoice line of the base invoice (`BG-25`).

use crate::Item;
use crate::prelude::*;

/// One charged position of the invoice (`BG-25`).
/// All amounts here are in the invoice currency (`BT-5`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvoiceLine {
    /// Line identifier (`BT-126`).
    pub id: Option<NonEmptyString>,
    /// Line note (`BT-127`).
    pub note: Option<NonEmptyString>,
    /// Object identifier (`BT-128`).
    pub object: Option<ObjectReference>,
    /// Invoiced quantity (`BT-129`+`BT-130`).
    pub quantity: Option<Quantity>,
    /// Line net amount (`BT-131`).
    pub net_amount: Option<Decimal>,
    /// Referenced purchase order line reference (`BT-132`).
    pub order_line_reference: Option<NonEmptyString>,
    /// Buyer accounting reference (`BT-133`).
    pub buyer_accounting_reference: Option<NonEmptyString>,
    /// Line period (`BG-26`).
    pub period: Option<Period>,
    /// Line allowances and charges (`BG-27`/`BG-28`).
    pub adjustments: Vec<LineAdjustment>,
    /// Price details (`BG-29`).
    pub price: Option<Price>,
    /// Line VAT treatment (`BG-30`).
    pub vat: Option<VatTreatment>,
    /// Item information (`BG-31`).
    pub item: Option<Item>,
}

impl Line for InvoiceLine {
    type Item = Item;

    fn id(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.id
    }

    fn note(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.note
    }

    fn object(&mut self) -> &mut Option<ObjectReference> {
        &mut self.object
    }

    fn quantity(&mut self) -> &mut Option<Quantity> {
        &mut self.quantity
    }

    fn net_amount(&mut self) -> &mut Option<Decimal> {
        &mut self.net_amount
    }

    fn order_line_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.order_line_reference
    }

    fn buyer_accounting_reference(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.buyer_accounting_reference
    }

    fn period(&mut self) -> &mut Option<Period> {
        &mut self.period
    }

    fn adjustments(&mut self) -> &mut Vec<LineAdjustment> {
        &mut self.adjustments
    }

    fn price(&mut self) -> &mut Option<Price> {
        &mut self.price
    }

    fn vat(&mut self) -> &mut Option<VatTreatment> {
        &mut self.vat
    }

    fn item(&mut self) -> &mut Option<Item> {
        &mut self.item
    }
}
