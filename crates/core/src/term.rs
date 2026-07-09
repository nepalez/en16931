use crate::prelude::*;

/// A business term of EN-16931, named by its `BT`/`BG` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Term {
    /// A business term (`BT-n`): a single leaf field.
    #[display("BT-{0}")]
    BT(u16),
    /// A business group (`BG-n`): a repeatable or aggregate group of terms.
    #[display("BG-{0}")]
    BG(u16),
}
