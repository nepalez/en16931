use crate::prelude::*;
use crate::{Error, vendor_id};

// The root element of the answer the service builds.
const ROOT: &str = "validationResults";

/// Extract the findings of the phive validation service from its answer.
///
/// The service checks a document against the schema and against every rule set it carries.
/// A finding of any of those checks becomes an entry with its weight,
/// the rule that fired, the message for a human reader, and the address of the node.
///
/// The message keeps the wording alone, without the rule identifier
/// the rule sets repeat before it, which the entry names in a field of its own.
///
/// A finding without an address fails the whole report, since it binds to no node.
/// The schema check reports such findings, so a document that breaks the schema
/// yields no findings at all.
///
/// An answer that is not a report of the service fails as well.
/// The service answers that way when it rejects the request itself,
/// and an empty list of findings would read as a clean document.
pub struct Phive;

impl Wrapper for Phive {
    type Error = Error;

    fn unwrap(&self, report: &str) -> Result<Vec<Entry>, Self::Error> {
        let document =
            Document::parse(report).map_err(|error| Error::Malformed(error.to_string()))?;
        let root = document.root_element();
        if !root.has_tag_name(ROOT) {
            return Err(Error::Malformed(format!(
                "the answer is rooted at {}",
                root.tag_name().name()
            )));
        }
        root.children()
            .filter(|node| node.has_tag_name("result"))
            .flat_map(|step| step.children().filter(|node| node.has_tag_name("item")))
            .map(entry)
            .collect()
    }

    fn vendor_id(&self, target: Target) -> Result<&'static str, CoreError> {
        vendor_id::try_from_target(target)
    }
}

// Reads one finding of the report.
fn entry(item: Node<'_, '_>) -> Result<Entry, Error> {
    let code = text_of(item, "errorID");
    let named = code
        .clone()
        .unwrap_or_else(|| "an unnamed finding".to_owned());
    let text = text_of(item, "errorText").unwrap_or_default();
    Ok(Entry {
        severity: severity(&text_of(item, "errorLevel").unwrap_or_default())?,
        text: wording(&text, code.as_deref()),
        code,
        original_location: text_of(item, "errorFieldName").ok_or(Error::MissingLocation(named))?,
        normalized_location: None,
    })
}

// The text of a child element of the finding, when the report writes one.
fn text_of(item: Node<'_, '_>, name: &str) -> Option<String> {
    item.children()
        .find(|node| node.has_tag_name(name))
        .and_then(|node| node.text())
        .map(str::to_owned)
}

// Keeps the wording of a message alone, without the rule identifier the rule sets repeat before it.
// A head that names another rule stays.
fn wording(text: &str, code: Option<&str>) -> String {
    let text = text.trim();
    match code.and_then(|code| text.strip_prefix(&format!("[{code}]"))) {
        Some(wording) => wording
            .trim_start_matches(|mark: char| !mark.is_alphanumeric())
            .to_owned(),
        None => text.to_owned(),
    }
}

