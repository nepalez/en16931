//! Parsing of UBL XML back into the document model.
//!
//! The parser mirrors the serializer element by element, driving the same trace
//! so it rebuilds the identical dictionary while reconstructing the model.

use base64::Engine as _;

use crate::format::trace::Trace;
use crate::format::ubl::{CAC, CBC, INV};
use crate::prelude::*;
use crate::{
    Abbreviations, Adjustment, AdjustmentAmount, AdjustmentReason, Amount, BinaryObject, Buyer,
    Classification, Contact, CreditTransfer, Delivery, Dictionary, DirectDebit, DocumentBuilder,
    ElectronicAddress, Error, ExemptionReason, Invoice, InvoiceLine, Item, ItemAttribute,
    LegalEntity, LineAdjustment, LocationReference, MimeCode, Namespace, NonEmptyString, Note,
    ObjectReference, OperationalEntity, Payee, PaymentCard, PaymentDetails, PaymentInstructions,
    Period, PostalAddress, PrecedingInvoice, Price, Quantity, Seller, SupportingDocument,
    TaxRepresentative, Unit, VatCategory, VatPoint, VatTreatment,
};

/// Parses a UBL document from XML,
/// rebuilding the dictionary and the abbreviations on the inverse path.
pub(crate) fn deserialize(
    xml: &str,
) -> Result<(DocumentBuilder, Dictionary, Abbreviations), Error> {
    let (tokens, abbreviations) = tokenize(xml)?;
    let mut parser = Parser {
        tokens,
        cursor: 0,
        trace: Trace::new(),
    };
    let builder = parser.document()?;
    Ok((builder, parser.trace.into_dictionary(), abbreviations))
}

// ---- tokens --------------------------------------------------------------

enum Token {
    Open {
        namespace: Namespace,
        name: String,
        attributes: Vec<(String, String)>,
    },
    Text(String),
    Close,
}

// Reads the document into resolved, owned tokens, collecting the abbreviations it
// declares and dropping insignificant whitespace.
fn tokenize(xml: &str) -> Result<(Vec<Token>, Abbreviations), Error> {
    let mut reader = NsReader::from_str(xml);
    let mut tokens = Vec::new();
    let mut abbreviations = Abbreviations::default();
    loop {
        let (resolved, event) = reader.read_resolved_event()?;
        match event {
            Event::Start(start) => {
                tokens.push(open_token(resolved, &start, &mut abbreviations)?);
            }
            Event::Empty(start) => {
                tokens.push(open_token(resolved, &start, &mut abbreviations)?);
                tokens.push(Token::Close);
            }
            Event::End(_) => tokens.push(Token::Close),
            Event::Text(value) => {
                let bytes = value.into_inner();
                let text = String::from_utf8_lossy(&bytes);
                if !text.trim().is_empty() {
                    tokens.push(Token::Text(text.into_owned()));
                }
            }
            Event::Eof => return Ok((tokens, abbreviations)),
            _ => {}
        }
    }
}

// Builds an `Open` token from a start tag, resolving its namespace and reading
// its attributes. A namespace declaration binds an abbreviation instead.
fn open_token(
    resolved: ResolveResult<'_>,
    start: &BytesStart<'_>,
    abbreviations: &mut Abbreviations,
) -> Result<Token, Error> {
    let ResolveResult::Bound(uri) = resolved else {
        return Err(Error::malformed_xml("an element has no namespace"));
    };
    let uri = String::from_utf8_lossy(uri.into_inner());
    let namespace = Namespace::from_uri(&uri)
        .ok_or_else(|| Error::malformed_xml(format!("unknown namespace: {uri}")))?;
    let name = String::from_utf8_lossy(start.local_name().as_ref()).into_owned();
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        if key == b"xmlns" || key.starts_with(b"xmlns:") {
            let abbreviation = String::from_utf8_lossy(key.strip_prefix(b"xmlns:").unwrap_or(b""));
            let uri = String::from_utf8_lossy(&attribute.value);
            if let Some(namespace) = Namespace::from_uri(&uri) {
                abbreviations.declare(&abbreviation, namespace)?;
            }
            continue;
        }
        let key = String::from_utf8_lossy(attribute.key.local_name().as_ref()).into_owned();
        let value = String::from_utf8_lossy(&attribute.value).into_owned();
        attributes.push((key, value));
    }
    Ok(Token::Open {
        namespace,
        name,
        attributes,
    })
}

// ---- parser --------------------------------------------------------------

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    trace: Trace,
}

