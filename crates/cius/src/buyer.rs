//! The buyer of the base invoice (`BG-7`).

use crate::prelude::*;
use crate::{Contact, Seller};

/// The party that receives the invoice and the goods or services (`BG-7`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Buyer {
    /// Buyer name (`BT-44`).
    pub name: Option<NonEmptyString>,
    /// Buyer trading name (`BT-45`).
    pub trading_name: Option<NonEmptyString>,
    /// Alternative identifiers of the same party (`BT-46`).
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

impl crate::prelude::Buyer for Buyer {
    type Contact = Contact;

    fn name(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.name
    }

    fn trading_name(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.trading_name
    }

    fn identifiers(&mut self) -> &mut Vec<OperationalEntity> {
        &mut self.identifiers
    }

    fn legal_entity(&mut self) -> &mut Option<LegalEntity> {
        &mut self.legal_entity
    }

    fn vat(&mut self) -> &mut Option<VatIdentifier> {
        &mut self.vat
    }

    fn electronic_address(&mut self) -> &mut Option<ElectronicAddress> {
        &mut self.electronic_address
    }

    fn address(&mut self) -> &mut Option<PostalAddress> {
        &mut self.address
    }

    fn contact(&mut self) -> &mut Option<Contact> {
        &mut self.contact
    }
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
