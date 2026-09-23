//! The price details (`BG-29`).

use crate::{Decimal, Quantity};

/// The item price and the quantity it applies to (`BG-29`).
///
/// Amounts are in the invoice currency (`BT-5`).
/// The price is stated per base quantity, not per single unit,
/// so a price of `5.00` over a base quantity of `1000` means `0.005` per unit.
/// The issuer states the net price along with the gross price and the discount.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Price {
    /// The price after the discount (`BT-146`).
    pub net: Option<Decimal>,
    /// The price before the discount (`BT-148`).
    pub gross: Option<Decimal>,
    /// Price discount (`BT-147`).
    pub discount: Option<Decimal>,
    /// Base quantity (`BT-149`+`BT-150`).
    pub base_quantity: Option<Quantity>,
}