impl Parser {
    fn document(&mut self) -> Result<DocumentBuilder, Error> {
        self.take_open(INV, "Invoice")?;
        self.trace.enter(INV, "Invoice");
        self.trace.record_root();

        let profile = self.rooted(CBC, "CustomizationID")?.parse()?;
        let business_process = if self.is_open(CBC, "ProfileID") {
            Some(self.rooted(CBC, "ProfileID")?.parse()?)
        } else {
            None
        };

        let number = self.leaf(CBC, "ID", "number")?.parse()?;
        let issue_date = parse_date(&self.leaf(CBC, "IssueDate", "issue_date")?)?;
        let payment_due_date = self.optional_date(CBC, "DueDate", "payment_due_date")?;
        let type_code = self.leaf(CBC, "InvoiceTypeCode", "type_code")?.parse()?;

        let mut notes = Vec::new();
        while self.is_open(CBC, "Note") {
            let instance = index(notes.len());
            let text = self.repeatable_leaf(CBC, "Note", "notes", instance)?;
            notes.push(parse_note(&text)?);
        }

        let mut vat_point = None;
        if self.is_open(CBC, "TaxPointDate") {
            vat_point = Some(VatPoint::Date(parse_date(&self.leaf(
                CBC,
                "TaxPointDate",
                "vat_point",
            )?)?));
        }

        let currency = parse_currency(&self.leaf(CBC, "DocumentCurrencyCode", "currency")?)?;
        let accounting_currency = if self.is_open(CBC, "TaxCurrencyCode") {
            Some(parse_currency(&self.leaf(
                CBC,
                "TaxCurrencyCode",
                "vat_accounting_total",
            )?)?)
        } else {
            None
        };

        let buyer_accounting_reference =
            self.optional_leaf(CBC, "AccountingCost", "buyer_accounting_reference")?;
        let buyer_reference = self.optional_leaf(CBC, "BuyerReference", "buyer_reference")?;

        let mut invoicing_period = None;
        if self.is_open(CAC, "InvoicePeriod") {
            let (period, event) = self.parse_invoice_period()?;
            invoicing_period = period;
            if let Some(point) = event {
                vat_point = Some(point);
            }
        }

        let (purchase_order_reference, sales_order_reference) =
            if self.is_open(CAC, "OrderReference") {
                self.parse_order_reference()?
            } else {
                (None, None)
            };

        let mut preceding_invoices = Vec::new();
        while self.is_open(CAC, "BillingReference") {
            let instance = index(preceding_invoices.len());
            preceding_invoices.push(self.parse_billing_reference(instance)?);
        }

        let despatch_advice_reference = self.optional_reference(
            CAC,
            "DespatchDocumentReference",
            "despatch_advice_reference",
        )?;
        let receiving_advice_reference = self.optional_reference(
            CAC,
            "ReceiptDocumentReference",
            "receiving_advice_reference",
        )?;
        let tender_or_lot_reference = self.optional_reference(
            CAC,
            "OriginatorDocumentReference",
            "tender_or_lot_reference",
        )?;
        let contract_reference =
            self.optional_reference(CAC, "ContractDocumentReference", "contract_reference")?;

        let mut object = None;
        let mut supporting_documents = Vec::new();
        while self.is_open(CAC, "AdditionalDocumentReference") {
            match self.parse_additional_document(supporting_documents.len())? {
                AdditionalDocument::Object(reference) => object = Some(reference),
                AdditionalDocument::Supporting(document) => supporting_documents.push(document),
            }
        }

        let project_reference =
            self.optional_reference(CAC, "ProjectReference", "project_reference")?;

        let seller = self.parse_supplier_party()?;
        let buyer = self.parse_customer_party()?;
        let payee = if self.is_open(CAC, "PayeeParty") {
            Some(self.parse_payee_party()?)
        } else {
            None
        };
        let tax_representative = if self.is_open(CAC, "TaxRepresentativeParty") {
            Some(self.parse_tax_representative_party()?)
        } else {
            None
        };
        let delivery = if self.is_open(CAC, "Delivery") {
            Some(self.parse_delivery()?)
        } else {
            None
        };
        let payment = if self.is_open(CAC, "PaymentMeans") {
            Some(self.parse_payment_means()?)
        } else {
            None
        };
        let payment_terms = if self.is_open(CAC, "PaymentTerms") {
            Some(self.parse_payment_terms()?)
        } else {
            None
        };

        let mut adjustments = Vec::new();
        while self.is_open(CAC, "AllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_adjustment(instance)?);
        }

        let mut exemptions = ExemptionMap::new();
        let mut accounting_value = None;
        if self.is_open(CAC, "TaxTotal") {
            exemptions = self.parse_tax_total()?;
        }
        if self.is_open(CAC, "TaxTotal") {
            accounting_value = Some(self.parse_accounting_tax_total()?);
        }
        let (paid, rounding) = if self.is_open(CAC, "LegalMonetaryTotal") {
            self.parse_legal_monetary_total()?
        } else {
            (None, None)
        };

        let mut lines = Vec::new();
        while self.is_open(CAC, "InvoiceLine") {
            let instance = index(lines.len());
            lines.push(self.parse_line(instance)?);
        }

        self.take_close()?;
        self.trace.leave();

        let vat_accounting_total = match (accounting_currency, accounting_value) {
            (Some(currency), Some(value)) => Some(Amount { value, currency }),
            _ => None,
        };

        let mut invoice = Invoice {
            number,
            issue_date,
            type_code,
            currency,
            vat_accounting_total,
            vat_point,
            payment_due_date,
            buyer_reference,
            project_reference,
            contract_reference,
            purchase_order_reference,
            sales_order_reference,
            receiving_advice_reference,
            despatch_advice_reference,
            tender_or_lot_reference,
            object,
            buyer_accounting_reference,
            payment_terms,
            notes,
            preceding_invoices,
            seller,
            buyer,
            payee,
            tax_representative,
            delivery,
            invoicing_period,
            adjustments,
            rounding,
            payment,
            paid,
            supporting_documents,
            lines,
        };
        exemptions.apply(&mut invoice);

        Ok(DocumentBuilder {
            invoice,
            profile,
            binding: crate::Binding::Ubl,
            business_process,
        })
    }

    // Parses the invoice period, returning the period and the VAT point event.
    fn parse_invoice_period(&mut self) -> Result<(Option<Period>, Option<VatPoint>), Error> {
        self.enter_group(CAC, "InvoicePeriod", "invoicing_period")?;
        let start = self.optional_date(CBC, "StartDate", "invoicing_period")?;
        let end = self.optional_date(CBC, "EndDate", "invoicing_period")?;
        let event = if self.is_open(CBC, "DescriptionCode") {
            Some(VatPoint::Event(
                self.leaf(CBC, "DescriptionCode", "vat_point")?.parse()?,
            ))
        } else {
            None
        };
        self.leave_group()?;
        Ok((period_from(start, end), event))
    }

    // Parses the order and sales order references.
    fn parse_order_reference(
        &mut self,
    ) -> Result<(Option<NonEmptyString>, Option<NonEmptyString>), Error> {
        self.enter_structural(CAC, "OrderReference")?;
        let order = self.optional_leaf(CBC, "ID", "purchase_order_reference")?;
        let sales = self.optional_leaf(CBC, "SalesOrderID", "sales_order_reference")?;
        self.leave_structural()?;
        Ok((order, sales))
    }

    // Parses one preceding invoice reference.
    fn parse_billing_reference(
        &mut self,
        instance: NonZeroUsize,
    ) -> Result<PrecedingInvoice, Error> {
        self.enter_repeatable(CAC, "BillingReference", "preceding_invoices", instance)?;
        self.enter_structural(CAC, "InvoiceDocumentReference")?;
        let number = self.leaf(CBC, "ID", "number")?.parse()?;
        let issue_date = self.optional_date(CBC, "IssueDate", "issue_date")?;
        self.leave_structural()?;
        self.leave_repeatable()?;
        Ok(PrecedingInvoice { number, issue_date })
    }

