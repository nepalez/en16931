//! The interface of the payee (`BG-10`).

use crate::{LegalEntity, NonEmptyString, OperationalEntity};

/// The party to which the payment is due, when it is not the seller (`BG-10`).
pub trait Payee {
    /// Payee name (`BT-59`).
    fn name(&mut self) -> &mut Option<NonEmptyString>;
    /// Alternative identifiers of the same party (`BT-60`).
    fn identifiers(&mut self) -> &mut Vec<OperationalEntity>;
    /// Payee legal registration (`BT-61`).
    fn legal_entity(&mut self) -> &mut Option<LegalEntity>;
}
