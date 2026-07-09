//! The profiles of the core: the `BT-24` a document declares and the terms it forbids.

use crate::Term;
use crate::prelude::*;

/// A supported profile: base EN-16931 or one of its CIUS restrictions and extensions.
///
/// A profile is a core component, not an extension axis.
/// It adds no vocabulary and only stamps its `BT-24` specification identifier on a document
/// and names the superset terms it forbids there.
///
/// The forbidden set is declarative data.
/// It covers the unconditional prohibitions only.
/// Conditional, relational, and code-list rules stay with the external validator.
///
/// The variant renders as its `BT-24` string and parses back from it.
/// Every version of a profile family is a distinct variant,
/// since it carries its own `BT-24` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
pub enum Profile {
    #[display(
        "urn:cen.eu:en16931:2017#compliant#urn:erechnung.gv.at:CIUS-AT-NAT:1.0.0#compliant#urn:erechnung.gv.at:CIUS-AT-GOV:1.1.0"
    )]
    CiusAtGov110,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:erechnung.gv.at:CIUS-AT-NAT:1.0.0")]
    CiusAtNat100,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:face.gob.es:CIUS-ES-FACE:1.0.0")]
    CiusEsFace100,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:fatturapa.gov.it:CIUS-IT:2.0.0")]
    CiusIt200,
    #[display("urn:cen.eu:en16931:2017")]
    En16931,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic")]
    FacturXBasic,
    #[display("urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended")]
    FacturXExtended,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:fdc:nen.nl:nlcius:v1.0")]
    Nlcius10,
    #[display(
        "urn:cen.eu:en16931:2017#compliant#urn:fdc:nen.nl:nlcius:v1.0#conformant#urn:fdc:nen.nl:gaccount:v1.0"
    )]
    Nlcius10GAccount,
    #[display("urn:fdc:oioubl.dk:trns:billing:invoice:3.0")]
    Oioubl3Invoice,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0")]
    PeppolBisBilling30,
    #[display(
        "urn:cen.eu:en16931:2017#conformant#urn:fdc:peppol.eu:2017:poacc:billing:international:aunz:3.0"
    )]
    PeppolBisBillingInternationalAunz30,
    #[display(
        "urn:cen.eu:en16931:2017#conformant#urn:fdc:peppol.eu:2017:poacc:billing:international:sg:3.0"
    )]
    PeppolBisBillingInternationalSg30,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:selfbilling:3.0")]
    PeppolBisSelfBilling30,
    #[display(
        "urn:cen.eu:en16931:2017#conformant#urn:fdc:peppol.eu:2017:poacc:selfbilling:international:aunz:3.0"
    )]
    PeppolBisSelfBillingInternationalAunz30,
    #[display("urn:peppol:pint:billing-1@ae-1")]
    PintAeBilling,
    #[display("urn:peppol:pint:selfbilling-1@ae-1")]
    PintAeSelfBilling,
    #[display("urn:peppol:pint:billing-1@aunz-1")]
    PintAunzBilling,
    #[display("urn:peppol:pint:selfbilling-1@aunz-1")]
    PintAunzSelfBilling,
    #[display("urn:peppol:pint:billing-1@eu-1")]
    PintEuBilling,
    #[display("urn:peppol:pint:billing-1@jp-1")]
    PintJpBilling,
    #[display("urn:peppol:pint:billing-1@my-1")]
    PintMyBilling,
    #[display("urn:peppol:pint:selfbilling-1@my-1")]
    PintMySelfBilling,
    #[display("urn:cen.eu:en16931:2017#conformant#urn:UBL.BE:1.0.0.20180214")]
    UblBe,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_1.2")]
    XRechnung12,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.0")]
    XRechnung20,
    #[display(
        "urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.0#conformant#urn:xoev-de:kosit:extension:xrechnung_2.0"
    )]
    XRechnung20Extension,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.1")]
    XRechnung21,
    #[display(
        "urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.1#conformant#urn:xoev-de:kosit:extension:xrechnung_2.1"
    )]
    XRechnung21Extension,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.2")]
    XRechnung22,
    #[display(
        "urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.2#conformant#urn:xoev-de:kosit:extension:xrechnung_2.2"
    )]
    XRechnung22Extension,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.3")]
    XRechnung23,
    #[display(
        "urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.3#conformant#urn:xoev-de:kosit:extension:xrechnung_2.3"
    )]
    XRechnung23Extension,
    #[display("urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0")]
    XRechnung30,
    #[display(
        "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0"
    )]
    XRechnung30Extension,
}

impl Profile {
    /// The terms the profile forbids in a serialized document.
    pub fn forbidden_terms(&self) -> &'static [Term] {
        match self {
            Self::PeppolBisBilling30 => &[Term::BT(21)],
            Self::CiusAtNat100 | Self::CiusAtGov110 => &[Term::BT(124)],
            _ => &[],
        }
    }

    /// Whether the profile forbids the given term.
    pub fn forbids(&self, term: Term) -> bool {
        self.forbidden_terms().contains(&term)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rejects_an_unknown_identifier() {
        assert!("urn:example:unknown".parse::<Profile>().is_err());
    }
}
