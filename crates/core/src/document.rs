use crate::{
    Abbreviations, Context, Dictionary, DocumentBuilder, Error, InvalidDocument, Invoice,
    InvoiceKind, Location, Parser, Path, Problem, RawNamespace, RawReport, Report, Serializer,
    Step, Target, ValidDocument,
};
use crate::{Deserializable, Format, Namespace, Serializable};

pub mod invalid;
pub mod valid;

/// The public reporting artifact of the library.
///
/// The document is an envelope: it holds the invoice by value and hands it back whole,
/// while the XML, the dictionary, and the abbreviations serve the binding of a validator's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document<I, F: Format> {
    // The staging form the document was serialized from or parsed into.
    pub(crate) builder: DocumentBuilder<I>,
    // The kind of the invoice, read from it when the document was made.
    pub(crate) kind: InvoiceKind,
    // The serialized XML of this document.
    pub(crate) xml: String,
    // One entry per node the binding handled, from its record-form path to a `Context`.
    pub(crate) dictionary: Dictionary<F::Namespace>,
    // The abbreviations a report location of this document may name.
    pub(crate) abbreviations: Abbreviations<F::Namespace>,
}

impl<I: Invoice + Deserializable<F, F::Namespace>, F: Format> Document<I, F> {
    /// Parses an XML document of the binding `F` into a `Document`.
    ///
    /// A malformed document yields `Error::MalformedXml`.
    pub fn parse(xml: &str) -> Result<Self, Error> {
        let (tokens, abbreviations) = F::tokenize(xml)?;
        let mut parser: Parser<F, F::Namespace> = Parser::new(tokens);
        let mut builder = I::deserialize(&mut parser)?;
        let kind = *builder.invoice.kind();
        Ok(Self {
            builder,
            kind,
            xml: xml.to_owned(),
            dictionary: parser.finish(),
            abbreviations,
        })
    }
}

impl<I, F: Format> Document<I, F> {
    /// The serialized XML.
    pub fn xml(&self) -> &str {
        &self.xml
    }

    /// What a validator needs to pick the rule set for this document.
    pub fn target(&self) -> Target {
        Target {
            profile: self.builder.profile,
            binding: F::BINDING,
            kind: self.kind,
        }
    }

    /// Consumes the document and yields its invoice, dropping the XML and the
    /// dictionary once the artifact becomes a plain business object.
    pub fn into_invoice(self) -> I {
        self.builder.invoice
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

impl<I: Invoice + Serializable<F, F::Namespace>, F: Format> TryFrom<DocumentBuilder<I>>
    for Document<I, F>
{
    type Error = Error;

    /// Serializes a `DocumentBuilder` into a `Document` of the binding `F`.
    ///
    /// The pass renders the XML and fills the dictionary in lockstep.
    fn try_from(mut builder: DocumentBuilder<I>) -> Result<Self, Self::Error> {
        let kind = *builder.invoice.kind();
        let mut serializer: Serializer<F, F::Namespace> =
            Serializer::new(builder.profile.forbidden_terms());
        I::serialize(&mut serializer, &mut builder);
        let (xml, dictionary, abbreviations) = serializer.finish();
        Ok(Self {
            builder,
            kind,
            xml,
            dictionary,
            abbreviations,
        })
    }
}
