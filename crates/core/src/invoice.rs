use crate::prelude::*;
use crate::{
    Adjustment, AdjustmentReason, Amount, Buyer, Currency, Date, Decimal, Delivery, InvoiceLine,
    InvoiceType, NonEmptyString, Note, ObjectReference, Payee, PaymentInstructions, Period,
    PrecedingInvoice, Seller, SupportingDocument, TaxRepresentative, VatPoint, VatTreatment,
};

/// Rounds a derived money amount to two decimals, half away from zero.
pub(crate) fn rounded(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// This object carries business facts an invoice can describe.
///
/// Every field but the type code is optional: the model checks the types of the values,
/// and the external validator checks the completeness of the document.
/// A profile decides on serialization which terms to forbid.
/// All amounts are in the invoice currency (`BT-5`).
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
    /// The sum of line net amounts (`BT-106`), or `None` when a line lacks an input.
    pub fn line_net_total(&self) -> Option<Decimal> {
        self.lines.iter().map(InvoiceLine::net_amount).sum()
    }

    /// The sum of document-level allowances (`BT-107`),
    /// or `None` when an adjustment lacks an input.
    pub fn allowances_total(&self) -> Option<Decimal> {
        Some(self.adjustment_totals()?.0)
    }

    /// The sum of document-level charges (`BT-108`),
    /// or `None` when an adjustment lacks an input.
    pub fn charges_total(&self) -> Option<Decimal> {
        Some(self.adjustment_totals()?.1)
    }

    /// The total without VAT (`BT-109`), or `None` when an input is absent.
    pub fn net_total(&self) -> Option<Decimal> {
        Some(self.line_net_total()? - self.allowances_total()? + self.charges_total()?)
    }

    /// The total VAT amount (`BT-110`): the sum of the per-category taxes, each rounded,
    /// or `None` when an input is absent.
    pub fn vat_total(&self) -> Option<Decimal> {
        Some(
            self.vat_breakdown()?
                .into_iter()
                .map(|group| group.tax)
                .sum(),
        )
    }

    /// The total with VAT (`BT-112`), or `None` when an input is absent.
    pub fn gross_total(&self) -> Option<Decimal> {
        Some(self.net_total()? + self.vat_total()?)
    }

    /// The amount due for payment (`BT-115`), or `None` when an input is absent.
    pub fn due(&self) -> Option<Decimal> {
        Some(
            self.gross_total()? - self.paid.unwrap_or(Decimal::ZERO)
                + self.rounding.unwrap_or(Decimal::ZERO),
        )
    }

    /// The VAT breakdown (`BG-23`): one group per category and rate,
    /// carrying the taxable base (`BT-116`) and the rounded tax percent (`BT-117`),
    /// or `None` when a line or an adjustment lacks an input.
    ///
    /// The group keeps a representative `VatTreatment`
    /// so the exemption reason (`BT-120`/`BT-121`) survives into the serialized breakdown.
    /// The binding renders it into the tax total.
    pub(crate) fn vat_breakdown(&self) -> Option<Vec<VatBreakdown>> {
        let groups = self
            .vat_groups()?
            .into_iter()
            .map(|(treatment, taxable)| {
                let tax = rounded(taxable * treatment.rate() / Decimal::from(100));
                VatBreakdown {
                    treatment,
                    taxable,
                    tax,
                }
            })
            .collect();
        Some(groups)
    }

    // The document-level allowances and charges totals, in that order.
    fn adjustment_totals(&self) -> Option<(Decimal, Decimal)> {
        let mut allowances = Decimal::ZERO;
        let mut charges = Decimal::ZERO;
        for adjustment in &self.adjustments {
            let value = adjustment.amount.as_ref()?.value();
            match adjustment.reason.as_ref()? {
                AdjustmentReason::Allowance { .. } => allowances += value,
                AdjustmentReason::Charge { .. } => charges += value,
            }
        }
        Some((allowances, charges))
    }

    // The taxable base per VAT category and rate:
    // it contains line nets plus document charges minus document allowances of that treatment.
    //
    // Each group keeps the first treatment it saw, so the exemption reason travels with it.
    // It underlies the per-category tax (`BT-116`/`BT-117`).
    fn vat_groups(&self) -> Option<Vec<(VatTreatment, Decimal)>> {
        let mut groups: Vec<(VatTreatment, Decimal)> = Vec::new();
        let mut accumulate = |vat: &VatTreatment, amount: Decimal| {
            let category = vat.category();
            let rate = vat.rate();
            match groups
                .iter_mut()
                .find(|(grouped, _)| grouped.category() == category && grouped.rate() == rate)
            {
                Some((_, taxable)) => *taxable += amount,
                None => groups.push((vat.clone(), amount)),
            }
        };
        for line in &self.lines {
            accumulate(line.vat.as_ref()?, line.net_amount()?);
        }
        for adjustment in &self.adjustments {
            let value = adjustment.amount.as_ref()?.value();
            let signed = match adjustment.reason.as_ref()? {
                AdjustmentReason::Charge { .. } => value,
                AdjustmentReason::Allowance { .. } => -value,
            };
            accumulate(adjustment.vat.as_ref()?, signed);
        }
        Some(groups)
    }
}

/// One group of the VAT breakdown (`BG-23`): a category and rate, its taxable base and tax.
pub(crate) struct VatBreakdown {
    /// A representative treatment of the group, carrying the category, rate, and exemption reason.
    pub treatment: VatTreatment,
    /// The taxable base of the group (`BT-116`).
    pub taxable: Decimal,
    /// The tax of the group (`BT-117`), rounded.
    pub tax: Decimal,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Percentage, Price, Quantity, Unit};

    fn line(gross: i64, rate: i64) -> InvoiceLine {
        InvoiceLine {
            quantity: Some(Quantity {
                unit: Unit::from_code("C62").expect("C62 is a unit"),
                value: Decimal::ONE,
            }),
            price: Some(Price {
                gross: Some(Decimal::new(gross, 2)),
                ..Default::default()
            }),
            vat: Some(VatTreatment::Standard {
                rate: Percentage::try_from(Decimal::from(rate)).expect("a valid rate"),
            }),
            ..Default::default()
        }
    }

    fn invoice(lines: Vec<InvoiceLine>) -> Invoice {
        Invoice {
            currency: Some(Currency::EUR),
            lines,
            ..Default::default()
        }
    }

    #[test]
    fn totals_the_net_vat_and_gross_across_vat_rates() {
        let invoice = invoice(vec![line(10000, 20), line(5000, 10)]);

        assert_eq!(invoice.net_total(), Some(Decimal::new(15000, 2)));
        // 100.00 * 20% + 50.00 * 10% = 20.00 + 5.00, each rounded per category
        assert_eq!(invoice.vat_total(), Some(Decimal::new(2500, 2)));
        assert_eq!(invoice.gross_total(), Some(Decimal::new(17500, 2)));
    }

    #[test]
    fn dues_the_gross_less_the_paid_plus_the_rounding() {
        let mut invoice = invoice(vec![line(10002, 20)]);
        invoice.paid = Some(Decimal::new(5000, 2));
        invoice.rounding = Some(Decimal::new(-2, 2));

        // gross 120.02 - paid 50.00 + rounding -0.02 = 70.00
        assert_eq!(invoice.due(), Some(Decimal::new(7000, 2)));
    }

    #[test]
    fn totals_nothing_when_a_line_lacks_a_price() {
        let unpriced = InvoiceLine {
            price: None,
            ..line(5000, 10)
        };
        let invoice = invoice(vec![line(10000, 20), unpriced]);

        assert_eq!(invoice.net_total(), None);
    }
}
