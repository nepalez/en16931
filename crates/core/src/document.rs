use crate::{Binding, Dictionary, DocumentBuilder, Error, Invoice, Profile};

/// The public reporting artifact of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    // The staging form the document was serialized from or parsed into.
    builder: DocumentBuilder,
    // The serialized XML of this document.
    xml: String,
    // One entry per node the binding handled: record-form path to `Context`.
    dictionary: Dictionary,
}

impl Document {
    /// Parses an XML document into a `Document`.
    ///
    /// It detects the binding from the root element and reconstructs the
    /// builder and the dictionary on the inverse path. A malformed document, or
    /// one with an unrecognized binding, yields `Error::MalformedXml`.
    pub fn parse(xml: &str) -> Result<Self, Error> {
        let binding = Binding::detect(xml)?;
        let (builder, dictionary) = binding.deserialize(xml)?;
        Ok(Self {
            builder,
            xml: xml.to_owned(),
            dictionary,
        })
    }

    /// The serialized XML.
    pub fn xml(&self) -> &str {
        &self.xml
    }

    /// The profile the document was serialized under.
    pub fn profile(&self) -> Profile {
        self.builder.profile
    }

    /// The binding the document is serialized in.
    pub fn binding(&self) -> Binding {
        self.builder.binding
    }
}

impl TryFrom<DocumentBuilder> for Document {
    type Error = Error;

    /// Serializes a `DocumentBuilder` into a `Document`, running its `Binding`.
    ///
    /// The pass renders the XML and fills the dictionary in lockstep.
    fn try_from(builder: DocumentBuilder) -> Result<Self, Self::Error> {
        let (xml, dictionary) = builder.binding.serialize(&builder);
        Ok(Self {
            builder,
            xml,
            dictionary,
        })
    }
}

impl From<Document> for Invoice {
    /// Consumes the document and yields its invoice, dropping the XML and the
    /// dictionary once the artifact becomes a plain business object.
    fn from(document: Document) -> Self {
        document.builder.invoice
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::test_helpers::builder;

    #[test]
    fn round_trips_a_builder_through_a_document() {
        for binding in [Binding::Ubl, Binding::Cii] {
            let document = Document::try_from(builder(binding)).expect("a serialized document");

            let parsed = Document::parse(&document.xml).expect("a parsed document");

            assert_eq!(parsed, document);
        }
    }

    #[test]
    fn rebuilds_the_same_dictionary_on_parse() {
        for binding in [Binding::Ubl, Binding::Cii] {
            let document = Document::try_from(builder(binding)).expect("a serialized document");

            let parsed = Document::parse(&document.xml).expect("a parsed document");

            assert_eq!(parsed.dictionary, document.dictionary);
        }
    }

    #[test]
    fn yields_the_request_parts() {
        let source = builder(Binding::Ubl);
        let (xml, _) = source.binding.serialize(&source);
        let document = Document::try_from(source.clone()).expect("a serialized document");

        assert_eq!(document.xml(), xml);
        assert_eq!(document.profile(), source.profile);
        assert_eq!(document.binding(), source.binding);
    }

    #[test]
    fn yields_the_request_parts_of_each_profile_of_one_invoice() {
        let source = builder(Binding::Ubl);
        let parts = |profile| {
            let document = Document::try_from(DocumentBuilder {
                profile,
                ..source.clone()
            })
            .expect("a serialized document");
            (
                document.profile(),
                document.binding(),
                document.xml().to_owned(),
            )
        };

        let (dutch_profile, dutch_binding, dutch_xml) = parts(Profile::Nlcius10);
        let (peppol_profile, peppol_binding, peppol_xml) = parts(Profile::PeppolBisBilling30);

        assert_eq!(dutch_profile, Profile::Nlcius10);
        assert_eq!(peppol_profile, Profile::PeppolBisBilling30);
        assert_eq!(dutch_binding, peppol_binding);
        assert!(dutch_xml.contains(&Profile::Nlcius10.to_string()));
        assert!(peppol_xml.contains(&Profile::PeppolBisBilling30.to_string()));
    }

    #[test]
    fn yields_its_invoice_by_value() {
        let source = builder(Binding::Ubl);
        let document = Document::try_from(source.clone()).expect("a serialized document");

        assert_eq!(Invoice::from(document), source.invoice);
    }
}
