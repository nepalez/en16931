use crate::NonEmptyString;

/// An item attribute (`BG-32`): a named characteristic of the item,
/// such as a color or a size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttribute {
    /// Name (`BT-160`).
    pub name: NonEmptyString,
    /// Value (`BT-161`).
    pub value: NonEmptyString,
}
