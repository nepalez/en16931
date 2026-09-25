use crate::prelude::*;
use crate::{Cii, Error, Format, InvoiceKind, Namespace, Ubl};

/// A serialization binding: one of the two EN-16931 XML syntaxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// OASIS Universal Business Language.
    Ubl,
    /// UN/CEFACT Cross Industry Invoice.
    Cii,
}

impl Binding {
    /// Detects the binding from the namespace of the document's root element.
    /// A malformed document or an unrecognized root namespace yield `Error::MalformedXml`.
    pub fn detect(xml: &str) -> Result<Self, Error> {
        let mut reader = NsReader::from_str(xml);
        loop {
            let (namespace, event) = reader.read_resolved_event()?;
            match event {
                Event::Start(_) | Event::Empty(_) => return Self::from_root(namespace),
                Event::Eof => return Err(Error::malformed_xml("no root element")),
                _ => {}
            }
        }
    }

    fn from_root(namespace: ResolveResult<'_>) -> Result<Self, Error> {
        let ResolveResult::Bound(uri) = namespace else {
            return Err(Error::malformed_xml("the root element has no namespace"));
        };
        let uri = uri.into_inner();
        if InvoiceKind::VARIANTS
            .iter()
            .any(|kind| uri == Ubl::root_namespace(*kind).uri().as_bytes())
        {
            Ok(Self::Ubl)
        } else if uri == Cii::root_namespace(InvoiceKind::default()).uri().as_bytes() {
            Ok(Self::Cii)
        } else {
            Err(Error::malformed_xml(format!(
                "unrecognized root namespace: {}",
                String::from_utf8_lossy(uri)
            )))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn detects_the_ubl_binding_from_its_root() {
        let xml =
            r#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"></Invoice>"#;

        assert_eq!(Binding::detect(xml).expect("a UBL document"), Binding::Ubl);
    }

    #[test]
    fn detects_the_ubl_binding_from_a_credit_note_root() {
        let xml = r#"<CreditNote xmlns="urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2"></CreditNote>"#;

        assert_eq!(Binding::detect(xml).expect("a UBL document"), Binding::Ubl);
    }

    #[test]
    fn detects_the_cii_binding_from_its_root() {
        let xml = r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"/>"#;

        assert_eq!(Binding::detect(xml).expect("a CII document"), Binding::Cii);
    }

    #[test]
    fn rejects_an_unrecognized_root_namespace() {
        let xml = r#"<foo xmlns="urn:example:unknown"/>"#;

        assert!(matches!(
            Binding::detect(xml),
            Err(Error::MalformedXml { .. })
        ));
    }
}
