use crate::{
    Buyer, Contact, ElectronicAddress, LegalEntity, NonEmptyString, OperationalEntity,
    PostalAddress, VatIdentifier,
};

/// The seller (`BG-4`): the party that issues the invoice and supplies the goods or services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seller {
    /// Seller name (`BT-27`).
    pub name: NonEmptyString,
    /// Seller trading name (`BT-28`).
    pub trading_name: Option<NonEmptyString>,
    /// Seller identifiers (`BT-29`): alternative identifiers of the same party.
    pub identifiers: Vec<OperationalEntity>,
    /// Seller legal registration (`BT-30`).
    pub legal_entity: Option<LegalEntity>,
    /// Seller additional legal information (`BT-33`).
    pub additional_legal_information: Option<NonEmptyString>,
    /// Seller VAT identifier (`BT-31`).
    pub vat: Option<VatIdentifier>,
    /// Seller tax registration identifier (`BT-32`).
    pub tax_registration: Option<NonEmptyString>,
    /// Seller electronic address (`BT-34`).
    pub electronic_address: Option<ElectronicAddress>,
    /// Seller postal address (`BG-5`).
    pub address: PostalAddress,
    /// Seller contact (`BG-6`).
    pub contact: Option<Contact>,
}

/// Re-casts the same organization from the buyer role into the seller role (a resale).
/// The seller-only fields start empty and may be filled afterwards.
impl From<Buyer> for Seller {
    fn from(buyer: Buyer) -> Self {
        Self {
            name: buyer.name,
            trading_name: buyer.trading_name,
            identifiers: buyer.identifiers,
            legal_entity: buyer.legal_entity,
            additional_legal_information: None,
            vat: buyer.vat,
            tax_registration: None,
            electronic_address: buyer.electronic_address,
            address: buyer.address,
            contact: buyer.contact,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CountryCode;

    #[test]
    fn promotes_a_buyer_to_a_seller() {
        let buyer = Buyer {
            name: "Acme".parse().expect("a valid name"),
            trading_name: None,
            identifiers: Vec::new(),
            legal_entity: None,
            vat: None,
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

        let seller = Seller::from(buyer);

        assert_eq!(seller.name.as_ref(), "Acme");
        assert!(seller.tax_registration.is_none());
        assert!(seller.additional_legal_information.is_none());
    }
}
