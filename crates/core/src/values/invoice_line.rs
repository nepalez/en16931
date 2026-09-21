use crate::{
    Decimal, Item, LineAdjustment, NonEmptyString, ObjectReference, Period, Price, Quantity,
    VatTreatment,
};

/// An invoice line (`BG-25`): one charged position of the invoice.
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
