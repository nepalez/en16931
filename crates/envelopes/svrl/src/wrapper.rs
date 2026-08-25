use crate::Error;
use crate::prelude::*;

// The namespace of the SVRL report a Schematron processor writes.
const SVRL: &str = "http://purl.oclc.org/dsdl/svrl";

/// Extract the findings of a bare SVRL report.
///
/// The report is the direct output of a Schematron processor, `svrl:schematron-output`,
/// taken without any service envelope around it. A finding is a `svrl:failed-assert`
/// or a `svrl:successful-report` element with its flag, the rule that fired,
/// the message text, and the address of the reported node.
///
/// The message text keeps the wording alone, without the rule identifier
/// the rule sets repeat at its head, which the finding already names in a field of its own.
///
/// A finding without an address fails the whole report, since it binds to no node.
/// A document that is not an SVRL report fails as well,
/// since an empty list of findings would read as a clean document.
pub struct Svrl;

impl Wrapper for Svrl {
    type Error = Error;

    fn unwrap(&self, report: &str) -> Result<Vec<Entry>, Self::Error> {
        let document =
            Document::parse(report).map_err(|error| Error::Malformed(error.to_string()))?;
        let root = document.root_element();
        if !root.has_tag_name((SVRL, "schematron-output")) {
            return Err(Error::Malformed(format!(
                "the report is rooted at {}",
                root.tag_name().name()
            )));
        }
        root.children()
            .filter(|node| {
                node.has_tag_name((SVRL, "failed-assert"))
                    || node.has_tag_name((SVRL, "successful-report"))
            })
            .map(entry)
            .collect()
    }
}

fn entry(finding: Node<'_, '_>) -> Result<Entry, Error> {
    let code = finding.attribute("id").map(str::to_owned);
    let named = code
        .clone()
        .unwrap_or_else(|| "an unnamed finding".to_owned());
    let text = finding
        .children()
        .filter(|node| node.has_tag_name((SVRL, "text")))
        .flat_map(|node| node.descendants().filter(Node::is_text))
        .filter_map(|node| node.text())
        .collect::<String>();
    Ok(Entry {
        severity: severity(finding.attribute("flag").unwrap_or_default())?,
        text: wording(&text, code.as_deref()),
        code,
        original_location: finding
            .attribute("location")
            .map(str::to_owned)
            .ok_or(Error::MissingLocation(named))?,
        normalized_location: None,
    })
}

fn wording(text: &str, code: Option<&str>) -> String {
    let text = text.trim();
    match code.and_then(|code| text.strip_prefix(&format!("[{code}]"))) {
        Some(wording) => wording
            .trim_start_matches(|mark: char| !mark.is_alphanumeric())
            .to_owned(),
        None => text.to_owned(),
    }
}

