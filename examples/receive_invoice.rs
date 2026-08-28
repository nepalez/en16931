//! Receives an invoice:
//! * takes the XML text that arrived from the seller,
//! * parses it with automatic binding detection,
//! * checks the document at the phive service,
//! * reads the report to highlight model fields,
//! * and takes the business object out.
//!
//! The example needs a live validator. Start the services first (`cargo make env-up`), then run:
//!
//! ```sh
//! cargo run -p en16931-examples --example receive_invoice
//! ```

use en16931_core::{Document, Invoice, RawReport, Wrapper};
use en16931_iso::Iso;
use en16931_phive::Phive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the received XML: the binding is recognized automatically
    // from the root element (here it turns out to be CII).
    let document = Document::parse(RECEIVED_XML)?;

    // Check the document by a validator of your own, as the seller did.
    let answer = post(&document)?;

    // Parse the report received from the validator.
    let report = RawReport::parse(&answer, &Phive, &Iso)?;

    match document.check(report)? {
        Ok(valid) => {
            // A report without errors may still carry some warnings.
            for problem in valid.problems() {
                println!("{problem}");
            }

            // On success, take the business object out of the document.
            let Invoice {
                number, issue_date, ..
            } = valid.into();
            println!("Received invoice {number} issued on {issue_date}");
        }
        Err(invalid) => {
            println!("The validator rejected the invoice.");
            for problem in invalid.problems() {
                println!("{problem}");
            }
        }
    }
    Ok(())
}

// Sends the document to the phive service under the rule set its target names.
// This responsibility is up to the library user.
fn post(document: &Document) -> Result<String, Box<dyn std::error::Error>> {
    let base = std::env::var("PHIVE_URL").unwrap_or_else(|_| "http://localhost:8083".to_owned());
    let token = std::env::var("PHIVE_TOKEN").unwrap_or_else(|_| "phorm-dev-token".to_owned());
    let rules = Phive.vendor_id(document.target())?;

    Ok(reqwest::blocking::Client::new()
        .post(format!("{base}/api/validate/{rules}:latest"))
        .header("Content-Type", "application/xml")
        .header("Accept", "application/xml")
        .header("X-Token", token)
        .body(document.xml().to_owned())
        .send()?
        .text()?)
}

