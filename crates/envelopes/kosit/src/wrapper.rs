use crate::Error;
use crate::prelude::*;

// The namespace of the XOEV report the validation tool answers with.
const VARL: &str = "http://www.xoev.de/de/validator/varl/1";

/// Extract the findings of the KoSIT validation tool from its report.
///
/// The tool gathers the outcome of every validation step it ran into one XOEV report.
/// A finding of any step is a `rep:message` element with its level,
/// the rule that fired, the message text, and the address of the reported node.
///
/// The message text keeps the wording alone. The indentation around it goes,
/// and so does the rule identifier the rule sets repeat at its head,
/// which the finding already names in a field of its own.
///
/// A finding without an address fails the whole report, since it binds to no node.
#[derive(Debug)]
pub struct Kosit;

impl Wrapper for Kosit {
    type Error = Error;

    fn unwrap(&self, report: &str) -> Result<Vec<Entry>, Self::Error> {
        let document =
            Document::parse(report).map_err(|error| Error::Malformed(error.to_string()))?;
        document
            .descendants()
            .filter(|node| node.has_tag_name((VARL, "message")))
            .map(entry)
            .collect()
    }
}

// Reads one finding of the report.
fn entry(message: Node<'_, '_>) -> Result<Entry, Error> {
    let named = message.attribute("id").unwrap_or("an unnamed finding");
    let code = message.attribute("code");
    let text = message
        .descendants()
        .filter(Node::is_text)
        .filter_map(|node| node.text())
        .collect::<String>();
    Ok(Entry {
        severity: severity(message.attribute("level").unwrap_or_default())?,
        code: code.map(str::to_owned),
        text: wording(&text, code),
        original_location: message
            .attribute("xpathLocation")
            .map(str::to_owned)
            .ok_or_else(|| Error::MissingLocation(named.to_owned()))?,
        normalized_location: None,
    })
}

// Strips the head a rule set writes before the wording of a message:
// the rule of the finding in brackets, and every mark that follows it.
// A head that names another rule belongs to the wording and stays.
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
        "error" => Ok(Severity::Error),
        "warning" => Ok(Severity::Warning),
        "information" => Ok(Severity::Information),
        other => Err(Error::UnknownLevel(other.to_owned())),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // A report of the shape the tool answers with, carrying a single finding.
    fn report(message: &str) -> String {
        format!(
            r#"<rep:report xmlns:rep="{VARL}" valid="false">
                 <rep:scenarioMatched>
                   <rep:validationStepResult id="val-sch.1" valid="false">
                     {message}
                   </rep:validationStepResult>
                 </rep:scenarioMatched>
               </rep:report>"#
        )
    }

    #[test]
    fn reads_the_findings_of_a_report() {
        let expected = Ok(vec![
            Entry {
                severity: Severity::Error,
                code: Some("BR-CO-16".to_owned()),
                text: "Amount due for payment (BT-115) = Invoice total amount with VAT (BT-112) -Paid amount (BT-113) +Rounding amount (BT-114).".to_owned(),
                original_location: "/ubl:Invoice/cac:LegalMonetaryTotal[1]".to_owned(),
                normalized_location: None,
            },
            Entry {
                severity: Severity::Information,
                code: Some("BR-DE-TMP-32".to_owned()),
                text: "Eine Rechnung sollte zur Angabe des Liefer-/Leistungsdatums entweder BT-72 \"Actual delivery date\", BG-14 \"Invoicing period\" oder in jeder Rechnungsposition BG-26 \"Invoice line period\" enthalten.".to_owned(),
                original_location: "/ubl:Invoice".to_owned(),
                normalized_location: None,
            },
        ]);

        let actual = Kosit.unwrap(include_str!("fixtures/report.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_warning() {
        let expected = Ok(vec![Entry {
            severity: Severity::Warning,
            code: Some("BR-DE-26".to_owned()),
            text: "a warning".to_owned(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-sch.1.1" level="warning" xpathLocation="/ubl:Invoice" code="BR-DE-26">a warning</rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_text_of_several_pieces() {
        let expected = Ok(vec![Entry {
            severity: Severity::Error,
            code: None,
            text: "a < b and c > d".to_owned(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-sch.1.1" level="error" xpathLocation="/ubl:Invoice">a &lt; b<![CDATA[ and c > d]]></rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn keeps_a_head_that_names_another_rule() {
        let expected = Ok(vec![Entry {
            severity: Severity::Error,
            code: Some("BR-CO-16".to_owned()),
            text: "[BR-CO-15] see the rule above".to_owned(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-sch.1.1" level="error" xpathLocation="/ubl:Invoice" code="BR-CO-16">[BR-CO-15] see the rule above</rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn flattens_a_text_of_nested_markup() {
        let expected = Ok(vec![Entry {
            severity: Severity::Error,
            code: None,
            text: "the rule BR-CO-16 failed".to_owned(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-sch.1.1" level="error" xpathLocation="/ubl:Invoice">the rule <em>BR-CO-16</em> failed</rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_finding_of_an_empty_element() {
        let expected = Ok(vec![Entry {
            severity: Severity::Error,
            code: None,
            text: String::new(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-sch.1.1" level="error" xpathLocation="/ubl:Invoice"/>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn skips_an_element_of_another_namespace() {
        let expected = Ok(Vec::new());

        let actual = Kosit.unwrap(&report(
            r#"<message xmlns="urn:example:other" level="error" xpathLocation="/ubl:Invoice">a stray element</message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_no_findings_of_a_report_without_them() {
        let expected = Ok(Vec::new());

        let actual = Kosit.unwrap(&report(""));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_without_a_location() {
        let expected = Err(Error::MissingLocation("val-xsd.1".to_owned()));

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-xsd.1" level="error">broken schema</rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_that_names_no_finding_id() {
        let expected = Err(Error::MissingLocation("an unnamed finding".to_owned()));

        let actual = Kosit.unwrap(&report(
            r#"<rep:message level="error">a finding of no name</rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_without_a_level() {
        let expected = Err(Error::UnknownLevel(String::new()));

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-sch.1.1" xpathLocation="/ubl:Invoice">a breach</rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_of_an_unknown_level() {
        let expected = Err(Error::UnknownLevel("hint".to_owned()));

        let actual = Kosit.unwrap(&report(
            r#"<rep:message id="val-sch.1.1" level="hint" xpathLocation="/ubl:Invoice">a hint</rep:message>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_malformed_report() {
        let expected = Err(Error::Malformed(
            "the document does not have a root node".to_owned(),
        ));

        let actual = Kosit.unwrap("<rep:report");

        assert_eq!(actual, expected);
    }
}
