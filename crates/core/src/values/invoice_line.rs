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

impl InvoiceLine {
    /// The line net amount (`BT-131`): the rounded quantity-times-net-price, plus the signed line
    /// allowances and charges, or `None` when any of these inputs is absent.
    pub fn net_amount(&self) -> Option<Decimal> {
        let quantity = self.quantity.as_ref()?;
        let price = self.price.as_ref()?;
        let base = price
            .base_quantity
            .as_ref()
            .map(|quantity| quantity.value)
            .unwrap_or(Decimal::ONE);
        let line = crate::invoice::rounded(quantity.value * price.net()? / base);
        let adjustments: Decimal = self
            .adjustments
            .iter()
            .map(LineAdjustment::signed_amount)
            .sum::<Option<Decimal>>()?;
        Some(line + adjustments)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AdjustmentAmount, AdjustmentReason, Quantity, Unit};

    fn line(quantity: i64, gross: i64, adjustments: Vec<LineAdjustment>) -> InvoiceLine {
        InvoiceLine {
            quantity: Some(Quantity {
                unit: Unit::from_code("C62").expect("C62 is a unit"),
                value: Decimal::new(quantity, 0),
            }),
            adjustments,
            price: Some(Price {
                gross: Some(Decimal::new(gross, 2)),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn nets_the_quantity_times_the_net_price() {
        assert_eq!(
            line(10, 500, Vec::new()).net_amount(),
            Some(Decimal::new(5000, 2))
        );
    }

    #[test]
    fn nets_after_the_line_allowances_and_charges() {
        let adjustments = vec![
            LineAdjustment {
                amount: Some(AdjustmentAmount::Absolute(Decimal::new(200, 2))),
                reason: Some(AdjustmentReason::Allowance {
                    code: None,
                    text: None,
                }),
            },
            LineAdjustment {
                amount: Some(AdjustmentAmount::Absolute(Decimal::new(50, 2))),
                reason: Some(AdjustmentReason::Charge {
                    code: None,
                    text: None,
                }),
            },
        ];

        // 10 * 5.00 - 2.00 + 0.50 = 48.50
        assert_eq!(
            line(10, 500, adjustments).net_amount(),
            Some(Decimal::new(4850, 2))
        );
    }

    #[test]
    fn nets_nothing_without_a_price() {
        let line = InvoiceLine {
            price: None,
            ..line(10, 500, Vec::new())
        };

        assert_eq!(line.net_amount(), None);
    }
}
