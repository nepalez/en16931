//! The VAT point of an invoice (`BT-7` date or `BT-8` event code).
//!
//! The event code source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-06`.

use crate::Error;
use crate::prelude::*;

/// The VAT point: when the VAT becomes chargeable, stated either as an explicit date (`BT-7`)
/// or as a code for the event that fixes it (`BT-8`).
///
/// The two forms are mutually exclusive (`BR-CO-03`): the fold encodes the choice so that
/// neither "both" nor "neither" is representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VatPoint {
    /// An explicit VAT point date (`BT-7`).
    Date(Date),
    /// The event that fixes the VAT point date (`BT-8`).
    Event(Event),
}

/// The VAT point event (`BT-8`, subset of `UNTDID 2005`):
/// the event that fixes the date the VAT becomes chargeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Event {
    Invoicing = 3,
    Delivery = 35,
    Payment = 432,
}

impl Display for Event {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", u16::from(*self))
    }
}

impl FromStr for Event {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u16>()
            .ok()
            .and_then(|code| Self::try_from(code).ok())
            .ok_or_else(|| Error::InvalidValue(format!("{value:?}")))
    }
}

impl TryFrom<&str> for Event {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl From<Event> for u32 {
    fn from(code: Event) -> Self {
        u16::from(code).into()
    }
}

impl TryFrom<u32> for Event {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        u16::try_from(value)
            .ok()
            .and_then(|code| Self::try_from(code).ok())
            .ok_or_else(|| Error::InvalidValue(format!("{value:?}")))
    }
}

impl From<Date> for VatPoint {
    fn from(value: Date) -> Self {
        Self::Date(value)
    }
}

impl From<Event> for VatPoint {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

impl TryFrom<u16> for VatPoint {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Event::try_from(value)
            .map(Self::Event)
            .map_err(|_| Error::InvalidValue(format!("{value:?}")))
    }
}

impl TryFrom<u32> for VatPoint {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Event::try_from(value).map(Self::Event)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_known_code() {
        let event: Event = "35".parse().expect("35 is a valid VAT point event");

        assert_eq!(event, Event::Delivery);
        assert_eq!(event.to_string(), "35");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("1".parse::<Event>().is_err());
    }

    #[test]
    fn converts_an_event_via_try_from() {
        assert_eq!(
            Event::try_from("432").expect("432 is a valid VAT point event"),
            Event::Payment
        );
        assert!(matches!(Event::try_from("1"), Err(Error::InvalidValue(_))));
    }

    #[test]
    fn builds_a_vat_point_from_a_numeric_code() {
        assert_eq!(
            VatPoint::try_from(35u16).expect("35 is a valid VAT point event"),
            VatPoint::Event(Event::Delivery)
        );
        assert!(matches!(
            VatPoint::try_from(1u32),
            Err(Error::InvalidValue(_))
        ));
    }
}
