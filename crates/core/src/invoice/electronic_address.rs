//! The electronic address of a party (`BT-34` seller, `BT-49` buyer).

use crate::{ElectronicAddressScheme, NonEmptyString};

/// A party's electronic address (`BT-34` seller, `BT-49` buyer):
/// the endpoint a party is reached at, identified under a CEF EAS scheme.
///
/// The scheme is mandatory: an electronic address is resolvable only by naming its scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronicAddress {
    /// The electronic address value.
    pub id: NonEmptyString,
    /// The address scheme (`schemeID`).
    pub scheme: ElectronicAddressScheme,
}