fn severity(flag: &str) -> Result<Severity, Error> {
    match flag {
        "fatal" => Ok(Severity::Error),
        "warning" => Ok(Severity::Warning),
        "information" => Ok(Severity::Information),
        other => Err(Error::UnknownFlag(other.to_owned())),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // A report of the shape a Schematron processor writes, carrying a single finding.
    fn report(finding: &str) -> String {
        format!(
            r#"<svrl:schematron-output xmlns:svrl="{SVRL}" title="EN16931 model bound to UBL" schemaVersion="">
                 <svrl:active-pattern id="UBL-model" name="UBL-model"/>
                 <svrl:fired-rule context="/ubl:Invoice"/>
                 {finding}
               </svrl:schematron-output>"#
        )
    }

    #[test]
    fn reads_the_findings_of_a_report() {
        let expected = Ok(vec![Entry {
            severity: Severity::Error,
            code: Some("BR-CO-16".to_owned()),
            text: "Amount due for payment (BT-115) = Invoice total amount with VAT (BT-112) -Paid amount (BT-113) +Rounding amount (BT-114).".to_owned(),
            original_location: "/ubl:Invoice/cac:LegalMonetaryTotal[1]".to_owned(),
            normalized_location: None,
        }]);

        let actual = Svrl.unwrap(include_str!("fixtures/report.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_the_findings_of_every_weight() {
        let expected = Ok(vec![
            Entry {
                severity: Severity::Warning,
                code: Some("BR-DE-21".to_owned()),
                text: "Das Element \"Specification identifier\" (BT-24) soll syntaktisch der Kennung des Standards XRechnung entsprechen.".to_owned(),
                original_location: "/Invoice".to_owned(),
                normalized_location: None,
            },
            Entry {
                severity: Severity::Error,
                code: Some("BR-DE-2".to_owned()),
                text: "Die Gruppe \"SELLER CONTACT\" (BG-6) muss übermittelt werden.".to_owned(),
                original_location: "/Invoice/cac:AccountingSupplierParty[1]".to_owned(),
                normalized_location: None,
            },
        ]);

        let actual = Svrl.unwrap(include_str!("fixtures/levels.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_an_information_finding() {
        let expected = Ok(vec![Entry {
            severity: Severity::Information,
            code: Some("BR-DE-TMP-32".to_owned()),
            text: "Eine Rechnung sollte zur Angabe des Liefer-/Leistungsdatums entweder BT-72 \"Actual delivery date\", BG-14 \"Invoicing period\" oder in jeder Rechnungsposition BG-26 \"Invoice line period\" enthalten.".to_owned(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Svrl.unwrap(include_str!("fixtures/information.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_no_findings_of_an_accepted_document() {
        let expected = Ok(Vec::new());

        let actual = Svrl.unwrap(include_str!("fixtures/clean.xml"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_successful_report() {
        let expected = Ok(vec![Entry {
            severity: Severity::Warning,
            code: Some("BR-DE-27".to_owned()),
            text: "a reported condition".to_owned(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Svrl.unwrap(&report(
            r#"<svrl:successful-report test="cbc:ID" id="BR-DE-27" flag="warning" location="/ubl:Invoice">
                 <svrl:text>[BR-DE-27] a reported condition</svrl:text>
               </svrl:successful-report>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_finding_that_names_no_rule() {
        let expected = Ok(vec![Entry {
            severity: Severity::Error,
            code: None,
            text: "a breach of no rule".to_owned(),
            original_location: "/ubl:Invoice".to_owned(),
            normalized_location: None,
        }]);

        let actual = Svrl.unwrap(&report(
            r#"<svrl:failed-assert test="cbc:ID" flag="fatal" location="/ubl:Invoice">
                 <svrl:text>a breach of no rule</svrl:text>
               </svrl:failed-assert>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_without_a_location() {
        let expected = Err(Error::MissingLocation("BR-CO-16".to_owned()));

        let actual = Svrl.unwrap(&report(
            r#"<svrl:failed-assert test="cbc:ID" id="BR-CO-16" flag="fatal">
                 <svrl:text>a finding without an address</svrl:text>
               </svrl:failed-assert>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_without_a_location_or_a_rule() {
        let expected = Err(Error::MissingLocation("an unnamed finding".to_owned()));

        let actual = Svrl.unwrap(&report(
            r#"<svrl:failed-assert test="cbc:ID" flag="fatal">
                 <svrl:text>a finding of no name</svrl:text>
               </svrl:failed-assert>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_without_a_flag() {
        let expected = Err(Error::UnknownFlag(String::new()));

        let actual = Svrl.unwrap(&report(
            r#"<svrl:failed-assert test="cbc:ID" id="BR-CO-16" location="/ubl:Invoice">
                 <svrl:text>a finding without a flag</svrl:text>
               </svrl:failed-assert>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_finding_of_an_unknown_flag() {
        let expected = Err(Error::UnknownFlag("hint".to_owned()));

        let actual = Svrl.unwrap(&report(
            r#"<svrl:failed-assert test="cbc:ID" id="BR-CO-16" flag="hint" location="/ubl:Invoice">
                 <svrl:text>a hint</svrl:text>
               </svrl:failed-assert>"#,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_report_of_another_shape() {
        let actual = Svrl.unwrap("<html><body>Forbidden</body></html>");

        assert!(matches!(actual, Err(Error::Malformed(_))));
    }

    #[test]
    fn rejects_a_malformed_report() {
        let actual = Svrl.unwrap("<svrl:schematron-output");

        assert!(matches!(actual, Err(Error::Malformed(_))));
    }

    #[test]
    fn rejects_an_empty_answer() {
        let actual = Svrl.unwrap("");

        assert!(matches!(actual, Err(Error::Malformed(_))));
    }
}
