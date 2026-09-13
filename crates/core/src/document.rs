use crate::prelude::*;
use crate::{
    Abbreviations, Context, Deserializable, Dictionary, DocumentBuilder, Error, Format,
    InvalidDocument, Invoice, Location, Namespace, Path, Problem, Profile, RawNamespace, RawReport,
    Report, Serializable, Step, Target, ValidDocument,
};

/// The public reporting artifact of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document<I, F: Format> {
    // The staging form the document was serialized from or parsed into.
    pub(crate) builder: DocumentBuilder<I>,
    // The serialized XML of this document.
    pub(crate) xml: String,
    // One entry per node the binding handled, from its record-form path to a `Context`.
    pub(crate) dictionary: Dictionary<F::Namespace>,
    // The abbreviations a report location of this document may name.
    pub(crate) abbreviations: Abbreviations<F::Namespace>,
}

impl<I: Deserializable<F> + Default, F: Format> Document<I, F> {
    /// Parses an XML document of the binding `F` into a `Document`.
    ///
    /// A malformed document yields `Error::MalformedXml`.
    pub fn parse(xml: &str) -> Result<Self, Error> {
        let mut document = Self::empty(DocumentBuilder {
            invoice: I::default(),
            profile: Profile::En16931,
            business_process: None,
        });
        document.xml = xml.to_owned();
        I::deserialize(&mut document)?;
        Ok(document)
    }
}

impl<I, F: Format> Document<I, F> {
    /// The serialized XML.
    pub fn xml(&self) -> &str {
        &self.xml
    }

    /// Binds the answer of a validator to the nodes of this document.
    ///
    /// The outer result covers a location no node answers.
    #[allow(clippy::type_complexity)]
    pub fn check(
        self,
        report: RawReport,
    ) -> Result<Result<ValidDocument<I, F>, InvalidDocument<I, F>>, Error> {
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

    // A document a serialization or a parsing pass has yet to fill.
    pub(crate) fn empty(builder: DocumentBuilder<I>) -> Self {
        Self {
            builder,
            xml: String::new(),
            dictionary: HashMap::new(),
            abbreviations: F::Namespace::default_abbreviations(),
        }
    }

    // Binds a normalized address to the node of this document it points at.
    fn resolve(&self, location: &Location) -> Option<Context> {
        let mut path = Path { steps: Vec::new() };
        let mut bound = None;
        for step in &location.steps {
            let Some(namespace) = (match step.namespace.as_ref() {
                Some(RawNamespace::Uri(uri)) => F::Namespace::from_uri(uri),
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

impl<F: Format> Document<Invoice, F> {
    /// What a validator needs to pick the rule set for this document.
    pub fn target(&self) -> Target {
        Target {
            profile: self.builder.profile,
            binding: F::BINDING,
            kind: self.builder.invoice.type_code.kind(),
        }
    }
}

impl<I: Serializable<F>, F: Format> TryFrom<DocumentBuilder<I>> for Document<I, F> {
    type Error = Error;

    /// Serializes a `DocumentBuilder` into a `Document` of the binding `F`.
    ///
    /// The pass renders the XML and fills the dictionary in lockstep.
    fn try_from(builder: DocumentBuilder<I>) -> Result<Self, Self::Error> {
        let mut document = Self::empty(builder);
        I::serialize(&mut document);
        Ok(document)
    }
}

impl<F: Format> From<Document<Invoice, F>> for Invoice {
    /// Consumes the document and yields its invoice, dropping the XML and the
    /// dictionary once the artifact becomes a plain business object.
    fn from(document: Document<Invoice, F>) -> Self {
        document.builder.invoice
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::test_helpers::builder;
    use crate::{
        Binding, Cii, Context, Entry, Location, LocationStep, RawNamespace, Segment, Severity, Ubl,
        ubl,
    };

    // A serialized document of the rich UBL fixture.
    fn document() -> Document<Invoice, Ubl> {
        Document::try_from(builder()).expect("a serialized document")
    }

    // The same document with `cbc` renamed to `foo`, an abbreviation of its own.
    fn renamed() -> Document<Invoice, Ubl> {
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
                written(ubl::Namespace::Inv.uri(), "Invoice", 1),
                written(ubl::Namespace::Cbc.uri(), "ID", 1),
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
    fn round_trips_a_builder_through_the_ubl_document() {
        let document = document();

        let parsed = Document::<Invoice, Ubl>::parse(&document.xml).expect("a parsed document");

        assert_eq!(parsed, document);
    }

    #[test]
    fn round_trips_a_builder_through_the_cii_document() {
        let document =
            Document::<Invoice, Cii>::try_from(builder()).expect("a serialized document");

        let parsed = Document::<Invoice, Cii>::parse(&document.xml).expect("a parsed document");

        assert_eq!(parsed, document);
    }

    #[test]
    fn yields_the_request_parts() {
        let source = builder();
        let document = document();

        let target = Target {
            profile: source.profile,
            binding: Binding::Ubl,
            kind: source.invoice.type_code.kind(),
        };

        assert!(document.xml().starts_with("<Invoice"));
        assert_eq!(document.target(), target);
    }

    #[test]
    fn yields_the_request_parts_of_each_profile_of_one_invoice() {
        let source = builder();

        for profile in [Profile::Nlcius10, Profile::PeppolBisBilling30] {
            let document = Document::<Invoice, Ubl>::try_from(DocumentBuilder {
                profile,
                ..source.clone()
            })
            .expect("a serialized document");
            let target = Target {
                profile,
                binding: Binding::Ubl,
                kind: source.invoice.type_code.kind(),
            };

            assert_eq!(document.target(), target);
            assert!(document.xml().contains(&profile.to_string()));
        }
    }

    #[test]
    fn yields_its_invoice_by_value() {
        let source = builder();

        assert_eq!(Invoice::from(document()), source.invoice);
    }
}
