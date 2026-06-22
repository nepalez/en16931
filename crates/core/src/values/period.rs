//! An invoicing period (`BG-14` document, `BG-26` invoice line).

use crate::Date;

/// An invoicing period (`BG-14` document level, `BG-26` invoice line):
/// the date range that the invoice or a line charges for.
///
/// At least one bound is present (`BR-CO-19`), an empty period is forbidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    /// A start date only (`BT-73` document, `BT-134` line).
    From(Date),
    /// An end date only (`BT-74` document, `BT-135` line).
    Until(Date),
    /// Both a start and an end date (`BT-73`+`BT-74`, `BT-134`+`BT-135`).
    Range {
        /// The period start date (`BT-73` document, `BT-134` line).
        start: Date,
        /// The period end date (`BT-74` document, `BT-135` line).
        end: Date,
    },
}

impl Period {
    /// The period start date (`BT-73`/`BT-134`), absent for an end-only period.
    pub fn start(&self) -> Option<Date> {
        match self {
            Self::From(start) | Self::Range { start, .. } => Some(*start),
            Self::Until(_) => None,
        }
    }

    /// The period end date (`BT-74`/`BT-135`), absent for a start-only period.
    pub fn end(&self) -> Option<Date> {
        match self {
            Self::Until(end) | Self::Range { end, .. } => Some(*end),
            Self::From(_) => None,
        }
    }
}