    // Parses one additional document reference into the object or a supporting document.
    fn parse_additional_document(
        &mut self,
        supporting: usize,
    ) -> Result<AdditionalDocument, Error> {
        // Peek to know whether it is the invoiced object before committing an instance index.
        let is_object = self.additional_is_object();
        if is_object {
            self.enter_group(CAC, "AdditionalDocumentReference", "object")?;
            let (attributes, id) = self.derived(CBC, "ID")?;
            let id = id.parse()?;
            let scheme = match attr(&attributes, "schemeID") {
                Some(value) => Some(value.parse()?),
                None => None,
            };
            self.derived(CBC, "DocumentTypeCode")?;
            self.leave_group()?;
            Ok(AdditionalDocument::Object(ObjectReference { id, scheme }))
        } else {
            let instance = index(supporting);
            self.enter_repeatable(
                CAC,
                "AdditionalDocumentReference",
                "supporting_documents",
                instance,
            )?;
            let reference = self.leaf(CBC, "ID", "reference")?.parse()?;
            let description = self.optional_leaf(CBC, "DocumentDescription", "description")?;
            let mut external_location = None;
            let mut attachment = None;
            if self.is_open(CAC, "Attachment") {
                self.enter_structural(CAC, "Attachment")?;
                if self.is_open(CAC, "ExternalReference") {
                    self.enter_structural(CAC, "ExternalReference")?;
                    external_location =
                        Some(parse_url(&self.leaf(CBC, "URI", "external_location")?)?);
                    self.leave_structural()?;
                } else {
                    attachment = Some(self.parse_binary()?);
                }
                self.leave_structural()?;
            }
            self.leave_repeatable()?;
            Ok(AdditionalDocument::Supporting(SupportingDocument {
                reference,
                description,
                external_location,
                attachment,
            }))
        }
    }

    // Parses an embedded binary object attachment.
    fn parse_binary(&mut self) -> Result<BinaryObject, Error> {
        let (attributes, encoded) = self.derived(CBC, "EmbeddedDocumentBinaryObject")?;
        let mime = attr(&attributes, "mimeCode")
            .ok_or_else(|| bad("a binary object without a mime code"))?;
        let mime: MimeCode = mime.parse()?;
        let filename = attr(&attributes, "filename")
            .ok_or_else(|| bad("a binary object without a filename"))?;
        let content = base64::engine::general_purpose::STANDARD
            .decode(encoded.trim())
            .map_err(|error| Error::malformed_xml(error.to_string()).caused_by(error))?;
        BinaryObject::new(content, mime, filename)
    }

    // Parses the supplier party into a seller.
    fn parse_supplier_party(&mut self) -> Result<Seller, Error> {
        self.enter_structural(CAC, "AccountingSupplierParty")?;
        self.enter_group(CAC, "Party", "seller")?;
        let electronic_address = self.optional_endpoint("electronic_address")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let trading_name = self.optional_party_name("trading_name")?;
        let address = self.parse_address(CAC, "PostalAddress")?;
        let mut vat = None;
        let mut tax_registration = None;
        while self.is_open(CAC, "PartyTaxScheme") {
            match self.parse_party_tax_scheme()? {
                PartyTaxScheme::Vat(value) => vat = Some(value),
                PartyTaxScheme::Other(value) => tax_registration = Some(value),
            }
        }
        let (name, legal_entity, additional_legal_information) = self.parse_legal_entity("name")?;
        let contact = self.optional_contact("contact")?;
        self.leave_group()?;
        self.leave_structural()?;
        Ok(Seller {
            name,
            trading_name,
            identifiers,
            legal_entity,
            additional_legal_information,
            vat,
            tax_registration,
            electronic_address,
            address,
            contact,
        })
    }

    // Parses the customer party into a buyer.
    fn parse_customer_party(&mut self) -> Result<Buyer, Error> {
        self.enter_structural(CAC, "AccountingCustomerParty")?;
        self.enter_group(CAC, "Party", "buyer")?;
        let electronic_address = self.optional_endpoint("electronic_address")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let trading_name = self.optional_party_name("trading_name")?;
        let address = self.parse_address(CAC, "PostalAddress")?;
        let mut vat = None;
        while self.is_open(CAC, "PartyTaxScheme") {
            if let PartyTaxScheme::Vat(value) = self.parse_party_tax_scheme()? {
                vat = Some(value);
            }
        }
        let (name, legal_entity, _) = self.parse_legal_entity("name")?;
        let contact = self.optional_contact("contact")?;
        self.leave_group()?;
        self.leave_structural()?;
        Ok(Buyer {
            name,
            trading_name,
            identifiers,
            legal_entity,
            vat,
            electronic_address,
            address,
            contact,
        })
    }

    // Parses the payee party.
    fn parse_payee_party(&mut self) -> Result<Payee, Error> {
        self.enter_group(CAC, "PayeeParty", "payee")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.parse_party_name("name")?;
        let legal_entity = if self.is_open(CAC, "PartyLegalEntity") {
            self.enter_structural(CAC, "PartyLegalEntity")?;
            let entity = self.parse_company_id("legal_entity")?;
            self.leave_structural()?;
            Some(entity)
        } else {
            None
        };
        self.leave_group()?;
        Ok(Payee {
            name,
            identifiers,
            legal_entity,
        })
    }

    // Parses the tax representative party.
    fn parse_tax_representative_party(&mut self) -> Result<TaxRepresentative, Error> {
        self.enter_group(CAC, "TaxRepresentativeParty", "tax_representative")?;
        let name = self.parse_party_name("name")?;
        let address = self.parse_address(CAC, "PostalAddress")?;
        let vat = match self.parse_party_tax_scheme()? {
            PartyTaxScheme::Vat(value) => value,
            PartyTaxScheme::Other(_) => {
                return Err(bad("a tax representative without a VAT scheme"));
            }
        };
        self.leave_group()?;
        Ok(TaxRepresentative { name, vat, address })
    }

    // Parses the delivery information.
    fn parse_delivery(&mut self) -> Result<Delivery, Error> {
        self.enter_group(CAC, "Delivery", "delivery")?;
        let date = self.optional_date(CBC, "ActualDeliveryDate", "date")?;
        let mut location = None;
        let mut address = None;
        if self.is_open(CAC, "DeliveryLocation") {
            self.enter_structural(CAC, "DeliveryLocation")?;
            if self.is_open(CBC, "ID") {
                let (attributes, id) = self.leaf_attr(CBC, "ID", "location")?;
                let issuer = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
                location = Some(LocationReference {
                    id: id.parse()?,
                    issuer,
                });
            }
            if self.is_open(CAC, "Address") {
                address = Some(self.parse_address(CAC, "Address")?);
            }
            self.leave_structural()?;
        }
        let name = if self.is_open(CAC, "DeliveryParty") {
            self.enter_structural(CAC, "DeliveryParty")?;
            let name = self.parse_party_name("name")?;
            self.leave_structural()?;
            Some(name)
        } else {
            None
        };
        self.leave_group()?;
        Ok(Delivery {
            name,
            location,
            date,
            address,
        })
    }

