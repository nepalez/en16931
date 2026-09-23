//! The interface of the buyer (`BG-7`).

use crate::{
    Contact, ElectronicAddress, LegalEntity, NonEmptyString, OperationalEntity, PostalAddress,
    VatIdentifier,
};

/// The party that receives the invoice and the goods or services (`BG-7`).
pub trait Buyer {
    /// The type of the buyer contact (`BG-9`).
    type Contact: Contact;

    /// Buyer name (`BT-44`).
    fn name(&mut self) -> &mut Option<NonEmptyString>;
    /// Buyer trading name (`BT-45`).
    fn trading_name(&mut self) -> &mut Option<NonEmptyString>;
    /// Alternative identifiers of the same party (`BT-46`).
    fn identifiers(&mut self) -> &mut Vec<OperationalEntity>;
    /// Buyer legal registration (`BT-47`).
    fn legal_entity(&mut self) -> &mut Option<LegalEntity>;
    /// Buyer VAT identifier (`BT-48`).
    fn vat(&mut self) -> &mut Option<VatIdentifier>;
    /// Buyer electronic address (`BT-49`).
    fn electronic_address(&mut self) -> &mut Option<ElectronicAddress>;
    /// Buyer postal address (`BG-8`).
    fn address(&mut self) -> &mut Option<PostalAddress>;
    /// Buyer contact (`BG-9`).
    fn contact(&mut self) -> &mut Option<Self::Contact>;
}
