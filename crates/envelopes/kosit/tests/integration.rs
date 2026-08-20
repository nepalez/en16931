//! Integration test that reads the answer of the live KoSIT validation tool.
//!
//! The test needs the services of step 2. Start them with `cargo make env-up`, then run:
//!
//! ```sh
//! cargo test -p en16931-kosit --test integration -- --ignored
//! ```

use en16931_core::{Entry, Severity, Wrapper};
use en16931_kosit::Kosit;

// An XRechnung invoice with a broken payable amount, which trips `BR-CO-16`.
const INVOICE: &str = include_str!("fixtures/invoice.xml");

// Sends the invoice to the tool and returns its report.
fn answer() -> String {
    let url = std::env::var("KOSIT_URL").unwrap_or_else(|_| "http://localhost:8082".to_owned());
    reqwest::blocking::Client::new()
        .post(url)
        .header("Content-Type", "application/xml")
        .body(INVOICE)
        .send()
        .expect("the validator request to succeed")
        .text()
        .expect("a response body")
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn reads_the_findings_of_the_live_tool() {
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

    let actual = Kosit.unwrap(&answer());

    assert_eq!(actual, expected);
}
