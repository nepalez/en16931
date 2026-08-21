//! The payment means code of an invoice
//! (`BT-81`, EN-16931 subset of UNCL4461).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-16`.

use crate::Error;
use crate::prelude::*;

/// The payment means (`BT-81`): how the payment is made.
///
/// The set is the EN-16931 subset of UNCL4461, enforced by `BR-CL-16`. Besides the
/// numeric codes it admits `ZZZ` (mutually defined), so the codes are kept as strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
pub enum PaymentMeans {
    #[display("1")]
    InstrumentNotDefined,
    #[display("2")]
    AchCredit,
    #[display("3")]
    AchDebit,
    #[display("4")]
    AchDemandDebitReversal,
    #[display("5")]
    AchDemandCreditReversal,
    #[display("6")]
    AchDemandCredit,
    #[display("7")]
    AchDemandDebit,
    #[display("8")]
    Hold,
    #[display("9")]
    NationalOrRegionalClearing,
    #[display("10")]
    InCash,
    #[display("11")]
    AchSavingsCreditReversal,
    #[display("12")]
    AchSavingsDebitReversal,
    #[display("13")]
    AchSavingsCredit,
    #[display("14")]
    AchSavingsDebit,
    #[display("15")]
    BookentryCredit,
    #[display("16")]
    BookentryDebit,
    #[display("17")]
    AchDemandCcdCredit,
    #[display("18")]
    AchDemandCcdDebit,
    #[display("19")]
    AchDemandCtpCredit,
    #[display("20")]
    Cheque,
    #[display("21")]
    BankersDraft,
    #[display("22")]
    CertifiedBankersDraft,
    #[display("23")]
    BankCheque,
    #[display("24")]
    BillOfExchangeAwaitingAcceptance,
    #[display("25")]
    CertifiedCheque,
    #[display("26")]
    LocalCheque,
    #[display("27")]
    AchDemandCtpDebit,
    #[display("28")]
    AchDemandCtxCredit,
    #[display("29")]
    AchDemandCtxDebit,
    #[display("30")]
    CreditTransfer,
    #[display("31")]
    DebitTransfer,
    #[display("32")]
    AchDemandCcdPlusCredit,
    #[display("33")]
    AchDemandCcdPlusDebit,
    #[display("34")]
    AchPpd,
    #[display("35")]
    AchSavingsCcdCredit,
    #[display("36")]
    AchSavingsCcdDebit,
    #[display("37")]
    AchSavingsCtpCredit,
    #[display("38")]
    AchSavingsCtpDebit,
    #[display("39")]
    AchSavingsCtxCredit,
    #[display("40")]
    AchSavingsCtxDebit,
    #[display("41")]
    AchSavingsCcdPlusCredit,
    #[display("42")]
    PaymentToBankAccount,
    #[display("43")]
    AchSavingsCcdPlusDebit,
    #[display("44")]
    AcceptedBillOfExchange,
    #[display("45")]
    ReferencedHomeBankingCreditTransfer,
    #[display("46")]
    InterbankDebitTransfer,
    #[display("47")]
    HomeBankingDebitTransfer,
    #[display("48")]
    BankCard,
    #[display("49")]
    DirectDebit,
    #[display("50")]
    PaymentByPostgiro,
    #[display("51")]
    TelereglementCfonbOptionA,
    #[display("52")]
    UrgentCommercialPayment,
    #[display("53")]
    UrgentTreasuryPayment,
    #[display("54")]
    CreditCard,
    #[display("55")]
    DebitCard,
    #[display("56")]
    Bankgiro,
    #[display("57")]
    StandingAgreement,
    #[display("58")]
    SepaCreditTransfer,
    #[display("59")]
    SepaDirectDebit,
    #[display("60")]
    PromissoryNote,
    #[display("61")]
    PromissoryNoteSignedByDebitor,
    #[display("62")]
    PromissoryNoteSignedByDebitorEndorsedByBank,
    #[display("63")]
    PromissoryNoteSignedByDebitorEndorsedByThirdParty,
    #[display("64")]
    PromissoryNoteSignedByBank,
    #[display("65")]
    PromissoryNoteSignedByBankEndorsedByAnotherBank,
    #[display("66")]
    PromissoryNoteSignedByThirdParty,
    #[display("67")]
    PromissoryNoteSignedByThirdPartyEndorsedByBank,
    #[display("68")]
    OnlinePaymentService,
    #[display("69")]
    TransferAdvice,
    #[display("70")]
    BillDrawnByCreditorOnDebitor,
    #[display("74")]
    BillDrawnByCreditorOnBank,
    #[display("75")]
    BillDrawnByCreditorEndorsedByAnotherBank,
    #[display("76")]
    BillDrawnByCreditorOnBankEndorsedByThirdParty,
    #[display("77")]
    BillDrawnByCreditorOnThirdParty,
    #[display("78")]
    BillDrawnByCreditorOnThirdPartyAcceptedEndorsedByBank,
    #[display("91")]
    NotTransferableBankersDraft,
    #[display("92")]
    NotTransferableLocalCheque,
    #[display("93")]
    ReferenceGiro,
    #[display("94")]
    UrgentGiro,
    #[display("95")]
    FreeFormatGiro,
    #[display("96")]
    RequestedMethodNotUsed,
    #[display("97")]
    ClearingBetweenPartners,
    #[display("98")]
    ElectronicallyRecordedMonetaryClaims,
    #[display("ZZZ")]
    MutuallyDefined,
}

impl TryFrom<&str> for PaymentMeans {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value).map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_numeric_code() {
        let means: PaymentMeans = "30".parse().expect("30 is a valid payment means");

        assert_eq!(means, PaymentMeans::CreditTransfer);
        assert_eq!(means.to_string(), "30");
    }

    #[test]
    fn parses_the_mutually_defined_code() {
        let means: PaymentMeans = "ZZZ".parse().expect("ZZZ is a valid payment means");

        assert_eq!(means, PaymentMeans::MutuallyDefined);
        assert_eq!(means.to_string(), "ZZZ");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("999".parse::<PaymentMeans>().is_err());
    }

    #[test]
    fn converts_via_try_from() {
        assert_eq!(
            PaymentMeans::try_from("30").expect("30 is a valid payment means"),
            PaymentMeans::CreditTransfer
        );
        assert!(matches!(
            PaymentMeans::try_from("999"),
            Err(Error::InvalidValue { .. })
        ));
    }
}
