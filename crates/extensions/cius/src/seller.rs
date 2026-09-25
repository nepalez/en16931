//! The seller of the base invoice (`BG-4`).

use crate::prelude::*;
use crate::{Buyer, Contact};

/// The party that issues the invoice and supplies the goods or services (`BG-4`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seller {
    /// Seller name (`BT-27`).
    pub name: Option<NonEmptyString>,
    /// Seller trading name (`BT-28`).
    pub trading_name: Option<NonEmptyString>,
    /// Alternative identifiers of the same party (`BT-29`).
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
    pub address: Option<PostalAddress>,
    /// Seller contact (`BG-6`).
    pub contact: Option<Contact>,
}

impl crate::prelude::Seller for Seller {
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

    fn tax_registration(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.tax_registration
    }

    fn additional_legal_information(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.additional_legal_information
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

    #[test]
    fn promotes_a_buyer_to_a_seller() {
        let buyer = Buyer {
            name: Some("Acme".parse().expect("a valid name")),
            ..Default::default()
        };

        let seller = Seller::from(buyer);

        assert_eq!(
            seller.name,
            Some("Acme".parse::<NonEmptyString>().expect("a valid name"))
        );
        assert!(seller.tax_registration.is_none());
        assert!(seller.additional_legal_information.is_none());
    }
}