    // Parses the payment means.
    fn parse_payment_means(&mut self) -> Result<PaymentInstructions, Error> {
        self.enter_group(CAC, "PaymentMeans", "payment")?;
        let (attributes, code) = self.leaf_attr(CBC, "PaymentMeansCode", "means")?;
        let means = code.parse()?;
        let means_text = match attr(&attributes, "name") {
            Some(value) => Some(value.parse()?),
            None => None,
        };
        let remittance_information =
            self.optional_leaf(CBC, "PaymentID", "remittance_information")?;
        let details = if self.is_open(CAC, "PayeeFinancialAccount") {
            let mut transfers = Vec::new();
            while self.is_open(CAC, "PayeeFinancialAccount") {
                transfers.push(self.parse_credit_transfer()?);
            }
            Some(PaymentDetails::CreditTransfers(transfers))
        } else if self.is_open(CAC, "CardAccount") {
            Some(PaymentDetails::Card(self.parse_card()?))
        } else if self.is_open(CAC, "PaymentMandate") {
            Some(PaymentDetails::DirectDebit(self.parse_direct_debit()?))
        } else {
            None
        };
        self.leave_group()?;
        Ok(PaymentInstructions {
            means,
            means_text,
            remittance_information,
            details,
        })
    }

    fn parse_credit_transfer(&mut self) -> Result<CreditTransfer, Error> {
        self.enter_structural(CAC, "PayeeFinancialAccount")?;
        let account = self.leaf(CBC, "ID", "account")?.parse()?;
        let account_name = self.optional_leaf(CBC, "Name", "account_name")?;
        let provider = if self.is_open(CAC, "FinancialInstitutionBranch") {
            self.enter_structural(CAC, "FinancialInstitutionBranch")?;
            let bic = self.leaf(CBC, "ID", "provider")?.parse()?;
            self.leave_structural()?;
            Some(bic)
        } else {
            None
        };
        self.leave_structural()?;
        Ok(CreditTransfer {
            account,
            account_name,
            provider,
        })
    }

    fn parse_card(&mut self) -> Result<PaymentCard, Error> {
        self.enter_structural(CAC, "CardAccount")?;
        let primary_account_number = self
            .leaf(CBC, "PrimaryAccountNumberID", "primary_account_number")?
            .parse()?;
        let holder_name = self.optional_leaf(CBC, "HolderName", "holder_name")?;
        self.leave_structural()?;
        Ok(PaymentCard {
            primary_account_number,
            holder_name,
        })
    }

    fn parse_direct_debit(&mut self) -> Result<DirectDebit, Error> {
        self.enter_structural(CAC, "PaymentMandate")?;
        let mandate_reference = self.optional_leaf(CBC, "ID", "mandate_reference")?;
        let creditor_identifier = self.optional_leaf(CBC, "PayerPartyID", "creditor_identifier")?;
        let debited_account = if self.is_open(CAC, "PayerFinancialAccount") {
            self.enter_structural(CAC, "PayerFinancialAccount")?;
            let account = self.leaf(CBC, "ID", "debited_account")?.parse()?;
            self.leave_structural()?;
            Some(account)
        } else {
            None
        };
        self.leave_structural()?;
        Ok(DirectDebit {
            mandate_reference,
            creditor_identifier,
            debited_account,
        })
    }

    // Parses the payment terms.
    fn parse_payment_terms(&mut self) -> Result<NonEmptyString, Error> {
        self.enter_group(CAC, "PaymentTerms", "payment_terms")?;
        let note = self.leaf(CBC, "Note", "payment_terms")?.parse()?;
        self.leave_group()?;
        Ok(note)
    }

    // Parses one document-level allowance or charge.
    fn parse_adjustment(&mut self, instance: NonZeroUsize) -> Result<Adjustment, Error> {
        self.enter_repeatable(CAC, "AllowanceCharge", "adjustments", instance)?;
        let charge = self.derived(CBC, "ChargeIndicator")?.1.trim() == "true";
        let reason = self.parse_reason(charge)?;
        let amount = self.parse_adjustment_amount()?;
        let vat = self.parse_tax_category()?;
        self.leave_repeatable()?;
        Ok(Adjustment {
            amount,
            vat,
            reason,
        })
    }

    // Parses the reason code and text of an adjustment.
    fn parse_reason(&mut self, charge: bool) -> Result<AdjustmentReason, Error> {
        let code = if self.is_open(CBC, "AllowanceChargeReasonCode") {
            Some(self.derived(CBC, "AllowanceChargeReasonCode")?.1)
        } else {
            None
        };
        let text = if self.is_open(CBC, "AllowanceChargeReason") {
            Some(self.derived(CBC, "AllowanceChargeReason")?.1.parse()?)
        } else {
            None
        };
        Ok(if charge {
            AdjustmentReason::Charge {
                code: match code {
                    Some(code) => Some(code.parse()?),
                    None => None,
                },
                text,
            }
        } else {
            AdjustmentReason::Allowance {
                code: match code {
                    Some(code) => Some(code.parse()?),
                    None => None,
                },
                text,
            }
        })
    }

    // Parses the amount of an adjustment, absolute or relative.
    fn parse_adjustment_amount(&mut self) -> Result<AdjustmentAmount, Error> {
        let factor = if self.is_open(CBC, "MultiplierFactorNumeric") {
            Some(self.derived(CBC, "MultiplierFactorNumeric")?.1)
        } else {
            None
        };
        let amount = self.derived(CBC, "Amount")?.1;
        match factor {
            Some(factor) => {
                let base = parse_decimal(&self.derived(CBC, "BaseAmount")?.1)?;
                Ok(AdjustmentAmount::Relative {
                    rate: factor.parse()?,
                    base,
                })
            }
            None => Ok(AdjustmentAmount::Absolute(parse_decimal(&amount)?)),
        }
    }

