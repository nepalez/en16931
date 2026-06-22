use crate::Error;
use crate::prelude::*;

/// The VAT exemption reason (`BT-121`, CEF `VATEX`):
/// the legal reason for a VAT exemption.
///
/// Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-22`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
pub enum ExemptionReason {
    #[display("VATEX-EU-79-C")]
    Eu79C,
    #[display("VATEX-EU-132")]
    Eu132,
    #[display("VATEX-EU-132-1A")]
    Eu1321A,
    #[display("VATEX-EU-132-1B")]
    Eu1321B,
    #[display("VATEX-EU-132-1C")]
    Eu1321C,
    #[display("VATEX-EU-132-1D")]
    Eu1321D,
    #[display("VATEX-EU-132-1E")]
    Eu1321E,
    #[display("VATEX-EU-132-1F")]
    Eu1321F,
    #[display("VATEX-EU-132-1G")]
    Eu1321G,
    #[display("VATEX-EU-132-1H")]
    Eu1321H,
    #[display("VATEX-EU-132-1I")]
    Eu1321I,
    #[display("VATEX-EU-132-1J")]
    Eu1321J,
    #[display("VATEX-EU-132-1K")]
    Eu1321K,
    #[display("VATEX-EU-132-1L")]
    Eu1321L,
    #[display("VATEX-EU-132-1M")]
    Eu1321M,
    #[display("VATEX-EU-132-1N")]
    Eu1321N,
    #[display("VATEX-EU-132-1O")]
    Eu1321O,
    #[display("VATEX-EU-132-1P")]
    Eu1321P,
    #[display("VATEX-EU-132-1Q")]
    Eu1321Q,
    #[display("VATEX-EU-135-1")]
    Eu1351,
    #[display("VATEX-EU-143")]
    Eu143,
    #[display("VATEX-EU-143-1A")]
    Eu1431A,
    #[display("VATEX-EU-143-1B")]
    Eu1431B,
    #[display("VATEX-EU-143-1C")]
    Eu1431C,
    #[display("VATEX-EU-143-1D")]
    Eu1431D,
    #[display("VATEX-EU-143-1E")]
    Eu1431E,
    #[display("VATEX-EU-143-1F")]
    Eu1431F,
    #[display("VATEX-EU-143-1FA")]
    Eu1431Fa,
    #[display("VATEX-EU-143-1G")]
    Eu1431G,
    #[display("VATEX-EU-143-1H")]
    Eu1431H,
    #[display("VATEX-EU-143-1I")]
    Eu1431I,
    #[display("VATEX-EU-143-1J")]
    Eu1431J,
    #[display("VATEX-EU-143-1K")]
    Eu1431K,
    #[display("VATEX-EU-143-1L")]
    Eu1431L,
    #[display("VATEX-EU-144")]
    Eu144,
    #[display("VATEX-EU-146-1E")]
    Eu1461E,
    #[display("VATEX-EU-159")]
    Eu159,
    #[display("VATEX-EU-309")]
    Eu309,
    #[display("VATEX-EU-148")]
    Eu148,
    #[display("VATEX-EU-148-A")]
    Eu148A,
    #[display("VATEX-EU-148-B")]
    Eu148B,
    #[display("VATEX-EU-148-C")]
    Eu148C,
    #[display("VATEX-EU-148-D")]
    Eu148D,
    #[display("VATEX-EU-148-E")]
    Eu148E,
    #[display("VATEX-EU-148-F")]
    Eu148F,
    #[display("VATEX-EU-148-G")]
    Eu148G,
    #[display("VATEX-EU-151")]
    Eu151,
    #[display("VATEX-EU-151-1A")]
    Eu1511A,
    #[display("VATEX-EU-151-1AA")]
    Eu1511Aa,
    #[display("VATEX-EU-151-1B")]
    Eu1511B,
    #[display("VATEX-EU-151-1C")]
    Eu1511C,
    #[display("VATEX-EU-151-1D")]
    Eu1511D,
    #[display("VATEX-EU-151-1E")]
    Eu1511E,
    #[display("VATEX-EU-D")]
    EuD,
    #[display("VATEX-EU-F")]
    EuF,
    #[display("VATEX-EU-I")]
    EuI,
    #[display("VATEX-EU-J")]
    EuJ,
    #[display("VATEX-FR-FRANCHISE")]
    FrFranchise,
    #[display("VATEX-FR-CNWVAT")]
    FrCnwvat,
    #[display("VATEX-EU-153")]
    Eu153,
    #[display("VATEX-FR-CGI261-1")]
    FrCgi2611,
    #[display("VATEX-FR-CGI261-2")]
    FrCgi2612,
    #[display("VATEX-FR-CGI261-3")]
    FrCgi2613,
    #[display("VATEX-FR-CGI261-4")]
    FrCgi2614,
    #[display("VATEX-FR-CGI261-5")]
    FrCgi2615,
    #[display("VATEX-FR-CGI261-7")]
    FrCgi2617,
    #[display("VATEX-FR-CGI261-8")]
    FrCgi2618,
    #[display("VATEX-FR-CGI261A")]
    FrCgi261A,
    #[display("VATEX-FR-CGI261B")]
    FrCgi261B,
    #[display("VATEX-FR-CGI261C-1")]
    FrCgi261C1,
    #[display("VATEX-FR-CGI261C-2")]
    FrCgi261C2,
    #[display("VATEX-FR-CGI261C-3")]
    FrCgi261C3,
    #[display("VATEX-FR-CGI261D-1")]
    FrCgi261D1,
    #[display("VATEX-FR-CGI261D-1BIS")]
    FrCgi261D1Bis,
    #[display("VATEX-FR-CGI261D-2")]
    FrCgi261D2,
    #[display("VATEX-FR-CGI261D-3")]
    FrCgi261D3,
    #[display("VATEX-FR-CGI261D-4")]
    FrCgi261D4,
    #[display("VATEX-FR-CGI261E-1")]
    FrCgi261E1,
    #[display("VATEX-FR-CGI261E-2")]
    FrCgi261E2,
    #[display("VATEX-FR-CGI277A")]
    FrCgi277A,
    #[display("VATEX-FR-CGI275")]
    FrCgi275,
    #[display("VATEX-FR-298SEXDECIESA")]
    Fr298Sexdeciesa,
    #[display("VATEX-FR-CGI295")]
    FrCgi295,
    #[display("VATEX-FR-AE")]
    FrAe,
}

impl TryFrom<&str> for ExemptionReason {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value).map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_known_code() {
        let reason: ExemptionReason = "VATEX-EU-132"
            .parse()
            .expect("VATEX-EU-132 is a valid exemption reason");

        assert_eq!(reason, ExemptionReason::Eu132);
        assert_eq!(reason.to_string(), "VATEX-EU-132");
    }

    #[test]
    fn parses_a_national_code() {
        let reason: ExemptionReason = "VATEX-FR-298SEXDECIESA"
            .parse()
            .expect("a national exemption reason parses");

        assert_eq!(reason, ExemptionReason::Fr298Sexdeciesa);
        assert_eq!(reason.to_string(), "VATEX-FR-298SEXDECIESA");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("VATEX-EU-999".parse::<ExemptionReason>().is_err());
    }

    #[test]
    fn converts_via_try_from() {
        assert_eq!(
            ExemptionReason::try_from("VATEX-EU-F")
                .expect("VATEX-EU-F is a valid exemption reason"),
            ExemptionReason::EuF
        );
        assert!(matches!(
            ExemptionReason::try_from("FOO"),
            Err(Error::InvalidValue(_))
        ));
    }
}
