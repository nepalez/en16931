use crate::{
    Contact, ElectronicAddress, LegalEntity, NonEmptyString, OperationalEntity, PostalAddress,
    Seller, VatIdentifier,
};

/// The buyer (`BG-7`): the party that receives the invoice and the goods or services.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Buyer {
    /// Buyer name (`BT-44`).
    pub name: Option<NonEmptyString>,
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
    pub address: Option<PostalAddress>,
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

    #[test]
    fn demotes_a_seller_to_a_buyer() {
        let seller = Seller {
            name: Some("Acme".parse().expect("a valid name")),
            additional_legal_information: Some("share capital".parse().expect("a value")),
            tax_registration: Some("TAX-1".parse().expect("a value")),
            ..Default::default()
        };

        let buyer = Buyer::from(seller);

        assert_eq!(
            buyer.name,
            Some("Acme".parse::<NonEmptyString>().expect("a valid name"))
        );
        assert!(buyer.vat.is_none());
    }
}