    // Parses an allowance or charge VAT category.
    fn parse_tax_category(&mut self) -> Result<VatTreatment, Error> {
        self.enter_nested(CAC, "TaxCategory")?;
        let category: VatCategory = self.derived(CBC, "ID")?.1.parse()?;
        let rate = self.derived(CBC, "Percent")?.1.parse()?;
        self.enter_nested(CAC, "TaxScheme")?;
        self.derived(CBC, "ID")?;
        self.leave_nested()?;
        self.leave_nested()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    // Parses the tax total (`BG-23`), collecting exemption reasons per category.
    fn parse_tax_total(&mut self) -> Result<ExemptionMap, Error> {
        let mut exemptions = ExemptionMap::new();
        self.enter_structural(CAC, "TaxTotal")?;
        self.derived(CBC, "TaxAmount")?;
        while self.is_open(CAC, "TaxSubtotal") {
            self.enter_structural(CAC, "TaxSubtotal")?;
            self.derived(CBC, "TaxableAmount")?;
            self.derived(CBC, "TaxAmount")?;
            self.enter_structural(CAC, "TaxCategory")?;
            let category: VatCategory = self.derived(CBC, "ID")?.1.parse()?;
            self.derived(CBC, "Percent")?;
            let mut code = None;
            let mut text = None;
            if self.is_open(CBC, "TaxExemptionReasonCode") {
                code = Some(self.derived(CBC, "TaxExemptionReasonCode")?.1.parse()?);
            }
            if self.is_open(CBC, "TaxExemptionReason") {
                text = Some(self.derived(CBC, "TaxExemptionReason")?.1.parse()?);
            }
            self.enter_structural(CAC, "TaxScheme")?;
            self.derived(CBC, "ID")?;
            self.leave_structural()?;
            self.leave_structural()?;
            self.leave_structural()?;
            if category == VatCategory::Exempt {
                exemptions.set(code, text);
            }
        }
        self.leave_structural()?;
        Ok(exemptions)
    }

    // Parses the accounting-currency tax total, returning its amount (`BT-111`).
    fn parse_accounting_tax_total(&mut self) -> Result<Decimal, Error> {
        self.enter_structural(CAC, "TaxTotal")?;
        let value = parse_decimal(&self.derived(CBC, "TaxAmount")?.1)?;
        self.leave_structural()?;
        Ok(value)
    }

    // Walks the legal monetary total, keeping the paid (`BT-113`) and rounding (`BT-114`)
    // amounts and discarding the derived totals.
    fn parse_legal_monetary_total(&mut self) -> Result<(Option<Decimal>, Option<Decimal>), Error> {
        self.enter_structural(CAC, "LegalMonetaryTotal")?;
        let mut paid = None;
        let mut rounding = None;
        while self.is_open_namespace(CBC) {
            let (_, name) = self.head()?;
            let (_, text) = self.derived(CBC, &name)?;
            match name.as_str() {
                "PrepaidAmount" => paid = Some(parse_decimal(&text)?),
                "PayableRoundingAmount" => rounding = Some(parse_decimal(&text)?),
                _ => {}
            }
        }
        self.leave_structural()?;
        Ok((paid, rounding))
    }

    // Parses one invoice line.
    fn parse_line(&mut self, instance: NonZeroUsize) -> Result<InvoiceLine, Error> {
        self.enter_repeatable(CAC, "InvoiceLine", "lines", instance)?;
        let id = self.leaf(CBC, "ID", "id")?.parse()?;
        let note = self.optional_leaf(CBC, "Note", "note")?;
        let (quantity_attrs, quantity_text) =
            self.leaf_attr(CBC, "InvoicedQuantity", "quantity")?;
        let quantity = parse_quantity(&quantity_attrs, &quantity_text)?;
        self.derived(CBC, "LineExtensionAmount")?;
        let buyer_accounting_reference =
            self.optional_leaf(CBC, "AccountingCost", "buyer_accounting_reference")?;
        let period = if self.is_open(CAC, "InvoicePeriod") {
            self.enter_group(CAC, "InvoicePeriod", "period")?;
            let start = self.optional_date(CBC, "StartDate", "period")?;
            let end = self.optional_date(CBC, "EndDate", "period")?;
            self.leave_group()?;
            period_from(start, end)
        } else {
            None
        };
        let order_line_reference = if self.is_open(CAC, "OrderLineReference") {
            self.enter_structural(CAC, "OrderLineReference")?;
            let reference = self.leaf(CBC, "LineID", "order_line_reference")?.parse()?;
            self.leave_structural()?;
            Some(reference)
        } else {
            None
        };
        let object = if self.is_open(CAC, "DocumentReference") {
            self.enter_structural(CAC, "DocumentReference")?;
            let (attributes, id) = self.leaf_attr(CBC, "ID", "object")?;
            let scheme = match attr(&attributes, "schemeID") {
                Some(value) => Some(value.parse()?),
                None => None,
            };
            self.leave_structural()?;
            Some(ObjectReference {
                id: id.parse()?,
                scheme,
            })
        } else {
            None
        };
        let mut adjustments = Vec::new();
        while self.is_open(CAC, "AllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_line_adjustment(instance)?);
        }
        let (item, vat) = self.parse_item()?;
        let price = self.parse_price()?;
        self.leave_repeatable()?;
        Ok(InvoiceLine {
            id,
            note,
            object,
            quantity,
            order_line_reference,
            buyer_accounting_reference,
            period,
            adjustments,
            price,
            vat,
            item,
        })
    }

    // Parses one line-level allowance or charge.
    fn parse_line_adjustment(&mut self, instance: NonZeroUsize) -> Result<LineAdjustment, Error> {
        self.enter_repeatable(CAC, "AllowanceCharge", "adjustments", instance)?;
        let charge = self.derived(CBC, "ChargeIndicator")?.1.trim() == "true";
        let reason = self.parse_reason(charge)?;
        let amount = self.parse_adjustment_amount()?;
        self.leave_repeatable()?;
        Ok(LineAdjustment { amount, reason })
    }

    // Parses the item, whose classified tax category yields the line VAT.
    fn parse_item(&mut self) -> Result<(Item, VatTreatment), Error> {
        self.take_open(CAC, "Item")?;
        self.trace.enter(CAC, "Item");
        self.trace.push_field("item");
        self.trace.record_context();

        let description = self.optional_leaf(CBC, "Description", "description")?;
        let name = self.leaf(CBC, "Name", "name")?.parse()?;
        let buyer_id = self.optional_identifier(CAC, "BuyersItemIdentification", "buyer_id")?;
        let seller_id = self.optional_identifier(CAC, "SellersItemIdentification", "seller_id")?;
        let standard_id = if self.is_open(CAC, "StandardItemIdentification") {
            self.enter_structural(CAC, "StandardItemIdentification")?;
            let (attributes, id) = self.leaf_attr(CBC, "ID", "standard_id")?;
            let issuer = attr(&attributes, "schemeID")
                .ok_or_else(|| bad("a standard item id without a scheme"))?
                .parse()?;
            self.leave_structural()?;
            Some(crate::ItemReference {
                id: id.parse()?,
                issuer,
            })
        } else {
            None
        };
        let country_of_origin = if self.is_open(CAC, "OriginCountry") {
            self.enter_structural(CAC, "OriginCountry")?;
            let code = self.leaf(CBC, "IdentificationCode", "country_of_origin")?;
            self.leave_structural()?;
            Some(parse_country(&code)?)
        } else {
            None
        };
        let mut classifications = Vec::new();
        while self.is_open(CAC, "CommodityClassification") {
            self.enter_structural(CAC, "CommodityClassification")?;
            let (attributes, id) =
                self.leaf_attr(CBC, "ItemClassificationCode", "classifications")?;
            let scheme = attr(&attributes, "listID")
                .ok_or_else(|| bad("a classification without a scheme"))?
                .parse()?;
            let version = match attr(&attributes, "listVersionID") {
                Some(value) => Some(value.parse()?),
                None => None,
            };
            self.leave_structural()?;
            classifications.push(Classification {
                id: id.parse()?,
                scheme,
                version,
            });
        }

        // The classified tax category is a sibling of the item in the model.
        self.trace.pop_context();
        let vat = self.parse_classified_tax_category()?;

        let mut attributes = Vec::new();
        while self.is_open(CAC, "AdditionalItemProperty") {
            self.enter_structural(CAC, "AdditionalItemProperty")?;
            let name = self.leaf(CBC, "Name", "attributes")?.parse()?;
            let value = self.leaf(CBC, "Value", "attributes")?.parse()?;
            self.leave_structural()?;
            attributes.push(ItemAttribute { name, value });
        }

        self.take_close()?;
        self.trace.leave();

        Ok((
            Item {
                name,
                description,
                seller_id,
                buyer_id,
                standard_id,
                classifications,
                country_of_origin,
                attributes,
            },
            vat,
        ))
    }

    // Parses the classified tax category into the line VAT treatment.
    fn parse_classified_tax_category(&mut self) -> Result<VatTreatment, Error> {
        self.enter_group(CAC, "ClassifiedTaxCategory", "vat")?;
        let category: VatCategory = self.leaf(CBC, "ID", "vat")?.parse()?;
        let rate = self.leaf(CBC, "Percent", "vat")?.parse()?;
        self.enter_structural(CAC, "TaxScheme")?;
        self.field_or_leaf(CBC, "ID", "vat")?;
        self.leave_structural()?;
        self.leave_group()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    // Parses the line price.
    fn parse_price(&mut self) -> Result<Price, Error> {
        self.enter_group(CAC, "Price", "price")?;
        let net = parse_decimal(&self.derived(CBC, "PriceAmount")?.1)?;
        let base_quantity = if self.is_open(CBC, "BaseQuantity") {
            let (attributes, text) = self.leaf_attr(CBC, "BaseQuantity", "price")?;
            Some(parse_quantity(&attributes, &text)?)
        } else {
            None
        };
        let (gross, discount) = if self.is_open(CAC, "AllowanceCharge") {
            self.enter_structural(CAC, "AllowanceCharge")?;
            self.field_or_leaf(CBC, "ChargeIndicator", "price")?;
            let amount = parse_decimal(&self.leaf(CBC, "Amount", "price")?)?;
            let base = parse_decimal(&self.leaf(CBC, "BaseAmount", "price")?)?;
            self.leave_structural()?;
            (base, Some(amount))
        } else {
            (net, None)
        };
        self.leave_group()?;
        Ok(Price {
            gross,
            discount,
            base_quantity,
        })
    }

    // ---- reusable party fragments ---------------------------------------

    fn optional_endpoint(
        &mut self,
        field: &'static str,
    ) -> Result<Option<ElectronicAddress>, Error> {
        if !self.is_open(CBC, "EndpointID") {
            return Ok(None);
        }
        let (attributes, id) = self.leaf_attr(CBC, "EndpointID", field)?;
        let scheme = attr(&attributes, "schemeID")
            .ok_or_else(|| bad("an endpoint without a scheme"))?
            .parse()?;
        Ok(Some(ElectronicAddress {
            id: id.parse()?,
            scheme,
        }))
    }

    fn parse_identifiers(&mut self, field: &'static str) -> Result<Vec<OperationalEntity>, Error> {
        let mut identifiers = Vec::new();
        while self.is_open(CAC, "PartyIdentification") {
            self.enter_group(CAC, "PartyIdentification", field)?;
            let (attributes, id) = self.leaf_attr(CBC, "ID", field)?;
            let issuer = match attr(&attributes, "schemeID") {
                Some(value) => Some(value.parse()?),
                None => None,
            };
            self.leave_group()?;
            identifiers.push(OperationalEntity {
                id: id.parse()?,
                issuer,
            });
        }
        Ok(identifiers)
    }

    fn optional_party_name(
        &mut self,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if self.is_open(CAC, "PartyName") {
            Ok(Some(self.parse_party_name(field)?))
        } else {
            Ok(None)
        }
    }

    fn parse_party_name(&mut self, field: &'static str) -> Result<NonEmptyString, Error> {
        self.enter_group(CAC, "PartyName", field)?;
        let name = self.leaf(CBC, "Name", field)?.parse()?;
        self.leave_group()?;
        Ok(name)
    }

    fn parse_party_tax_scheme(&mut self) -> Result<PartyTaxScheme, Error> {
        // The serializer chose the context field from the scheme, so peek it first.
        let field: &'static str = if self.peek_tax_scheme() == "VAT" {
            "vat"
        } else {
            "tax_registration"
        };
        self.enter_group(CAC, "PartyTaxScheme", field)?;
        let company = self.leaf(CBC, "CompanyID", field)?;
        self.enter_structural(CAC, "TaxScheme")?;
        let scheme = self.leaf(CBC, "ID", field)?;
        self.leave_structural()?;
        self.leave_group()?;
        if scheme == "VAT" {
            Ok(PartyTaxScheme::Vat(company.parse()?))
        } else {
            Ok(PartyTaxScheme::Other(company.parse()?))
        }
    }

    // Peeks the tax scheme code of the current party tax scheme, without consuming.
    fn peek_tax_scheme(&self) -> String {
        let mut cursor = self.cursor;
        let mut in_scheme = false;
        while let Some(token) = self.tokens.get(cursor) {
            match token {
                Token::Open { name, .. } if name == "TaxScheme" => in_scheme = true,
                Token::Open { name, .. } if in_scheme && name == "ID" => {
                    if let Some(Token::Text(text)) = self.tokens.get(cursor + 1) {
                        return text.trim().to_owned();
                    }
                }
                _ => {}
            }
            cursor += 1;
        }
        String::new()
    }

    fn parse_legal_entity(
        &mut self,
        name_field: &'static str,
    ) -> Result<(NonEmptyString, Option<LegalEntity>, Option<NonEmptyString>), Error> {
        self.enter_structural(CAC, "PartyLegalEntity")?;
        let name = self.leaf(CBC, "RegistrationName", name_field)?.parse()?;
        let legal_entity = if self.is_open(CBC, "CompanyID") {
            Some(self.parse_company_id("legal_entity")?)
        } else {
            None
        };
        let legal_form =
            self.optional_leaf(CBC, "CompanyLegalForm", "additional_legal_information")?;
        self.leave_structural()?;
        Ok((name, legal_entity, legal_form))
    }

    fn parse_company_id(&mut self, field: &'static str) -> Result<LegalEntity, Error> {
        let (attributes, id) = self.leaf_attr(CBC, "CompanyID", field)?;
        let issuer = match attr(&attributes, "schemeID") {
            Some(value) => Some(value.parse()?),
            None => None,
        };
        Ok(LegalEntity {
            id: id.parse()?,
            issuer,
        })
    }

    fn optional_contact(&mut self, field: &'static str) -> Result<Option<Contact>, Error> {
        if !self.is_open(CAC, "Contact") {
            return Ok(None);
        }
        self.enter_group(CAC, "Contact", field)?;
        let name = self.optional_leaf(CBC, "Name", "name")?;
        let telephone = self.optional_leaf(CBC, "Telephone", "telephone")?;
        let email = if self.is_open(CBC, "ElectronicMail") {
            Some(parse_email(&self.leaf(CBC, "ElectronicMail", "email")?)?)
        } else {
            None
        };
        self.leave_group()?;
        Ok(Some(Contact {
            name,
            telephone,
            email,
        }))
    }

    fn parse_address(
        &mut self,
        namespace: Namespace,
        element: &str,
    ) -> Result<PostalAddress, Error> {
        self.enter_group(namespace, element, "address")?;
        let line1 = self.optional_leaf(CBC, "StreetName", "line1")?;
        let line2 = self.optional_leaf(CBC, "AdditionalStreetName", "line2")?;
        let city = self.optional_leaf(CBC, "CityName", "city")?;
        let postal_code = self.optional_leaf(CBC, "PostalZone", "postal_code")?;
        let country_subdivision =
            self.optional_leaf(CBC, "CountrySubentity", "country_subdivision")?;
        let line3 = if self.is_open(CAC, "AddressLine") {
            self.enter_structural(CAC, "AddressLine")?;
            let line = self.leaf(CBC, "Line", "line3")?.parse()?;
            self.leave_structural()?;
            Some(line)
        } else {
            None
        };
        self.enter_structural(CAC, "Country")?;
        let country = parse_country(&self.leaf(CBC, "IdentificationCode", "country")?)?;
        self.leave_structural()?;
        self.leave_group()?;
        Ok(PostalAddress {
            line1,
            line2,
            line3,
            city,
            country,
            country_subdivision,
            postal_code,
        })
    }

    fn optional_identifier(
        &mut self,
        namespace: Namespace,
        element: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if !self.is_open(namespace, element) {
            return Ok(None);
        }
        self.enter_structural(namespace, element)?;
        let id = self.leaf(CBC, "ID", field)?.parse()?;
        self.leave_structural()?;
        Ok(Some(id))
    }

    // ---- mirror primitives ----------------------------------------------

    // Reads a leaf mapped to a model field, returning its text.
    fn leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        Ok(self.leaf_attr(namespace, name, field)?.1)
    }

