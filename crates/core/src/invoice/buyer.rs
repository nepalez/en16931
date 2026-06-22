use crate::{
    Contact, ElectronicAddress, LegalEntity, NonEmptyString, OperationalEntity, PostalAddress,
    Seller, VatIdentifier,
};

/// The buyer (`BG-7`): the party that receives the invoice and the goods or services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buyer {
    /// Buyer name (`BT-44`).
    pub name: NonEmptyString,
    /// Buyer trading name (`BT-45`).
    pub trading_name: Option<NonEmptyString>,
    /// Buyer identifiers (`BT-46`): alternative identifiers of the same party.
    pub identifiers: Vec<OperationalEntity>,
    /// Buyer legal registration (`BT-47`).
    pub legal_entity: Option<LegalEntity>,
    /// Buyer VAT identifier (`BT-48`).
    pub vat: Option<VatIdentifier>,
    /// Buyer electronic address (`BT-49`).
    pub electronic_address: Option<ElectronicAddress>,
    /// Buyer postal address (`BG-8`).
    pub address: PostalAddress,
    /// Buyer contact (`BG-9`).
    pub contact: Option<Contact>,
}

/// Re-casts the same organization from the seller role into the buyer role (a resale).
/// The seller-only fields (tax registration, additional legal information) are dropped.
impl From<Seller> for Buyer {
    fn from(seller: Seller) -> Self {
        Self {
            name: seller.name,
            trading_name: seller.trading_name,
            identifiers: seller.identifiers,
            legal_entity: seller.legal_entity,
            vat: seller.vat,
            electronic_address: seller.electronic_address,
            address: seller.address,
            contact: seller.contact,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CountryCode;

    #[test]
    fn demotes_a_seller_to_a_buyer() {
        let seller = Seller {
            name: "Acme".parse().expect("a valid name"),
            trading_name: None,
            identifiers: Vec::new(),
            legal_entity: None,
            additional_legal_information: Some("share capital".parse().expect("a value")),
            vat: None,
            tax_registration: Some("TAX-1".parse().expect("a value")),
            electronic_address: None,
            address: PostalAddress {
                line1: None,
                line2: None,
                line3: None,
                city: None,
                postal_code: None,
                country_subdivision: None,
                country: CountryCode::for_alpha2("DE").expect("DE is a country code"),
            },
            contact: None,
        };

        let buyer = Buyer::from(seller);

        assert_eq!(buyer.name.as_ref(), "Acme");
        assert!(buyer.vat.is_none());
    }
}
