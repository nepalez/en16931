use crate::{Decimal, Quantity};

/// Price details (`BG-29`): the item price and the quantity it applies to.
///
/// Amounts are in the invoice currency (`BT-5`).
/// The price is stated per base quantity, not per single unit,
/// so a price of `5.00` over a base quantity of `1000` means `0.005` per unit.
/// The net price (`BT-146`) is `gross - discount`. The binding derives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price {
    /// Gross price (`BT-148`): the price before the discount, always present.
    pub gross: Decimal,
    /// Price discount (`BT-147`).
    pub discount: Option<Decimal>,
    /// Base quantity (`BT-149`+`BT-150`).
    pub base_quantity: Option<Quantity>,
}

impl Price {
    /// The net price (`BT-146`): the gross price less the discount.
    pub fn net(&self) -> Decimal {
        self.gross - self.discount.unwrap_or(Decimal::ZERO)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn price(gross: i64, discount: Option<i64>) -> Price {
        Price {
            gross: Decimal::new(gross, 2),
            discount: discount.map(|value| Decimal::new(value, 2)),
            base_quantity: None,
        }
    }

    #[test]
    fn nets_the_gross_less_the_discount() {
        assert_eq!(price(1000, Some(150)).net(), Decimal::new(850, 2));
    }

    #[test]
    fn nets_to_the_gross_without_a_discount() {
        assert_eq!(price(1000, None).net(), Decimal::new(1000, 2));
    }
}