    // Reads a leaf mapped to a model field, returning its attributes and text.
    fn leaf_attr(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<(Vec<(String, String)>, String), Error> {
        let attributes = self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_field(field);
        self.trace.record_context();
        let text = self.take_text();
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok((attributes, text))
    }

    // Reads a leaf that maps to a model field when present, ignoring the value otherwise.
    fn field_or_leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        self.leaf(namespace, name, field)
    }

    // Reads an optional leaf mapped to a model field.
    fn optional_leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf(namespace, name, field)?.parse()?))
        } else {
            Ok(None)
        }
    }

    // Reads an optional date leaf mapped to a model field.
    fn optional_date(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<Option<Date>, Error> {
        if self.is_open(namespace, name) {
            Ok(Some(parse_date(&self.leaf(namespace, name, field)?)?))
        } else {
            Ok(None)
        }
    }

    // Reads a single-identifier reference group when present.
    fn optional_reference(
        &mut self,
        namespace: Namespace,
        element: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if !self.is_open(namespace, element) {
            return Ok(None);
        }
        self.enter_group(namespace, element, field)?;
        let id = self.leaf(CBC, "ID", field)?.parse()?;
        self.leave_group()?;
        Ok(Some(id))
    }

    // Reads a repeatable single-value element.
    fn repeatable_leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        instance: NonZeroUsize,
    ) -> Result<String, Error> {
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_instance(field, instance);
        self.trace.record_context();
        let text = self.take_text();
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok(text)
    }

    // Reads a regulatory leaf recorded at the root context.
    fn rooted(&mut self, namespace: Namespace, name: &str) -> Result<String, Error> {
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_root();
        let text = self.take_text();
        self.take_close()?;
        self.trace.leave();
        Ok(text)
    }

    // Reads a derived leaf with no model field, returning its attributes and text.
    fn derived(
        &mut self,
        namespace: Namespace,
        name: &str,
    ) -> Result<(Vec<(String, String)>, String), Error> {
        let attributes = self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_context();
        let text = self.take_text();
        self.take_close()?;
        self.trace.leave();
        Ok((attributes, text))
    }

    fn enter_structural(&mut self, namespace: Namespace, name: &str) -> Result<(), Error> {
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_root();
        Ok(())
    }

    fn leave_structural(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.leave();
        Ok(())
    }

    fn enter_nested(&mut self, namespace: Namespace, name: &str) -> Result<(), Error> {
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_context();
        Ok(())
    }

    fn leave_nested(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.leave();
        Ok(())
    }

    fn enter_group(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<(), Error> {
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_field(field);
        self.trace.record_context();
        Ok(())
    }

    fn leave_group(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok(())
    }

    fn enter_repeatable(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        instance: NonZeroUsize,
    ) -> Result<(), Error> {
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_instance(field, instance);
        self.trace.record_context();
        Ok(())
    }

    fn leave_repeatable(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok(())
    }

    // ---- token cursor ---------------------------------------------------

    // Whether the current open element belongs to the invoiced object.
    fn additional_is_object(&self) -> bool {
        // Scan the children of the current AdditionalDocumentReference for a
        // DocumentTypeCode of 130, without consuming any token.
        let mut depth = 0usize;
        let mut cursor = self.cursor;
        while cursor < self.tokens.len() {
            match &self.tokens[cursor] {
                Token::Open { name, .. } => {
                    if depth == 1 && name == "DocumentTypeCode" {
                        if let Some(Token::Text(value)) = self.tokens.get(cursor + 1) {
                            return value.trim() == "130";
                        }
                    }
                    depth += 1;
                }
                Token::Close => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Token::Text(_) => {}
            }
            cursor += 1;
        }
        false
    }

    fn head(&self) -> Result<(Namespace, String), Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace, name, ..
            }) => Ok((*namespace, name.clone())),
            _ => Err(bad("expected an element")),
        }
    }

    fn is_open(&self, namespace: Namespace, name: &str) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, name: local, .. }) if *found == namespace && local == name
        )
    }

    fn is_open_namespace(&self, namespace: Namespace) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, .. }) if *found == namespace
        )
    }

    fn take_open(
        &mut self,
        namespace: Namespace,
        name: &str,
    ) -> Result<Vec<(String, String)>, Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace: found,
                name: local,
                attributes,
            }) if *found == namespace && local == name => {
                let attributes = attributes.clone();
                self.cursor += 1;
                Ok(attributes)
            }
            _ => Err(Error::malformed_xml(format!("expected <{name}>"))),
        }
    }

    fn take_close(&mut self) -> Result<(), Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Close) => {
                self.cursor += 1;
                Ok(())
            }
            _ => Err(bad("expected an end tag")),
        }
    }

    fn take_text(&mut self) -> String {
        if let Some(Token::Text(text)) = self.tokens.get(self.cursor) {
            let text = text.clone();
            self.cursor += 1;
            text
        } else {
            String::new()
        }
    }
}

