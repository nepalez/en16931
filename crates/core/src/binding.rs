use crate::format::{cii, ubl};
use crate::prelude::*;
use crate::{Dictionary, DocumentBuilder, Error, Namespace};

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
            let (namespace, event) = reader
                .read_resolved_event()
                .map_err(|error| Error::MalformedXml(error.to_string()))?;
            match event {
                Event::Start(_) | Event::Empty(_) => return Self::from_root(namespace),
                Event::Eof => return Err(Error::MalformedXml("no root element".to_owned())),
                _ => {}
            }
        }
    }

    /// Serializes a `DocumentBuilder` to this binding's XML.
    /// Returns the XML along with the dictionary binding its nodes to the document's.
    pub fn serialize(self, builder: &DocumentBuilder) -> (String, Dictionary) {
        match self {
            Self::Ubl => ubl::serialize(builder),
            Self::Cii => cii::serialize(builder),
        }
    }

    /// Parses a `DocumentBuilder` from this binding's XML,
    /// rebuilding the dictionary on the inverse path.
    /// The profile is recovered from `BT-24`.
    ///
    /// A malformed document yields `Error::MalformedXml`.
    pub fn deserialize(self, xml: &str) -> Result<(DocumentBuilder, Dictionary), Error> {
        match self {
            Self::Ubl => ubl::deserialize(xml),
            Self::Cii => cii::deserialize(xml),
        }
    }

    fn from_root(namespace: ResolveResult<'_>) -> Result<Self, Error> {
        let ResolveResult::Bound(uri) = namespace else {
            return Err(Error::MalformedXml(
                "the root element has no namespace".to_owned(),
            ));
        };
        let uri = uri.into_inner();
        if uri == Namespace::Invoice.uri().as_bytes() {
            Ok(Self::Ubl)
        } else if uri == Namespace::CrossIndustryInvoice.uri().as_bytes() {
            Ok(Self::Cii)
        } else {
            Err(Error::MalformedXml(format!(
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
    fn detects_the_cii_binding_from_its_root() {
        let xml = r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"/>"#;

        assert_eq!(Binding::detect(xml).expect("a CII document"), Binding::Cii);
    }

    #[test]
    fn rejects_an_unrecognized_root_namespace() {
        let xml = r#"<foo xmlns="urn:example:unknown"/>"#;

        assert!(matches!(Binding::detect(xml), Err(Error::MalformedXml(_))));
    }
}
