//! The interface of the seller (`BG-4`).

use crate::{
    Contact, ElectronicAddress, LegalEntity, NonEmptyString, OperationalEntity, PostalAddress,
    VatIdentifier,
};

/// The party that issues the invoice and supplies the goods or services (`BG-4`).
pub trait Seller {
    /// The type of the seller contact (`BG-6`).
    type Contact: Contact;

    /// Seller name (`BT-27`).
    fn name(&mut self) -> &mut Option<NonEmptyString>;
    /// Seller trading name (`BT-28`).
    fn trading_name(&mut self) -> &mut Option<NonEmptyString>;
    /// Alternative identifiers of the same party (`BT-29`).
    fn identifiers(&mut self) -> &mut Vec<OperationalEntity>;
    /// Seller legal registration (`BT-30`).
    fn legal_entity(&mut self) -> &mut Option<LegalEntity>;
    /// Seller VAT identifier (`BT-31`).
    fn vat(&mut self) -> &mut Option<VatIdentifier>;
    /// Seller tax registration identifier (`BT-32`).
    fn tax_registration(&mut self) -> &mut Option<NonEmptyString>;
    /// Seller additional legal information (`BT-33`).
    fn additional_legal_information(&mut self) -> &mut Option<NonEmptyString>;
    /// Seller electronic address (`BT-34`).
    fn electronic_address(&mut self) -> &mut Option<ElectronicAddress>;
    /// Seller postal address (`BG-5`).
    fn address(&mut self) -> &mut Option<PostalAddress>;
    /// Seller contact (`BG-6`).
    fn contact(&mut self) -> &mut Option<Self::Contact>;
}
