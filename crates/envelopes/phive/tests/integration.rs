//! Integration test that reads the answer of the live phive validation service.
//!
//! The test needs the services of step 2. Start them with `cargo make env-up`, then run:
//!
//! ```sh
//! cargo test -p en16931-phive --test integration -- --ignored
//! ```

use en16931_core::{Binding, DocumentKind, Entry, Profile, Severity, Target, Wrapper};
use en16931_phive::Phive;

// A Peppol invoice with a broken line sum, which trips `BR-CO-10` and `BR-CO-13`.
const INVOICE: &str = include_str!("fixtures/invoice.xml");

// Sends the invoice to the service and returns its answer.
fn answer() -> String {
    let base = std::env::var("PHIVE_URL").unwrap_or_else(|_| "http://localhost:8083".to_owned());
    let token = std::env::var("PHIVE_TOKEN").unwrap_or_else(|_| "phorm-dev-token".to_owned());
    let rules = Phive
        .vendor_id(Target {
            profile: Profile::PeppolBisBilling30,
            binding: Binding::Ubl,
            kind: DocumentKind::Invoice,
        })
        .expect("a rule set for the profile");
    reqwest::blocking::Client::new()
        .post(format!("{base}/api/validate/{rules}:latest"))
        .header("Content-Type", "application/xml")
        .header("Accept", "application/xml")
        .header("X-Token", token)
        .body(INVOICE)
        .send()
        .expect("the validator request to succeed")
        .text()
        .expect("a response body")
}

#[test]
#[ignore = "requires live validators (cargo make env-up)"]
fn reads_the_findings_of_the_live_service() {
    let expected = Ok(vec![
        Entry {
            severity: Severity::Error,
            code: Some("BR-CO-10".to_owned()),
            text: "Sum of Invoice line net amount (BT-106) = Σ Invoice line net amount (BT-131)."
                .to_owned(),
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

    let actual = Phive.unwrap(&answer());

    assert_eq!(actual, expected);
}