// ---- helpers -------------------------------------------------------------

enum AdditionalDocument {
    Object(ObjectReference),
    Supporting(SupportingDocument),
}

enum PartyTaxScheme {
    Vat(crate::VatIdentifier),
    Other(NonEmptyString),
}

// The exemption reason of the single exempt VAT group, applied back to the model.
struct ExemptionMap {
    exempt: Option<(Option<ExemptionReason>, Option<NonEmptyString>)>,
}

impl ExemptionMap {
    fn new() -> Self {
        Self { exempt: None }
    }

    fn set(&mut self, code: Option<ExemptionReason>, text: Option<NonEmptyString>) {
        self.exempt = Some((code, text));
    }

    // Fills the exemption reason into every exempt line and adjustment VAT.
    fn apply(&self, invoice: &mut Invoice) {
        let Some((code, text)) = &self.exempt else {
            return;
        };
        let fill = |vat: &mut VatTreatment| {
            if let VatTreatment::Exempt {
                code: slot_code,
                text: slot_text,
            } = vat
            {
                *slot_code = *code;
                *slot_text = text.clone();
            }
        };
        for line in &mut invoice.lines {
            fill(&mut line.vat);
        }
        for adjustment in &mut invoice.adjustments {
            fill(&mut adjustment.vat);
        }
    }
}

