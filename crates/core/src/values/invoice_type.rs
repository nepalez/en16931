//! The invoice type code (`BT-3`, EN-16931 subset of UNTDID 1001).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-01`.

use crate::Error;
use crate::prelude::*;

/// The invoice type code (`BT-3`): the document type.
///
/// The set is the EN-16931 subset of UNTDID 1001, enforced by `BR-CL-01`.
/// It unites the UBL `InvoiceTypeCode` and `CreditNoteTypeCode` lists.
/// The discriminant of each variant is its numeric code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum InvoiceType {
    RequestForPayment = 71,
    DebitNoteGoodsServices = 80,
    CreditNoteGoodsServices = 81,
    MeteredServicesInvoice = 82,
    CreditNoteFinancialAdjustments = 83,
    DebitNoteFinancialAdjustments = 84,
    TaxNotification = 102,
    InvoicingDataSheet = 130,
    DirectPaymentValuation = 202,
    ProvisionalPaymentValuation = 203,
    PaymentValuation = 204,
    InterimApplicationForPayment = 211,
    FinalPaymentRequest = 218,
    PaymentRequestForCompletedUnits = 219,
    SelfBilledCreditNote = 261,
    ConsolidatedCreditNote = 262,
    PriceVariationInvoice = 295,
    CreditNotePriceVariation = 296,
    DelcredereCreditNote = 308,
    ProformaInvoice = 325,
    PartialInvoice = 326,
    CommercialInvoiceWithPackingList = 331,
    CommercialInvoice = 380,
    CreditNote = 381,
    CommissionNote = 382,
    DebitNote = 383,
    CorrectedInvoice = 384,
    ConsolidatedInvoice = 385,
    PrepaymentInvoice = 386,
    HireInvoice = 387,
    TaxInvoice = 388,
    SelfBilledInvoice = 389,
    DelcredereInvoice = 390,
    FactoredInvoice = 393,
    LeaseInvoice = 394,
    ConsignmentInvoice = 395,
    FactoredCreditNote = 396,
    OcrPaymentCreditNote = 420,
    DebitAdvice = 456,
    ReversalOfDebit = 457,
    ReversalOfCredit = 458,
    SelfBilledCorrectiveInvoice = 471,
    FactoredCorrectiveInvoice = 472,
    SelfBilledFactoredCorrectiveInvoice = 473,
    SelfPrepaymentInvoice = 500,
    SelfBilledFactoredInvoice = 501,
    SelfBilledFactoredCreditNote = 502,
    PrepaymentCreditNote = 503,
    SelfBilledDebitNote = 527,
    ForwardersCreditNote = 532,
    ForwardersInvoiceDiscrepancyReport = 553,
    InsurersInvoice = 575,
    ForwardersInvoice = 623,
    PortChargesDocuments = 633,
    InvoiceInformationForAccounting = 751,
    FreightInvoice = 780,
    ClaimNotification = 817,
    ConsularInvoice = 870,
    PartialConstructionInvoice = 875,
    PartialFinalConstructionInvoice = 876,
    FinalConstructionInvoice = 877,
    CustomsInvoice = 935,
}

/// Whether an invoice type code denotes an invoice or a credit note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Invoice,
    CreditNote,
}

impl InvoiceType {
    pub fn kind(&self) -> DocumentKind {
        match self {
            Self::CreditNoteGoodsServices
            | Self::CreditNoteFinancialAdjustments
            | Self::SelfBilledCreditNote
            | Self::ConsolidatedCreditNote
            | Self::CreditNotePriceVariation
            | Self::DelcredereCreditNote
            | Self::CreditNote
            | Self::FactoredCreditNote
            | Self::OcrPaymentCreditNote
            | Self::ReversalOfCredit
            | Self::SelfBilledFactoredCreditNote
            | Self::PrepaymentCreditNote
            | Self::ForwardersCreditNote => DocumentKind::CreditNote,
            _ => DocumentKind::Invoice,
        }
    }
}

impl Display for InvoiceType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", u16::from(*self))
    }
}

impl FromStr for InvoiceType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u16>()
            .ok()
            .and_then(|code| Self::try_from(code).ok())
            .ok_or_else(|| Error::InvalidValue(format!("{value:?}")))
    }
}

impl TryFrom<&str> for InvoiceType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl From<InvoiceType> for u32 {
    fn from(code: InvoiceType) -> Self {
        u16::from(code).into()
    }
}

impl TryFrom<u32> for InvoiceType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        u16::try_from(value)
            .ok()
            .and_then(|code| Self::try_from(code).ok())
            .ok_or_else(|| Error::InvalidValue(format!("{value:?}")))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_commercial_invoice_code() {
        let code: InvoiceType = "380".parse().expect("380 is a valid invoice type");

        assert_eq!(code, InvoiceType::CommercialInvoice);
        assert_eq!(code.to_string(), "380");
    }

    #[test]
    fn parses_a_credit_note_code() {
        let code: InvoiceType = "381".parse().expect("381 is a valid invoice type");

        assert_eq!(code, InvoiceType::CreditNote);
    }

    #[test]
    fn converts_from_and_into_a_number() {
        let code = InvoiceType::try_from(380u32).expect("380 is a valid invoice type");

        assert_eq!(code, InvoiceType::CommercialInvoice);
        assert_eq!(u16::from(code), 380);
        assert_eq!(u32::from(code), 380);
    }

    #[test]
    fn classifies_the_document_kind() {
        assert_eq!(InvoiceType::CommercialInvoice.kind(), DocumentKind::Invoice);
        assert_eq!(InvoiceType::CreditNote.kind(), DocumentKind::CreditNote);
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!(matches!(
            "999".parse::<InvoiceType>(),
            Err(Error::InvalidValue(_))
        ));
    }

    #[test]
    fn rejects_an_unknown_number() {
        assert!(matches!(
            InvoiceType::try_from(999u32),
            Err(Error::InvalidValue(_))
        ));
    }

    #[test]
    fn converts_via_try_from_str() {
        assert_eq!(
            InvoiceType::try_from("380").expect("380 is a valid invoice type"),
            InvoiceType::CommercialInvoice
        );
    }
}
