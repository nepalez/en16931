//! One group of the VAT breakdown (`BG-23`).

use crate::{Decimal, VatTreatment};

/// A VAT category and rate with the amounts the issuer states for it (`BG-23`).
///
/// All its amounts are in the invoice currency (`BT-5`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VatBreakdown {
    /// VAT treatment of the group (`BT-118`+`BT-119`+`BT-120`+`BT-121`).
    pub treatment: Option<VatTreatment>,
    /// Taxable amount (`BT-116`).
    pub taxable: Option<Decimal>,
    /// Tax amount (`BT-117`).
    pub tax: Option<Decimal>,
}
