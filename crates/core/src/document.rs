use crate::{
    Abbreviations, Binding, Context, Dictionary, DocumentBuilder, Error, InvalidDocument, Invoice,
    Location, Namespace, Path, Problem, Profile, RawNamespace, RawReport, Report, Step,
    ValidDocument,
};

/// The public reporting artifact of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    // The staging form the document was serialized from or parsed into.
    builder: DocumentBuilder,
    // The serialized XML of this document.
    xml: String,
    // One entry per node the binding handled: record-form path to `Context`.
    dictionary: Dictionary,
    // The abbreviations a report location of this document may name.
    abbreviations: Abbreviations,
}

impl Document {
    /// Parses an XML document into a `Document`.
    ///
    /// It detects the binding from the root element and reconstructs the
    /// builder and the dictionary on the inverse path. A malformed document, or
    /// one with an unrecognized binding, yields `Error::MalformedXml`.
    pub fn parse(xml: &str) -> Result<Self, Error> {
        let binding = Binding::detect(xml)?;
        let (builder, dictionary, abbreviations) = binding.deserialize(xml)?;
        Ok(Self {
            builder,
            xml: xml.to_owned(),
            dictionary,
            abbreviations,
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

    /// The abbreviations a normalizer resolves a report location against.
    pub fn abbreviations(&self) -> &Abbreviations {
        &self.abbreviations
    }

    /// Binds the answer of a validator to the nodes of this document.
    pub fn check(self, report: RawReport) -> Result<Result<ValidDocument, InvalidDocument>, Error> {
        let mut problems = Vec::with_capacity(report.findings.len());

        for finding in report.findings {
            let bound = finding
                .normalized_location
                .as_ref()
                .and_then(|location| self.resolve(location));
            let Some(context) = bound else {
                return Err(Error::UnboundLocation(finding.original_location));
            };
            problems.push(Problem {
                severity: finding.severity,
                code: finding.code,
                text: finding.text,
                context,
            });
        }

        let report = Report { problems };
        match report.has_errors() {
            true => Ok(Err(InvalidDocument {
                document: self,
                report,
            })),
            false => Ok(Ok(ValidDocument {
                document: self,
                report,
            })),
        }
    }

    // Binds a normalized address to the node of this document it points at.
    fn resolve(&self, location: &Location) -> Option<Context> {
        let mut path = Path { steps: Vec::new() };
        let mut bound = None;
        for step in &location.steps {
            let Some(namespace) = (match step.namespace.as_ref() {
                Some(RawNamespace::Uri(uri)) => Namespace::from_uri(uri),
                Some(RawNamespace::Abbreviation(name)) => self.abbreviations.resolve(name),
                None => None,
            }) else {
                break;
            };

            path.steps.push(Step {
                namespace,
                name: step.name.clone(),
                index: step.index,
            });
            let Some(context) = self.dictionary.get(&path) else {
                break;
            };

            bound = Some(context.clone());
        }
        bound
    }
}

impl TryFrom<DocumentBuilder> for Document {
    type Error = Error;

    /// Serializes a `DocumentBuilder` into a `Document`, running its `Binding`.
    ///
    /// The pass renders the XML and fills the dictionary in lockstep.
    fn try_from(builder: DocumentBuilder) -> Result<Self, Self::Error> {
        let (xml, dictionary, abbreviations) = builder.binding.serialize(&builder);
        Ok(Self {
            builder,
            xml,
            dictionary,
            abbreviations,
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
    use crate::prelude::*;
    use crate::{Context, Entry, Location, LocationStep, RawNamespace, Segment, Severity};

    // A serialized document of the rich UBL fixture.
    fn document() -> Document {
        Document::try_from(builder(Binding::Ubl)).expect("a serialized document")
    }

    // The same document with `cbc` renamed to `foo`, an abbreviation of its own.
    fn renamed() -> Document {
        Document::parse(include_str!("format/ubl/fixtures/3.xml")).expect("a valid UBL document")
    }

    // An address of the steps the dialect abbreviated.
    fn location(steps: &[(&str, &str, usize)]) -> Location {
        Location {
            steps: steps
                .iter()
                .map(|(abbreviation, name, index)| abbreviated(abbreviation, name, *index))
                .collect(),
        }
    }

    // A step whose namespace the dialect abbreviated.
    fn abbreviated(abbreviation: &str, name: &str, index: usize) -> LocationStep {
        LocationStep {
            namespace: Some(RawNamespace::Abbreviation(abbreviation.to_owned())),
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    // A step whose namespace the dialect wrote in full.
    fn written(uri: &str, name: &str, index: usize) -> LocationStep {
        LocationStep {
            namespace: Some(RawNamespace::Uri(uri.to_owned())),
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    // A trailing attribute step, which belongs to no namespace.
    fn attribute(name: &str) -> LocationStep {
        LocationStep {
            namespace: None,
            name: name.to_owned(),
            index: NonZeroUsize::new(1).expect("a positive index"),
        }
    }

    // The address of the registration name of the seller, a leaf field.
    fn seller_name() -> Vec<LocationStep> {
        vec![
            abbreviated("ubl", "Invoice", 1),
            abbreviated("cac", "AccountingSupplierParty", 1),
            abbreviated("cac", "Party", 1),
            abbreviated("cac", "PartyLegalEntity", 1),
            abbreviated("cbc", "RegistrationName", 1),
        ]
    }

    // A context of the given model segments.
    fn context(segments: Vec<Segment>) -> Context {
        Context { segments }
    }

    // A single non-indexed field segment.
    fn field(name: &'static str) -> Segment {
        Segment {
            field: name,
            index: None,
        }
    }

    // A repeatable-group instance segment.
    fn instance(name: &'static str, index: usize) -> Segment {
        Segment {
            field: name,
            index: NonZeroUsize::new(index),
        }
    }

    // A finding of the given weight, addressing the second invoice line.
    fn finding(severity: Severity) -> Entry {
        Entry {
            severity,
            code: Some("BR-21".to_owned()),
            text: "each line needs an identifier".to_owned(),
            original_location: "/ubl:Invoice/cac:InvoiceLine[2]".to_owned(),
            normalized_location: Some(location(&[
                ("ubl", "Invoice", 1),
                ("cac", "InvoiceLine", 2),
            ])),
        }
    }

    // The context of the second invoice line, which every finding above addresses.
    fn second_line() -> Context {
        context(vec![instance("lines", 2)])
    }

    #[test]
    fn binds_an_address_of_a_leaf_to_its_field() {
        let location = Location {
            steps: seller_name(),
        };

        let bound = document().resolve(&location);

        assert_eq!(bound, Some(context(vec![field("seller"), field("name")])));
    }

    #[test]
    fn binds_an_address_of_a_group_to_its_instance() {
        let location = location(&[("ubl", "Invoice", 1), ("cac", "InvoiceLine", 2)]);

        let bound = document().resolve(&location);

        assert_eq!(bound, Some(second_line()));
    }

    #[test]
    fn binds_an_address_of_a_term_less_node_to_the_root() {
        let location = location(&[
            ("ubl", "Invoice", 1),
            ("cac", "LegalMonetaryTotal", 1),
            ("cbc", "PayableAmount", 1),
        ]);

        let bound = document().resolve(&location);

        assert_eq!(bound, Some(context(Vec::new())));
    }

    #[test]
    fn binds_an_address_of_an_attribute_to_the_element() {
        let mut steps = seller_name();
        steps.push(attribute("languageID"));

        let bound = document().resolve(&Location { steps });

        assert_eq!(bound, Some(context(vec![field("seller"), field("name")])));
    }

    #[test]
    fn binds_an_address_deeper_than_the_document_to_the_nearest_node() {
        let mut steps = seller_name();
        steps.push(abbreviated("cbc", "Absent", 1));

        let bound = document().resolve(&Location { steps });

        assert_eq!(bound, Some(context(vec![field("seller"), field("name")])));
    }

    #[test]
    fn resolves_a_namespace_the_dialect_wrote_in_full() {
        let location = Location {
            steps: vec![
                written(Namespace::Invoice.uri(), "Invoice", 1),
                written(Namespace::CommonBasicComponents.uri(), "ID", 1),
            ],
        };

        let bound = document().resolve(&location);

        assert_eq!(bound, Some(context(vec![field("number")])));
    }

    #[test]
    fn resolves_an_abbreviation_of_the_document() {
        let location = location(&[("ubl", "Invoice", 1), ("foo", "ID", 1)]);

        let bound = renamed().resolve(&location);

        assert_eq!(bound, Some(context(vec![field("number")])));
    }

    #[test]
    fn binds_no_address_of_another_binding() {
        let location = location(&[("rsm", "CrossIndustryInvoice", 1)]);

        let bound = document().resolve(&location);

        assert_eq!(bound, None);
    }

    #[test]
    fn binds_no_address_of_an_unknown_abbreviation() {
        let location = location(&[("xsi", "Invoice", 1)]);

        let bound = document().resolve(&location);

        assert_eq!(bound, None);
    }

    #[test]
    fn rejects_a_document_of_an_error() {
        let source = document();

        let outcome = source.check(RawReport {
            findings: vec![finding(Severity::Error)],
        });

        let Ok(Err(rejected)) = outcome else {
            panic!("an error should reject the document");
        };
        assert_eq!(
            rejected.report().problems,
            vec![Problem {
                severity: Severity::Error,
                code: Some("BR-21".to_owned()),
                text: "each line needs an identifier".to_owned(),
                context: second_line(),
            }]
        );
    }

    #[test]
    fn accepts_a_document_of_no_error() {
        let source = document();

        let outcome = source.check(RawReport {
            findings: vec![finding(Severity::Warning), finding(Severity::Information)],
        });

        let Ok(Ok(accepted)) = outcome else {
            panic!("a report of no error should accept the document");
        };
        assert_eq!(accepted.report().problems.len(), 2);
        assert_eq!(accepted.report().problems[0].context, second_line());
    }

    #[test]
    fn yields_the_checked_document_and_its_invoice_back() {
        let source = document();

        let outcome = source.clone().check(RawReport {
            findings: vec![finding(Severity::Warning)],
        });

        let Ok(Ok(accepted)) = outcome else {
            panic!("a report of no error should accept the document");
        };
        assert_eq!(Invoice::from(accepted.clone()), source.builder.invoice);
        assert_eq!(Document::from(accepted), source);
    }

    #[test]
    fn fails_the_pass_of_an_unbound_location() {
        let source = document();
        let stray = Entry {
            original_location: "/rsm:CrossIndustryInvoice".to_owned(),
            normalized_location: Some(location(&[("rsm", "CrossIndustryInvoice", 1)])),
            ..finding(Severity::Error)
        };

        let outcome = source.check(RawReport {
            findings: vec![finding(Severity::Error), stray],
        });

        let Err(error) = outcome else {
            panic!("an unbound location should fail the pass");
        };
        assert!(
            matches!(error, Error::UnboundLocation(address) if address == "/rsm:CrossIndustryInvoice")
        );
    }

    #[test]
    fn fails_the_pass_of_a_location_no_normalizer_read() {
        let source = document();
        let unread = Entry {
            normalized_location: None,
            ..finding(Severity::Error)
        };

        let outcome = source.check(RawReport {
            findings: vec![unread],
        });

        assert!(matches!(outcome, Err(Error::UnboundLocation(_))));
    }

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
        let (xml, _, _) = source.binding.serialize(&source);
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