// Maps the level of a finding onto the weight the core carries.
fn severity(level: &str) -> Result<Severity, Error> {
    match level {
        "ERROR" | "FATAL_ERROR" => Ok(Severity::Error),
        "WARN" => Ok(Severity::Warning),
        "INFO" | "SUCCESS" => Ok(Severity::Information),
        other => Err(Error::UnknownLevel(other.to_owned())),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // An answer of the shape the service builds, carrying a single finding.
    fn answer(item: &str) -> String {
        format!(
            r#"<validationResults>
                 <success>false</success>
                 <result>
                   <artifactType>schematron-xslt</artifactType>
                   {item}
                 </result>
               </validationResults>"#
        )
    }

    #[test]
    fn reads_the_findings_of_a_report() {
        let expected = Ok(vec![
            Entry {
                severity: Severity::Error,
                code: Some("BR-CO-10".to_owned()),
                text: "Sum of Invoice line net amount (BT-106) = Σ Invoice line net amount (BT-131).".to_owned(),
                original_location: "/:Invoice[1]/cac:LegalMonetaryTotal[1]".to_owned(),
                normalized_location: None,
            },
            Entry {
                severity: Severity::Error,
                code: Some("BR-CO-13".to_owned()),
                text: "Invoice total amount without VAT (BT-109) = Σ Invoice line net amount (BT-131) - Sum of allowances on document level (BT-107) + Sum of charges on document level (BT-108).".to_owned(),
                original_location: "/:Invoice[1]/cac:LegalMonetaryTotal[1]".to_owned(),
                normalized_location: None,
            },
        ]);

        let actual = Phive.unwrap(include_str!("fixtures/report.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_the_findings_of_every_weight() {
        let expected = Ok(vec![
            Entry {
                severity: Severity::Error,
                code: Some("BR-CO-16".to_owned()),
                text: "Amount due for payment (BT-115) = Invoice total amount with VAT (BT-112) -Paid amount (BT-113) +Rounding amount (BT-114).".to_owned(),
                original_location: "/:Invoice[1]/cac:LegalMonetaryTotal[1]".to_owned(),
                normalized_location: None,
            },
            Entry {
                severity: Severity::Information,
                code: Some("BR-DE-TMP-32".to_owned()),
                text: "Eine Rechnung sollte zur Angabe des Liefer-/Leistungsdatums entweder BT-72 \"Actual delivery date\", BG-14 \"Invoicing period\" oder in jeder Rechnungsposition BG-26 \"Invoice line period\" enthalten.".to_owned(),
                original_location: "/:Invoice[1]".to_owned(),
                normalized_location: None,
            },
        ]);

        let actual = Phive.unwrap(include_str!("fixtures/levels.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_no_findings_of_an_accepted_document() {
        let expected = Ok(Vec::new());

        let actual = Phive.unwrap(include_str!("fixtures/clean.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_warning() {
        let expected = Ok(vec![Entry {
            severity: Severity::Warning,
            code: Some("PEPPOL-EN16931-R110".to_owned()),
            text: "a warning".to_owned(),
            original_location: "/:Invoice[1]".to_owned(),
            normalized_location: None,
        }]);

        let actual = Phive.unwrap(&answer(
            r#"<item>
                 <errorLevel>WARN</errorLevel>
                 <errorID>PEPPOL-EN16931-R110</errorID>
                 <errorFieldName>/:Invoice[1]</errorFieldName>
                 <errorText>[PEPPOL-EN16931-R110] a warning</errorText>
               </item>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_finding_that_names_no_rule() {
        let expected = Ok(vec![Entry {
            severity: Severity::Error,
            code: None,
            text: "a breach of no rule".to_owned(),
            original_location: "/:Invoice[1]".to_owned(),
            normalized_location: None,
        }]);

        let actual = Phive.unwrap(&answer(
            r#"<item>
                 <errorLevel>ERROR</errorLevel>
                 <errorFieldName>/:Invoice[1]</errorFieldName>
                 <errorText>a breach of no rule</errorText>
               </item>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_report_of_a_broken_schema() {
        let actual = Phive.unwrap(include_str!("fixtures/schema.xml"));

        assert_eq!(
            actual,
            Err(Error::MissingLocation("an unnamed finding".to_owned()))
        );
    }

    #[test]
    fn rejects_a_finding_of_an_unknown_level() {
        let expected = Err(Error::UnknownLevel("HINT".to_owned()));

        let actual = Phive.unwrap(&answer(
            r#"<item>
                 <errorLevel>HINT</errorLevel>
                 <errorFieldName>/:Invoice[1]</errorFieldName>
                 <errorText>a hint</errorText>
               </item>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_answer_of_another_shape() {
        let actual = Phive.unwrap("<html><body>Forbidden</body></html>");

        assert!(matches!(actual, Err(Error::Malformed(_))));
    }

    #[test]
    fn rejects_an_empty_answer() {
        let actual = Phive.unwrap("");

        assert!(matches!(actual, Err(Error::Malformed(_))));
    }
}
