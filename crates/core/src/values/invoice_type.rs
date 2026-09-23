//! The invoice type code (`BT-3`, EN-16931 subset of UNTDID 1001).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-01`.

use crate::Error;
use crate::prelude::*;

/// The document type (`BT-3`).
///
/// The set is the EN-16931 subset of UNTDID 1001, enforced by `BR-CL-01`.
/// It unites the UBL `InvoiceTypeCode` and `CreditNoteTypeCode` lists.
/// The discriminant of each variant is its numeric code.
/// The default is the commercial invoice (`380`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
    #[default]
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

/// The kind of an invoice: a claim for a payment, or a credit note that reduces a claim.
///
/// The issuer decides the kind by the business event, before it picks the type code (`BT-3`),
/// and the external validator checks that the kind, the type code, and the amounts agree.
/// The default is an invoice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InvoiceKind {
    /// A claim for a payment.
    #[default]
    Invoice,
    /// A reduction of a claim issued before.
    CreditNote,
}

/// The kind a type code denotes by the `BR-CL-01` lists, for a binding that states no kind
/// of its own (CII). Code `81` sits in both lists and is taken as a credit note.
impl From<InvoiceType> for InvoiceKind {
    fn from(code: InvoiceType) -> Self {
        match code {
            InvoiceType::CreditNoteGoodsServices
            | InvoiceType::CreditNoteFinancialAdjustments
            | InvoiceType::SelfBilledCreditNote
            | InvoiceType::ConsolidatedCreditNote
            | InvoiceType::CreditNotePriceVariation
            | InvoiceType::DelcredereCreditNote
            | InvoiceType::CreditNote
            | InvoiceType::FactoredCreditNote
            | InvoiceType::OcrPaymentCreditNote
            | InvoiceType::ReversalOfCredit
            | InvoiceType::SelfBilledFactoredCreditNote
            | InvoiceType::PrepaymentCreditNote
            | InvoiceType::ForwardersCreditNote => Self::CreditNote,
            _ => Self::Invoice,
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
            .ok_or_else(|| Error::invalid_value(format!("{value:?}")))
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
            .ok_or_else(|| Error::invalid_value(format!("{value:?}")))
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
    fn classifies_the_invoice_kind() {
        assert_eq!(
            InvoiceKind::from(InvoiceType::CommercialInvoice),
            InvoiceKind::Invoice
        );
        assert_eq!(
            InvoiceKind::from(InvoiceType::CreditNote),
            InvoiceKind::CreditNote
        );
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!(matches!(
            "999".parse::<InvoiceType>(),
            Err(Error::InvalidValue { .. })
        ));
    }

    #[test]
    fn rejects_an_unknown_number() {
        assert!(matches!(
            InvoiceType::try_from(999u32),
            Err(Error::InvalidValue { .. })
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
