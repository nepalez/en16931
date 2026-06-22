use crate::{
    Decimal, Item, LineAdjustment, NonEmptyString, ObjectReference, Period, Price, Quantity,
    VatTreatment,
};

/// An invoice line (`BG-25`): one charged position of the invoice.
/// All amounts here are in the invoice currency (`BT-5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceLine {
    /// Line identifier (`BT-126`).
    pub id: NonEmptyString,
    /// Line note (`BT-127`).
    pub note: Option<NonEmptyString>,
    /// Object identifier (`BT-128`).
    pub object: Option<ObjectReference>,
    /// Invoiced quantity (`BT-129`+`BT-130`).
    pub quantity: Quantity,
    /// Referenced purchase order line reference (`BT-132`).
    pub order_line_reference: Option<NonEmptyString>,
    /// Buyer accounting reference (`BT-133`).
    pub buyer_accounting_reference: Option<NonEmptyString>,
    /// Line period (`BG-26`).
    pub period: Option<Period>,
    /// Line allowances and charges (`BG-27`/`BG-28`).
    pub adjustments: Vec<LineAdjustment>,
    /// Price details (`BG-29`).
    pub price: Price,
    /// Line VAT treatment (`BG-30`).
    pub vat: VatTreatment,
    /// Item information (`BG-31`).
    pub item: Item,
}

impl InvoiceLine {
    /// The line net amount (`BT-131`): the rounded quantity-times-net-price, plus the signed line
    /// allowances and charges.
    pub fn net_amount(&self) -> Decimal {
        let base = self
            .price
            .base_quantity
            .map(|quantity| quantity.value)
            .unwrap_or(Decimal::ONE);
        let line = crate::invoice::rounded(self.quantity.value * self.price.net() / base);
        let adjustments: Decimal = self
            .adjustments
            .iter()
            .map(LineAdjustment::signed_amount)
            .sum();
        line + adjustments
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AdjustmentAmount, AdjustmentReason, Item, Quantity, Unit};

    fn line(quantity: i64, gross: i64, adjustments: Vec<LineAdjustment>) -> InvoiceLine {
        InvoiceLine {
            id: "1".parse().expect("a valid id"),
            note: None,
            object: None,
            quantity: Quantity {
                unit: Unit::from_code("C62").expect("C62 is a unit"),
                value: Decimal::new(quantity, 0),
            },
            order_line_reference: None,
            buyer_accounting_reference: None,
            period: None,
            adjustments,
            price: Price {
                gross: Decimal::new(gross, 2),
                discount: None,
                base_quantity: None,
            },
            vat: VatTreatment::ZeroRated,
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

    #[test]
    fn nets_the_quantity_times_the_net_price() {
        assert_eq!(
            line(10, 500, Vec::new()).net_amount(),
            Decimal::new(5000, 2)
        );
    }

    #[test]
    fn nets_after_the_line_allowances_and_charges() {
        let adjustments = vec![
            LineAdjustment {
                amount: AdjustmentAmount::Absolute(Decimal::new(200, 2)),
                reason: AdjustmentReason::Allowance {
                    code: None,
                    text: None,
                },
            },
            LineAdjustment {
                amount: AdjustmentAmount::Absolute(Decimal::new(50, 2)),
                reason: AdjustmentReason::Charge {
                    code: None,
                    text: None,
                },
            },
        ];

        // 10 * 5.00 - 2.00 + 0.50 = 48.50
        assert_eq!(
            line(10, 500, adjustments).net_amount(),
            Decimal::new(4850, 2)
        );
    }
}
