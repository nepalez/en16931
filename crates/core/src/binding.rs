
/// A serialization binding: one of the two EN-16931 XML syntaxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// OASIS Universal Business Language.
    Ubl,
    /// UN/CEFACT Cross Industry Invoice.
    Cii,
}
