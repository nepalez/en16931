use crate::{Decimal, Quantity};

/// Price details (`BG-29`): the item price and the quantity it applies to.
///
/// Amounts are in the invoice currency (`BT-5`).
/// The price is stated per base quantity, not per single unit,
/// so a price of `5.00` over a base quantity of `1000` means `0.005` per unit.
/// The issuer states the net price along with the gross price and the discount.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Price {
    /// Net price (`BT-146`): the price after the discount.
    pub net: Option<Decimal>,
    /// Gross price (`BT-148`): the price before the discount.
    pub gross: Option<Decimal>,
    /// Price discount (`BT-147`).
    pub discount: Option<Decimal>,
    /// Base quantity (`BT-149`+`BT-150`).
    pub base_quantity: Option<Quantity>,
}
