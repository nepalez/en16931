//! The payee of the base invoice (`BG-10`).

use crate::prelude::*;

/// The party to which the payment is due, when it is not the seller (`BG-10`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Payee {
    /// Payee name (`BT-59`).
    pub name: Option<NonEmptyString>,
    /// Alternative identifiers of the same party (`BT-60`).
    pub identifiers: Vec<OperationalEntity>,
    /// Payee legal registration (`BT-61`).
    pub legal_entity: Option<LegalEntity>,
}

impl crate::prelude::Payee for Payee {
    fn name(&mut self) -> &mut Option<NonEmptyString> {
        &mut self.name
    }

    fn identifiers(&mut self) -> &mut Vec<OperationalEntity> {
        &mut self.identifiers
    }

    fn legal_entity(&mut self) -> &mut Option<LegalEntity> {
        &mut self.legal_entity
    }
}
