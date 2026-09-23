use crate::prelude::*;

/// A business term of EN-16931, named by its `BT`/`BG` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Term {
    /// A single leaf field (`BT-n`).
    #[display("BT-{0}")]
    BT(u16),
    /// A repeatable or aggregate group of terms (`BG-n`).
    #[display("BG-{0}")]
    BG(u16),
}
