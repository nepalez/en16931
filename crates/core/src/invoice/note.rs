use crate::NonEmptyString;
use crate::prelude::*;

/// An invoice note (`BG-1`): a free-text remark, optionally tagged with a subject code.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Note {
    /// Subject code (`BT-21`).
    pub subject_code: Option<NonEmptyString>,
    /// Text (`BT-22`).
    pub text: NonEmptyString,
}
