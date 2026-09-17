use crate::{Decimal, NonEmptyString, Percentage, VatCategory};

mod exemption_reason;

pub use exemption_reason::ExemptionReason;

/// The VAT treatment of a taxable element.
/// It includes VAT category with the rate or exemption reason the category admits
/// (`BT-95`/`BT-102`/`BT-151`, `BT-96`/`BT-103`/`BT-152`, `BT-120`/`BT-121`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VatTreatment {
    /// Standard rate (`S`).
    Standard { rate: Percentage },
    /// Canary Islands general indirect tax (`L`, IGIC).
    CanaryIslands { rate: Percentage },
    /// Ceuta and Melilla tax on production, services, and imports (`M`, IPSI).
    CeutaMelilla { rate: Percentage },
    /// VAT transferred in Italy (`B`, split payment).
    TransferredItaly { rate: Percentage },
    /// Exempt from VAT (`E`).
    Exempt {
        /// Exemption reason code (`BT-121`).
        code: Option<ExemptionReason>,
        /// Exemption reason text (`BT-120`).
        text: Option<NonEmptyString>,
    },
    /// Zero rate (`Z`).
    ZeroRated,
    /// VAT reverse charge (`AE`).
    ReverseCharge,
    /// Intra-community supply (`K`).
    IntraCommunitySupply,
    /// Export outside the EU (`G`).
    FreeExport,
    /// Not subject to VAT (`O`).
    OutsideScope,
}

impl VatTreatment {
    /// The VAT category code (`BT-118`/`BT-151`) of this treatment.
    pub fn category(&self) -> VatCategory {
        match self {
            Self::Standard { .. } => VatCategory::Standard,
            Self::CanaryIslands { .. } => VatCategory::CanaryIslands,
            Self::CeutaMelilla { .. } => VatCategory::CeutaMelilla,
            Self::TransferredItaly { .. } => VatCategory::TransferredItaly,
            Self::Exempt { .. } => VatCategory::Exempt,
            Self::ZeroRated => VatCategory::ZeroRated,
            Self::ReverseCharge => VatCategory::ReverseCharge,
            Self::IntraCommunitySupply => VatCategory::IntraCommunitySupply,
            Self::FreeExport => VatCategory::FreeExport,
            Self::OutsideScope => VatCategory::OutsideScope,
        }
    }

    /// The VAT rate (`BT-119`/`BT-152`) of this treatment,
    /// zero for the non-rated categories.
    pub fn rate(&self) -> Decimal {
        match self {
            Self::Standard { rate }
            | Self::CanaryIslands { rate }
            | Self::CeutaMelilla { rate }
            | Self::TransferredItaly { rate } => Decimal::from(*rate),
            _ => Decimal::ZERO,
        }
    }

    /// Reconstructs a treatment from its category code and rate,
    /// the inverse of `category`.
    ///
    /// A binding parser uses it to read a classified tax category.
    /// The rate applies only to the rated categories,
    /// and an exemption reason is filled separately from the VAT breakdown.
    pub(crate) fn from_category(category: VatCategory, rate: Percentage) -> Self {
        match category {
            VatCategory::Standard => Self::Standard { rate },
            VatCategory::CanaryIslands => Self::CanaryIslands { rate },
            VatCategory::CeutaMelilla => Self::CeutaMelilla { rate },
            VatCategory::TransferredItaly => Self::TransferredItaly { rate },
            VatCategory::Exempt => Self::Exempt {
                code: None,
                text: None,
            },
            VatCategory::ZeroRated => Self::ZeroRated,
            VatCategory::ReverseCharge => Self::ReverseCharge,
            VatCategory::IntraCommunitySupply => Self::IntraCommunitySupply,
            VatCategory::FreeExport => Self::FreeExport,
            VatCategory::OutsideScope => Self::OutsideScope,
        }
    }
}
