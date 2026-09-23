//! An item attribute (`BG-32`).

use crate::NonEmptyString;

/// A named characteristic of the item, such as a color or a size (`BG-32`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemAttribute {
    /// Name (`BT-160`).
    pub name: Option<NonEmptyString>,
    /// Value (`BT-161`).
    pub value: Option<NonEmptyString>,
}
