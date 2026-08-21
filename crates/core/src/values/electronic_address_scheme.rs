//! The electronic address scheme code of an endpoint (`BT-34`/`BT-49`, CEF EAS).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-25`.
//! Variant names follow [CEF EAS](https://docs.peppol.eu/poacc/billing/3.0/codelist/eas/).

use crate::Error;
use crate::prelude::*;

/// The electronic address scheme (`BT-34`/`BT-49`, CEF EAS): the scheme of an endpoint identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
pub enum ElectronicAddressScheme {
    #[display("0002")]
    FrenchSirene,
    #[display("0007")]
    SwedishOrganisationNumber,
    #[display("0009")]
    FrenchSiret,
    #[display("0037")]
    FinnishBusinessId,
    #[display("0060")]
    Duns,
    #[display("0088")]
    EanLocationCode,
    #[display("0096")]
    DanishChamberOfCommerce,
    #[display("0097")]
    ItalianEdiforum,
    #[display("0106")]
    DutchKvk,
    #[display("0130")]
    EuCommissionDirectorate,
    #[display("0135")]
    Sia,
    #[display("0142")]
    Seceti,
    #[display("0147")]
    StandardCompanyCode,
    #[display("0151")]
    AustralianAbn,
    #[display("0154")]
    CzechIco,
    #[display("0158")]
    SlovakIco,
    #[display("0170")]
    TeikokuCompanyCode,
    #[display("0177")]
    OdetteInternationalLimited,
    #[display("0183")]
    Uidb,
    #[display("0184")]
    DanishDigstorg,
    #[display("0188")]
    JapaneseCorporateNumber,
    #[display("0190")]
    Oin,
    #[display("0191")]
    EstonianCompanyCode,
    #[display("0192")]
    NorwegianOrganisationNumber,
    #[display("0193")]
    UblBePartyIdentifier,
    #[display("0194")]
    KoiosOpenTechnicalDictionary,
    #[display("0195")]
    SingaporeEInvoice,
    #[display("0196")]
    IcelandicKennitala,
    #[display("0198")]
    Erstorg,
    #[display("0199")]
    Lei,
    #[display("0200")]
    LithuanianLegalEntityCode,
    #[display("0201")]
    ItalianIpaUnit,
    #[display("0202")]
    ItalianPec,
    #[display("0203")]
    EdeliveryParticipant,
    #[display("0204")]
    LeitwegId,
    #[display("0205")]
    Coddest,
    #[display("0208")]
    BelgianEnterpriseNumber,
    #[display("0209")]
    Gs1IdentificationKeys,
    #[display("0210")]
    ItalianFiscalCode,
    #[display("0211")]
    ItalianVat,
    #[display("0212")]
    FinnishOrganizationIdentifier,
    #[display("0213")]
    FinnishOrganizationVat,
    #[display("0215")]
    NetServiceId,
    #[display("0216")]
    Ovtcode,
    #[display("0217")]
    DutchEstablishmentNumber,
    #[display("0218")]
    LatvianRegistrationNumber,
    #[display("0219")]
    LatvianTaxpayerCode,
    #[display("0220")]
    LatvianNaturalPersonRegister,
    #[display("0221")]
    JapaneseInvoiceIssuer,
    #[display("0225")]
    FrctcElectronicAddress,
    #[display("0230")]
    NationalEInvoicingFramework,
    #[display("0235")]
    Tin,
    #[display("0240")]
    FrenchLegalEntityRegister,
    #[display("0244")]
    NigerianTaxId,
    #[display("0242")]
    Spis,
    #[display("0245")]
    SlovakVatId,
    #[display("0246")]
    ICD0246,
    #[display("0248")]
    ICD0248,
    #[display("9910")]
    HungaryVatNumber,
    #[display("9913")]
    BusinessRegistersNetwork,
    #[display("9914")]
    AustriaVatNumber,
    #[display("9915")]
    AustriaOrganisationCode,
    #[display("9918")]
    Swift,
    #[display("9919")]
    AustriaCompanyRegister,
    #[display("9920")]
    SpainVatNumber,
    #[display("9922")]
    AndorraVatNumber,
    #[display("9923")]
    AlbaniaVatNumber,
    #[display("9924")]
    BosniaVatNumber,
    #[display("9925")]
    BelgiumVatNumber,
    #[display("9926")]
    BulgariaVatNumber,
    #[display("9927")]
    SwitzerlandVatNumber,
    #[display("9928")]
    CyprusVatNumber,
    #[display("9929")]
    CzechRepublicVatNumber,
    #[display("9930")]
    GermanyVatNumber,
    #[display("9931")]
    EstoniaVatNumber,
    #[display("9932")]
    UnitedKingdomVatNumber,
    #[display("9933")]
    GreeceVatNumber,
    #[display("9934")]
    CroatiaVatNumber,
    #[display("9935")]
    IrelandVatNumber,
    #[display("9936")]
    LiechtensteinVatNumber,
    #[display("9937")]
    LithuaniaVatNumber,
    #[display("9938")]
    LuxemburgVatNumber,
    #[display("9939")]
    LatviaVatNumber,
    #[display("9940")]
    MonacoVatNumber,
    #[display("9941")]
    MontenegroVatNumber,
    #[display("9942")]
    MacedoniaVatNumber,
    #[display("9943")]
    MaltaVatNumber,
    #[display("9944")]
    NetherlandsVatNumber,
    #[display("9945")]
    PolandVatNumber,
    #[display("9946")]
    PortugalVatNumber,
    #[display("9947")]
    RomaniaVatNumber,
    #[display("9948")]
    SerbiaVatNumber,
    #[display("9949")]
    SloveniaVatNumber,
    #[display("9950")]
    SlovakiaVatNumber,
    #[display("9951")]
    SanMarinoVatNumber,
    #[display("9952")]
    TurkeyVatNumber,
    #[display("9953")]
    VaticanVatNumber,
    #[display("9957")]
    FrenchVatNumber,
    #[display("9959")]
    UsVatNumber,
    #[display("AN")]
    Oftp,
    #[display("AQ")]
    X400,
    #[display("AS")]
    As2,
    #[display("AU")]
    Ftp,
    #[display("EM")]
    Email,
}

impl TryFrom<&str> for ElectronicAddressScheme {
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
        let scheme: ElectronicAddressScheme = "0088"
            .parse()
            .expect("0088 is a valid electronic address scheme");

        assert_eq!(scheme, ElectronicAddressScheme::EanLocationCode);
        assert_eq!(scheme.to_string(), "0088");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("0003".parse::<ElectronicAddressScheme>().is_err());
    }

    #[test]
    fn converts_via_try_from() {
        assert_eq!(
            ElectronicAddressScheme::try_from("0060")
                .expect("0060 is a valid electronic address scheme"),
            ElectronicAddressScheme::Duns
        );
        assert!(matches!(
            ElectronicAddressScheme::try_from("ZZ"),
            Err(Error::InvalidValue { .. })
        ));
    }
}