fn index(position: usize) -> NonZeroUsize {
    NonZeroUsize::new(position + 1).expect("a positive index")
}

fn attr<'a>(attributes: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

fn period_from(start: Option<Date>, end: Option<Date>) -> Option<Period> {
    match (start, end) {
        (Some(start), Some(end)) => Some(Period::Range { start, end }),
        (Some(start), None) => Some(Period::From(start)),
        (None, Some(end)) => Some(Period::Until(end)),
        (None, None) => None,
    }
}

fn parse_quantity(attributes: &[(String, String)], text: &str) -> Result<Quantity, Error> {
    let code = attr(attributes, "unitCode").ok_or_else(|| bad("a quantity without a unit"))?;
    let unit = Unit::from_code(code)
        .ok_or_else(|| Error::malformed_xml(format!("invalid unit: {code}")))?;
    Ok(Quantity {
        unit,
        value: parse_decimal(text)?,
    })
}

fn parse_note(value: &str) -> Result<Note, Error> {
    if let Some(rest) = value.strip_prefix('#') {
        if let Some((code, text)) = rest.split_once('#') {
            return Ok(Note {
                subject_code: Some(code.parse()?),
                text: text.parse()?,
            });
        }
    }
    Ok(Note {
        subject_code: None,
        text: value.parse()?,
    })
}

fn parse_date(value: &str) -> Result<Date, Error> {
    let mut parts = value.split('-');
    let year = parts.next().and_then(|part| part.parse::<i32>().ok());
    let month = parts
        .next()
        .and_then(|part| part.parse::<u8>().ok())
        .and_then(|month| Month::try_from(month).ok());
    let day = parts.next().and_then(|part| part.parse::<u8>().ok());
    match (year, month, day) {
        (Some(year), Some(month), Some(day)) => {
            Date::from_calendar_date(year, month, day).map_err(|error| {
                Error::malformed_xml(format!("invalid date: {value}")).caused_by(error)
            })
        }
        _ => Err(Error::malformed_xml(format!("invalid date: {value}"))),
    }
}

fn parse_decimal(value: &str) -> Result<Decimal, Error> {
    value
        .trim()
        .parse::<Decimal>()
        .map_err(|_| Error::malformed_xml(format!("invalid decimal: {value}")))
}

fn parse_currency(value: &str) -> Result<Currency, Error> {
    Currency::from_code(value.trim())
        .ok_or_else(|| Error::malformed_xml(format!("invalid currency: {value}")))
}

fn parse_country(value: &str) -> Result<CountryCode, Error> {
    CountryCode::for_alpha2(value.trim())
        .map_err(|error| Error::malformed_xml(format!("invalid country: {value}")).caused_by(error))
}

fn parse_email(value: &str) -> Result<EmailAddress, Error> {
    value
        .trim()
        .parse::<EmailAddress>()
        .map_err(|error| Error::malformed_xml(format!("invalid email: {value}")).caused_by(error))
}

fn parse_url(value: &str) -> Result<Url, Error> {
    value
        .trim()
        .parse::<Url>()
        .map_err(|error| Error::malformed_xml(format!("invalid url: {value}")).caused_by(error))
}

fn bad(message: &str) -> Error {
    Error::malformed_xml(message)
}
