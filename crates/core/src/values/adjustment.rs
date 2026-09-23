use crate::{AllowanceReason, ChargeReason, Decimal, NonEmptyString, Percentage, VatTreatment};

/// A deduction or addition applied to the whole invoice (`BG-20` allowance, `BG-21` charge).
/// The `reason` direction tells an allowance from a charge.
///
/// All its amounts are in the invoice currency (`BT-5`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Adjustment {
    /// Amount (`BT-92`/`BT-99`+`BT-93`+`BT-94`/`BT-100`+`BT-101`).
    pub amount: Option<AdjustmentAmount>,
    /// VAT treatment (`BT-95`+`BT-96`/`BT-102`+`BT-103`).
    pub vat: Option<VatTreatment>,
    /// Reason and direction (`BT-97`+`BT-98`/`BT-104`+`BT-105`).
    pub reason: Option<AdjustmentReason>,
}

/// A deduction or addition applied to one invoice line (`BG-27` allowance, `BG-28` charge).
/// The line carries its own VAT, so no VAT treatment appears here.
///
/// All its amounts are in the invoice currency (`BT-5`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineAdjustment {
    /// Amount (`BT-136`/`BT-141`+`BT-137`+`BT-138`/`BT-142`+`BT-143`).
    pub amount: Option<AdjustmentAmount>,
    /// Reason and direction (`BT-139`+`BT-140`/`BT-144`+`BT-145`).
    pub reason: Option<AdjustmentReason>,
}

/// The amount of an adjustment: a fixed sum, or a sum stated as a percentage of a base.
/// The issuer states the sum in both forms: the library does not derive it from the base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjustmentAmount {
    /// A fixed amount (`BT-92`/`BT-99`/`BT-136`/`BT-141`).
    Absolute(Decimal),
    /// An amount stated as a percentage of a base
    /// (`BT-94`/`BT-101`/`BT-138`/`BT-143` of `BT-93`/`BT-100`/`BT-137`/ `BT-142`).
    Relative {
        /// The adjustment amount (`BT-92`/`BT-99`/`BT-136`/`BT-141`).
        amount: Decimal,
        /// The percentage rate applied to the base.
        rate: Percentage,
        /// The base the rate is applied to.
        base: Decimal,
    },
}

/// The reason for an adjustment, keyed by its direction: an allowance or a charge.
/// The direction selects the reason code list
/// (`UNCL5189` for an allowance, `UNCL7161` for a charge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjustmentReason {
    /// An allowance (a deduction).
    Allowance {
        /// Reason code (`BT-98`/`BT-140`).
        code: Option<AllowanceReason>,
        /// Reason text (`BT-97`/`BT-139`).
        text: Option<NonEmptyString>,
    },
    /// A charge (an addition).
    Charge {
        /// Reason code (`BT-105`/`BT-145`).
        code: Option<ChargeReason>,
        /// Reason text (`BT-104`/`BT-144`).
        text: Option<NonEmptyString>,
    },
}
