//! An invoice note (`BG-1`).

use crate::NonEmptyString;

/// A free-text remark, optionally tagged with a subject code (`BG-1`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Note {
    /// Subject code (`BT-21`).
    pub subject_code: Option<NonEmptyString>,
    /// Text (`BT-22`).
    pub text: Option<NonEmptyString>,
}
