use crate::prelude::*;
use crate::{LegalEntity, NonEmptyString, OperationalEntity};

/// The payee (`BG-10`): the party to which the payment is due,
/// when it is not the seller.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Payee {
    /// Payee name (`BT-59`).
    pub name: NonEmptyString,
    /// Payee identifiers (`BT-60`): alternative identifiers of the same party.
    #[builder(default)]
    pub identifiers: Vec<OperationalEntity>,
    /// Payee legal registration (`BT-61`).
    pub legal_entity: Option<LegalEntity>,
}