// The XML document as it arrived from the seller.
const RECEIVED_XML: &str = r#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100" xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100">
  <rsm:ExchangedDocumentContext>
    <ram:BusinessProcessSpecifiedDocumentContextParameter>
      <ram:ID>urn:fdc:peppol.eu:2017:poacc:billing:01:1.0</ram:ID>
    </ram:BusinessProcessSpecifiedDocumentContextParameter>
    <ram:GuidelineSpecifiedDocumentContextParameter>
      <ram:ID>urn:cen.eu:en16931:2017</ram:ID>
    </ram:GuidelineSpecifiedDocumentContextParameter>
  </rsm:ExchangedDocumentContext>
  <rsm:ExchangedDocument>
    <ram:ID>INV-2026-001</ram:ID>
    <ram:TypeCode>380</ram:TypeCode>
    <ram:IssueDateTime>
      <udt:DateTimeString format="102">20260115</udt:DateTimeString>
    </ram:IssueDateTime>
  </rsm:ExchangedDocument>
  <rsm:SupplyChainTradeTransaction>
    <ram:IncludedSupplyChainTradeLineItem>
      <ram:AssociatedDocumentLineDocument>
        <ram:LineID>1</ram:LineID>
      </ram:AssociatedDocumentLineDocument>
      <ram:SpecifiedTradeProduct>
        <ram:Name>Item name</ram:Name>
      </ram:SpecifiedTradeProduct>
      <ram:SpecifiedLineTradeAgreement>
        <ram:GrossPriceProductTradePrice>
          <ram:ChargeAmount>100.00</ram:ChargeAmount>
        </ram:GrossPriceProductTradePrice>
        <ram:NetPriceProductTradePrice>
          <ram:ChargeAmount>100.00</ram:ChargeAmount>
        </ram:NetPriceProductTradePrice>
      </ram:SpecifiedLineTradeAgreement>
      <ram:SpecifiedLineTradeDelivery>
        <ram:BilledQuantity unitCode="C62">2</ram:BilledQuantity>
      </ram:SpecifiedLineTradeDelivery>
      <ram:SpecifiedLineTradeSettlement>
        <ram:ApplicableTradeTax>
          <ram:TypeCode>VAT</ram:TypeCode>
          <ram:CategoryCode>S</ram:CategoryCode>
          <ram:RateApplicablePercent>19</ram:RateApplicablePercent>
        </ram:ApplicableTradeTax>
        <ram:SpecifiedTradeSettlementLineMonetarySummation>
          <ram:LineTotalAmount>200.00</ram:LineTotalAmount>
        </ram:SpecifiedTradeSettlementLineMonetarySummation>
      </ram:SpecifiedLineTradeSettlement>
    </ram:IncludedSupplyChainTradeLineItem>
    <ram:ApplicableHeaderTradeAgreement>
      <ram:BuyerReference>04011000-12345-03</ram:BuyerReference>
      <ram:SellerTradeParty>
        <ram:Name>Seller Official Name</ram:Name>
        <ram:SpecifiedLegalOrganization>
          <ram:ID>DE123456</ram:ID>
        </ram:SpecifiedLegalOrganization>
        <ram:DefinedTradeContact>
          <ram:PersonName>Anna Seller</ram:PersonName>
          <ram:TelephoneUniversalCommunication>
            <ram:CompleteNumber>+49 30 123456</ram:CompleteNumber>
          </ram:TelephoneUniversalCommunication>
          <ram:EmailURIUniversalCommunication>
            <ram:URIID>anna@example.de</ram:URIID>
          </ram:EmailURIUniversalCommunication>
        </ram:DefinedTradeContact>
        <ram:PostalTradeAddress>
          <ram:PostcodeCode>10115</ram:PostcodeCode>
          <ram:LineOne>Main street 1</ram:LineOne>
          <ram:CityName>Berlin</ram:CityName>
          <ram:CountryID>DE</ram:CountryID>
        </ram:PostalTradeAddress>
        <ram:URIUniversalCommunication>
          <ram:URIID schemeID="0088">4035811991007</ram:URIID>
        </ram:URIUniversalCommunication>
        <ram:SpecifiedTaxRegistration>
          <ram:ID schemeID="VA">DE123456789</ram:ID>
        </ram:SpecifiedTaxRegistration>
      </ram:SellerTradeParty>
      <ram:BuyerTradeParty>
        <ram:Name>Buyer Official Name</ram:Name>
        <ram:PostalTradeAddress>
          <ram:PostcodeCode>10115</ram:PostcodeCode>
          <ram:LineOne>Main street 1</ram:LineOne>
          <ram:CityName>Berlin</ram:CityName>
          <ram:CountryID>DE</ram:CountryID>
        </ram:PostalTradeAddress>
        <ram:URIUniversalCommunication>
          <ram:URIID schemeID="0088">4035812991006</ram:URIID>
        </ram:URIUniversalCommunication>
      </ram:BuyerTradeParty>
    </ram:ApplicableHeaderTradeAgreement>
    <ram:ApplicableHeaderTradeDelivery>
    </ram:ApplicableHeaderTradeDelivery>
    <ram:ApplicableHeaderTradeSettlement>
      <ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>
      <ram:SpecifiedTradeSettlementPaymentMeans>
        <ram:TypeCode>30</ram:TypeCode>
        <ram:PayeePartyCreditorFinancialAccount>
          <ram:IBANID>DE89370400440532013000</ram:IBANID>
        </ram:PayeePartyCreditorFinancialAccount>
      </ram:SpecifiedTradeSettlementPaymentMeans>
      <ram:ApplicableTradeTax>
        <ram:CalculatedAmount>38.00</ram:CalculatedAmount>
        <ram:TypeCode>VAT</ram:TypeCode>
        <ram:BasisAmount>200.00</ram:BasisAmount>
        <ram:CategoryCode>S</ram:CategoryCode>
        <ram:RateApplicablePercent>19</ram:RateApplicablePercent>
      </ram:ApplicableTradeTax>
      <ram:BillingSpecifiedPeriod>
        <ram:StartDateTime>
          <udt:DateTimeString format="102">20260101</udt:DateTimeString>
        </ram:StartDateTime>
        <ram:EndDateTime>
          <udt:DateTimeString format="102">20260131</udt:DateTimeString>
        </ram:EndDateTime>
      </ram:BillingSpecifiedPeriod>
      <ram:SpecifiedTradePaymentTerms>
        <ram:Description>Payable within 30 days</ram:Description>
        <ram:DueDateDateTime>
          <udt:DateTimeString format="102">20260215</udt:DateTimeString>
        </ram:DueDateDateTime>
      </ram:SpecifiedTradePaymentTerms>
      <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
        <ram:LineTotalAmount>200.00</ram:LineTotalAmount>
        <ram:TaxBasisTotalAmount>200.00</ram:TaxBasisTotalAmount>
        <ram:TaxTotalAmount currencyID="EUR">38.00</ram:TaxTotalAmount>
        <ram:GrandTotalAmount>238.00</ram:GrandTotalAmount>
        <ram:DuePayableAmount>238.00</ram:DuePayableAmount>
      </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
    </ram:ApplicableHeaderTradeSettlement>
  </rsm:SupplyChainTradeTransaction>
</rsm:CrossIndustryInvoice>"#;
