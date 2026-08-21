//! The identifier scheme code (ISO 6523 ICD) that accompanies a party, legal
//! entity, item, or delivery-location identifier (`schemeID`).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rules `BR-CL-10`/`BR-CL-11`/`BR-CL-21`/`BR-CL-26`.
//! Variant names follow [ISO 6523 ICD](https://docs.peppol.eu/poacc/billing/3.0/codelist/ICD/).

use crate::Error;
use crate::prelude::*;

/// The ISO 6523 ICD scheme (`schemeID`): the agency that issues an accompanying identifier.
/// The value is rendered as a zero-padded four-digit string (e.g. `0088`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum IssuingAgency {
    FrenchSirene = 2,
    BelgianFinancialInstitution = 3,
    NbsOsiNetwork = 4,
    UsFedGovOsiNetwork = 5,
    UsDodOsiNetwork = 6,
    SwedishOrganisationNumber = 7,
    BelgianNationalNumber = 8,
    FrenchSiret = 9,
    ISO9541 = 10,
    AmateurRadioOsi = 11,
    Ecma = 12,
    VsaFtp = 13,
    NistOsiImplememtsWorkshop = 14,
    Edi = 15,
    Ewos = 16,
    CommonLanguage = 17,
    SnaOsiNetwork = 18,
    AirTransportNetwork = 19,
    Cern = 20,
    Swift = 21,
    OsfDce = 22,
    Nordunet = 23,
    Dec = 24,
    OsiAsiaOceaniaWorkshop = 25,
    NatoISO6523IcdeCodingScheme = 26,
    Atn = 27,
    ISO6523 = 28,
    Okpo = 29,
    AtTOsiNetwork = 30,
    EdiPartnerIdentificationCode = 31,
    TelecomAustralia = 32,
    SGWOsiInternetwork = 33,
    ReuterOpenAddressStandard = 34,
    ISO6523Icd = 35,
    Teletrust = 36,
    FinnishBusinessId = 37,
    TheAustralianGosipNetwork = 38,
    TheOzDodOsiNetwork = 39,
    Unilever = 40,
    CiticorpNetwork = 41,
    DbpTelekom = 42,
    Hydronett = 43,
    Tisi = 44,
    Ici = 45,
    Funloc = 46,
    BullOdiDsaUnixNetwork = 47,
    Osinz = 48,
    AucklandAreaHealth = 49,
    Firmenich = 50,
    AgfaDis = 51,
    Smpte = 52,
    MigrosNetworkMNetopz = 53,
    ISO6523Icdpcr = 54,
    EnergyNet = 55,
    Noi = 56,
    SaintGobain = 57,
    Siemens = 58,
    Danznet = 59,
    Duns = 60,
    SoffexOsi = 61,
    KpnOvn = 62,
    Ascomosinet = 63,
    UniformeTransportCode = 64,
    SolvayOsiCoding = 65,
    Roche = 66,
    Zellwegerosinet = 67,
    IntelCorporationOsi = 68,
    Sita = 69,
    DaimlerChryslerNetwork = 70,
    Lego = 71,
    Navistar = 72,
    IcdFormattedAtmAddress = 73,
    Arinc = 74,
    AlcatelNetwork = 75,
    ItalianUninfoObject = 76,
    ItalianUninfoOsi = 77,
    MitelEquipment = 78,
    AtmForum = 79,
    UkNhs = 80,
    InternationalNsap = 81,
    NorwegianNta = 82,
    Atml = 83,
    AthensChamberOfCommerce = 84,
    SwissChamberOfCommerce = 85,
    Uscib = 86,
    BelgianChamberOfCommerce = 87,
    EanLocationCode = 88,
    BritishChamberOfCommerce = 89,
    InternetIpAddressing = 90,
    Cisco = 91,
    CanadianBusinessNumber = 93,
    GermanDiht = 94,
    HpInternalNetwork = 95,
    DanishChamberOfCommerce = 96,
    ItalianEdiforum = 97,
    TelAvivChamberOfCommerce = 98,
    SiemensSupervisoryNetwork = 99,
    PngIcdScheme = 100,
    SouthAfricanCodeAllocation = 101,
    Heag = 102,
    Bt = 104,
    PortugueseChamberOfCommerce = 105,
    DutchKvk = 106,
    SwedishChamberOfCommerce = 107,
    AustralianChamberOfCommerce = 108,
    BellSouthAesa = 109,
    BellAtlantic = 110,
    ObjectIdentifiers = 111,
    IsoStandardsRegister = 112,
    Originnet = 113,
    CheckPoint = 114,
    PacificBellNetwork = 115,
    Pss = 116,
    Stentor = 117,
    AtmNetworkZn96 = 118,
    Mci = 119,
    Advantis = 120,
    AffableSoftware = 121,
    BbDataGmbh = 122,
    Basf = 123,
    Iota = 124,
    Henkel = 125,
    Gte = 126,
    DresdnerBank = 127,
    BcnrSwissClearingBankNumber = 128,
    SwissBpi = 129,
    EuCommissionDirectorate = 130,
    NationalOrganizationCode = 131,
    Certicom = 132,
    Tc68Oid = 133,
    Infonet = 134,
    Sia = 135,
    CableWirelessAtm = 136,
    GlobalAesaScheme = 137,
    FranceTelecomAtm = 138,
    ICD0139 = 139,
    Topas = 140,
    NatoCage = 141,
    Seceti = 142,
    EinesteinetAg = 143,
    Dodaac = 144,
    FrenchDgcp = 145,
    FrenchDgi = 146,
    StandardCompanyCode = 147,
    Dnic = 148,
    GlobalBusinessIdentifier = 149,
    MadgeNetworksAtm = 150,
    AustralianAbn = 151,
    EdiraSchemeIdentifierCode = 152,
    ConcertAesa = 153,
    CzechIco = 154,
    GlobalCrossingAesa = 155,
    Auna = 156,
    DutchKpnAtm = 157,
    SlovakIco = 158,
    Actalis = 159,
    GtinGlobalTradeItemNumber = 160,
    EccmaOpenTechnicalDirectory = 161,
    CenIsss = 162,
    UsEpaFacilityIdentifier = 163,
    TelusCorporation = 164,
    Fieie = 165,
    SwissguideIdentifierScheme = 166,
    PriorityTelecomAtm = 167,
    VodafoneIrelandOsiAddressing = 168,
    SwissZefix = 169,
    TeikokuCompanyCode = 170,
    LuxembourgCpCps = 171,
    Prolist = 172,
    EciSs = 173,
    Stepnexus = 174,
    SiemensAg = 175,
    ParadineGmbh = 176,
    OdetteInternationalLimited = 177,
    Route1Mobinet = 178,
    Penango = 179,
    LithuanianMilitaryPki = 180,
    Uidb = 183,
    DanishDigstorg = 184,
    PercevalObjectCode = 185,
    Trustpoint = 186,
    AmazonId = 187,
    JapaneseCorporateNumber = 188,
    Ebid = 189,
    Oin = 190,
    EstonianCompanyCode = 191,
    NorwegianOrganisationNumber = 192,
    UblBePartyIdentifier = 193,
    KoiosOpenTechnicalDictionary = 194,
    SingaporeEInvoice = 195,
    IcelandicKennitala = 196,
    AppliaPlStandard = 197,
    Erstorg = 198,
    Lei = 199,
    LithuanianLegalEntityCode = 200,
    ItalianIpaUnit = 201,
    ItalianPec = 202,
    EdeliveryParticipant = 203,
    LeitwegId = 204,
    Coddest = 205,
    Rci = 206,
    Poci = 207,
    BelgianEnterpriseNumber = 208,
    Gs1IdentificationKeys = 209,
    ItalianFiscalCode = 210,
    ItalianVat = 211,
    FinnishOrganizationIdentifier = 212,
    FinnishOrganizationVat = 213,
    TradeplaceTradepiStandard = 214,
    NetServiceId = 215,
    Ovtcode = 216,
    DutchEstablishmentNumber = 217,
    LatvianRegistrationNumber = 218,
    LatvianTaxpayerCode = 219,
    LatvianNaturalPersonRegister = 220,
    JapaneseInvoiceIssuer = 221,
    MetadataRegistrySupport = 222,
    EuBasedCompany = 223,
    FtctcCodeRoutage = 224,
    FrctcElectronicAddress = 225,
    FrctcParticulier = 226,
    NonEuBasedCompany = 227,
    Ridet = 228,
    Tahiti = 229,
    NationalEInvoicingFramework = 230,
    FrenchSingleTaxableCompany = 231,
    NorwegianNobb = 232,
    NorwegianElnummer = 233,
    ToimitusosoiteId = 234,
    Tin = 235,
    Toimipaikkald = 236,
    DanishCpr = 237,
    FrenchPpfPdp = 238,
    Eaeu = 239,
    FrenchLegalEntityRegister = 240,
    ICD0241 = 241,
    Spis = 242,
    ICD0243 = 243,
    NigerianTaxId = 244,
    SlovakVatId = 245,
    ICD0246 = 246,
    ICD0247 = 247,
    ICD0248 = 248,
}

impl Display for IssuingAgency {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:04}", u16::from(*self))
    }
}

impl FromStr for IssuingAgency {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u16>()
            .ok()
            .and_then(|code| Self::try_from(code).ok())
            .ok_or_else(|| Error::invalid_value(format!("{value:?}")))
    }
}

impl TryFrom<&str> for IssuingAgency {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl From<IssuingAgency> for u32 {
    fn from(code: IssuingAgency) -> Self {
        u16::from(code).into()
    }
}

impl TryFrom<u32> for IssuingAgency {
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
    fn parses_a_known_code() {
        let scheme: IssuingAgency = "0002".parse().expect("0002 is a valid issuing agency");

        assert_eq!(scheme, IssuingAgency::FrenchSirene);
        assert_eq!(scheme.to_string(), "0002");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("9999".parse::<IssuingAgency>().is_err());
    }

    #[test]
    fn converts_via_try_from() {
        assert_eq!(
            IssuingAgency::try_from("0012").expect("0012 is a valid issuing agency"),
            IssuingAgency::Ecma
        );
        assert!(matches!(
            IssuingAgency::try_from("9999"),
            Err(Error::InvalidValue { .. })
        ));
    }
}
