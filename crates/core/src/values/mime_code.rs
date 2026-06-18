//! The attached document mime code (`BT-125`, `BR-CL-24`).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-24`.
//! The set is closed: only these media types may carry a binary attachment.

use crate::Error;
use crate::prelude::*;

/// The mime code of a binary attachment (`BT-125`): the media type of embedded document.
/// The set is the closed `BR-CL-24` list of media types allowed for an attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
pub enum MimeCode {
    /// PDF document (`application/pdf`).
    #[display("application/pdf")]
    Pdf,
    /// PNG image (`image/png`).
    #[display("image/png")]
    Png,
    /// JPEG image (`image/jpeg`).
    #[display("image/jpeg")]
    Jpeg,
    /// Comma-separated values (`text/csv`).
    #[display("text/csv")]
    Csv,
    /// Office Open XML spreadsheet (`.xlsx`).
    #[display("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")]
    Xlsx,
    /// OpenDocument spreadsheet (`.ods`).
    #[display("application/vnd.oasis.opendocument.spreadsheet")]
    Ods,
}

impl TryFrom<&str> for MimeCode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value).map_err(Into::into)
    }
}

#[cfg(feature = "mime")]
mod _mime {
    use super::*;

    impl From<MimeCode> for Mime {
        fn from(code: MimeCode) -> Self {
            code.to_string()
                .parse()
                .expect("a BR-CL-24 code is a valid media type")
        }
    }

    impl TryFrom<&Mime> for MimeCode {
        type Error = Error;

        fn try_from(value: &Mime) -> Result<Self, Self::Error> {
            MimeCode::try_from(format!("{}/{}", value.type_(), value.subtype()).as_str())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_known_code() {
        let mime: MimeCode = "application/pdf".parse().expect("a valid mime code parses");

        assert_eq!(mime, MimeCode::Pdf);
        assert_eq!(mime.to_string(), "application/pdf");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("application/zip".parse::<MimeCode>().is_err());
    }

    #[cfg(feature = "mime")]
    #[test]
    fn converts_into_mime() {
        let pdf: Mime = "application/pdf"
            .parse()
            .expect("a valid media type parses");

        assert_eq!(Mime::from(MimeCode::Pdf), pdf);
    }

    #[cfg(feature = "mime")]
    #[test]
    fn converts_from_mime() {
        let pdf: Mime = "application/pdf"
            .parse()
            .expect("a valid media type parses");

        assert_eq!(
            MimeCode::try_from(&pdf).expect("a known media type maps to a mime code"),
            MimeCode::Pdf
        );
    }
}
