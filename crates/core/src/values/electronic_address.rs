//! The electronic address of a party (`BT-34` seller, `BT-49` buyer).

use crate::{ElectronicAddressScheme, NonEmptyString};

/// The endpoint a party is reached at, identified under a CEF EAS scheme
/// (`BT-34` seller, `BT-49` buyer).
///
/// An electronic address is resolvable only by naming its scheme,
/// which the validator requires.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ElectronicAddress {
    /// The electronic address value.
    pub id: Option<NonEmptyString>,
    /// The address scheme (`schemeID`).
    pub scheme: Option<ElectronicAddressScheme>,
}
