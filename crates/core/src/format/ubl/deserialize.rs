use crate::format::Token;
use crate::format::trace::Trace;
use crate::format::ubl;
use crate::prelude::*;
use crate::{
    Adjustment, AdjustmentAmount, AdjustmentReason, Amount, BinaryObject, Buyer, Classification,
    Contact, CreditTransfer, Delivery, Deserializable, DirectDebit, Document, DocumentBuilder,
    ElectronicAddress, Error, ExemptionReason, Format, Invoice, InvoiceLine, Item, ItemAttribute,
    LegalEntity, LineAdjustment, LocationReference, MimeCode, NonEmptyString, Note,
    ObjectReference, OperationalEntity, Payee, PaymentCard, PaymentDetails, PaymentInstructions,
    Percentage, Period, PostalAddress, PrecedingInvoice, Price, Quantity, Seller,
    SupportingDocument, TaxRepresentative, Ubl, Unit, VatBreakdown, VatCategory, VatPoint,
    VatTreatment,
};

impl Deserializable<Ubl> for Invoice {
    fn deserialize(document: &mut Document<Self, Ubl>) -> Result<(), Error> {
        let (tokens, abbreviations) = Ubl::tokenize(&document.xml)?;
        let mut parser = Parser {
            tokens,
            cursor: 0,
            trace: Trace::new(),
        };
        document.builder = parser.document()?;
        document.dictionary = parser.trace.into_dictionary();
        document.abbreviations = abbreviations;
        Ok(())
    }
}

// ---- parser --------------------------------------------------------------

struct Parser {
    tokens: Vec<Token<ubl::Namespace>>,
    cursor: usize,
    trace: Trace<ubl::Namespace>,
}

impl Parser {
    fn document(&mut self) -> Result<DocumentBuilder<Invoice>, Error> {
        self.take_open(Ubl::root_namespace(), Ubl::ROOT_ELEMENT)?;
        self.trace.enter(Ubl::root_namespace(), Ubl::ROOT_ELEMENT);
        self.trace.record_root();

        let profile = self
            .rooted(ubl::Namespace::Cbc, "CustomizationID")?
            .parse()?;
        let business_process = if self.is_open(ubl::Namespace::Cbc, "ProfileID") {
            Some(self.rooted(ubl::Namespace::Cbc, "ProfileID")?.parse()?)
        } else {
            None
        };

        let number = self.optional_leaf(ubl::Namespace::Cbc, "ID", "number")?;
        let issue_date = self.optional_date(ubl::Namespace::Cbc, "IssueDate", "issue_date")?;
        let payment_due_date =
            self.optional_date(ubl::Namespace::Cbc, "DueDate", "payment_due_date")?;
        let type_code = self
            .leaf(ubl::Namespace::Cbc, "InvoiceTypeCode", "type_code")?
            .parse()?;

        let mut notes = Vec::new();
        while self.is_open(ubl::Namespace::Cbc, "Note") {
            let instance = index(notes.len());
            let text = self.repeatable_leaf(ubl::Namespace::Cbc, "Note", "notes", instance)?;
            notes.push(parse_note(&text)?);
        }

        let mut vat_point = None;
        if self.is_open(ubl::Namespace::Cbc, "TaxPointDate") {
            vat_point = Some(VatPoint::Date(parse_date(&self.leaf(
                ubl::Namespace::Cbc,
                "TaxPointDate",
                "vat_point",
            )?)?));
        }

        let currency = self
            .optional_text(ubl::Namespace::Cbc, "DocumentCurrencyCode", "currency")?
            .map(|code| parse_currency(&code))
            .transpose()?;
        let accounting_currency = if self.is_open(ubl::Namespace::Cbc, "TaxCurrencyCode") {
            Some(parse_currency(&self.leaf(
                ubl::Namespace::Cbc,
                "TaxCurrencyCode",
                "vat_accounting_total",
            )?)?)
        } else {
            None
        };

        let buyer_accounting_reference = self.optional_leaf(
            ubl::Namespace::Cbc,
            "AccountingCost",
            "buyer_accounting_reference",
        )?;
        let buyer_reference =
            self.optional_leaf(ubl::Namespace::Cbc, "BuyerReference", "buyer_reference")?;

        let mut invoicing_period = None;
        if self.is_open(ubl::Namespace::Cac, "InvoicePeriod") {
            let (period, event) = self.parse_invoice_period()?;
            invoicing_period = period;
            if let Some(point) = event {
                vat_point = Some(point);
            }
        }

        let (purchase_order_reference, sales_order_reference) =
            if self.is_open(ubl::Namespace::Cac, "OrderReference") {
                self.parse_order_reference()?
            } else {
                (None, None)
            };

        let mut preceding_invoices = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "BillingReference") {
            let instance = index(preceding_invoices.len());
            preceding_invoices.push(self.parse_billing_reference(instance)?);
        }

        let despatch_advice_reference = self.optional_reference(
            ubl::Namespace::Cac,
            "DespatchDocumentReference",
            "despatch_advice_reference",
        )?;
        let receiving_advice_reference = self.optional_reference(
            ubl::Namespace::Cac,
            "ReceiptDocumentReference",
            "receiving_advice_reference",
        )?;
        let tender_or_lot_reference = self.optional_reference(
            ubl::Namespace::Cac,
            "OriginatorDocumentReference",
            "tender_or_lot_reference",
        )?;
        let contract_reference = self.optional_reference(
            ubl::Namespace::Cac,
            "ContractDocumentReference",
            "contract_reference",
        )?;

        let mut object = None;
        let mut supporting_documents = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "AdditionalDocumentReference") {
            match self.parse_additional_document(supporting_documents.len())? {
                AdditionalDocument::Object(reference) => object = Some(reference),
                AdditionalDocument::Supporting(document) => supporting_documents.push(document),
            }
        }

        let project_reference =
            self.optional_reference(ubl::Namespace::Cac, "ProjectReference", "project_reference")?;

        let seller = if self.is_open(ubl::Namespace::Cac, "AccountingSupplierParty") {
            Some(self.parse_supplier_party()?)
        } else {
            None
        };
        let buyer = if self.is_open(ubl::Namespace::Cac, "AccountingCustomerParty") {
            Some(self.parse_customer_party()?)
        } else {
            None
        };
        let payee = if self.is_open(ubl::Namespace::Cac, "PayeeParty") {
            Some(self.parse_payee_party()?)
        } else {
            None
        };
        let tax_representative = if self.is_open(ubl::Namespace::Cac, "TaxRepresentativeParty") {
            Some(self.parse_tax_representative_party()?)
        } else {
            None
        };
        let delivery = if self.is_open(ubl::Namespace::Cac, "Delivery") {
            Some(self.parse_delivery()?)
        } else {
            None
        };
        let payment = if self.is_open(ubl::Namespace::Cac, "PaymentMeans") {
            Some(self.parse_payment_means()?)
        } else {
            None
        };
        let payment_terms = if self.is_open(ubl::Namespace::Cac, "PaymentTerms") {
            self.parse_payment_terms()?
        } else {
            None
        };

        let mut adjustments = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "AllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_adjustment(instance)?);
        }

        let mut tax_total = TaxTotal {
            exemptions: ExemptionMap::new(),
            vat_total: None,
            breakdown: Vec::new(),
        };
        let mut accounting_value = None;
        if self.is_open(ubl::Namespace::Cac, "TaxTotal") {
            tax_total = self.parse_tax_total()?;
        }
        if self.is_open(ubl::Namespace::Cac, "TaxTotal") {
            accounting_value = self.parse_accounting_tax_total()?;
        }
        let monetary_total = if self.is_open(ubl::Namespace::Cac, "LegalMonetaryTotal") {
            self.parse_legal_monetary_total()?
        } else {
            MonetaryTotal::default()
        };

        let mut lines = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "InvoiceLine") {
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
            line_net_total: monetary_total.line_net_total,
            allowances_total: monetary_total.allowances_total,
            charges_total: monetary_total.charges_total,
            net_total: monetary_total.net_total,
            vat_total: tax_total.vat_total,
            gross_total: monetary_total.gross_total,
            rounding: monetary_total.rounding,
            payment,
            paid: monetary_total.paid,
            due: monetary_total.due,
            vat_breakdown: tax_total.breakdown,
            supporting_documents,
            lines,
        };
        tax_total.exemptions.apply(&mut invoice);

        Ok(DocumentBuilder {
            invoice,
            profile,
            business_process,
        })
    }

    // Parses the invoice period, returning the period and the VAT point event.
    fn parse_invoice_period(&mut self) -> Result<(Option<Period>, Option<VatPoint>), Error> {
        self.enter_group(ubl::Namespace::Cac, "InvoicePeriod", "invoicing_period")?;
        let start = self.optional_date(ubl::Namespace::Cbc, "StartDate", "invoicing_period")?;
        let end = self.optional_date(ubl::Namespace::Cbc, "EndDate", "invoicing_period")?;
        let event = if self.is_open(ubl::Namespace::Cbc, "DescriptionCode") {
            Some(VatPoint::Event(
                self.leaf(ubl::Namespace::Cbc, "DescriptionCode", "vat_point")?
                    .parse()?,
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
        self.enter_structural(ubl::Namespace::Cac, "OrderReference")?;
        let order = self.optional_leaf(ubl::Namespace::Cbc, "ID", "purchase_order_reference")?;
        let sales =
            self.optional_leaf(ubl::Namespace::Cbc, "SalesOrderID", "sales_order_reference")?;
        self.leave_structural()?;
        Ok((order, sales))
    }

    // Parses one preceding invoice reference.
    fn parse_billing_reference(
        &mut self,
        instance: NonZeroUsize,
    ) -> Result<PrecedingInvoice, Error> {
        self.enter_repeatable(
            ubl::Namespace::Cac,
            "BillingReference",
            "preceding_invoices",
            instance,
        )?;
        self.enter_structural(ubl::Namespace::Cac, "InvoiceDocumentReference")?;
        let number = self.optional_leaf(ubl::Namespace::Cbc, "ID", "number")?;
        let issue_date = self.optional_date(ubl::Namespace::Cbc, "IssueDate", "issue_date")?;
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
            self.enter_group(ubl::Namespace::Cac, "AdditionalDocumentReference", "object")?;
            let mut id = None;
            let mut scheme = None;
            if let Some((attributes, value)) = self.optional_derived(ubl::Namespace::Cbc, "ID")? {
                id = Some(value.parse()?);
                scheme = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.derived(ubl::Namespace::Cbc, "DocumentTypeCode")?;
            self.leave_group()?;
            Ok(AdditionalDocument::Object(ObjectReference { id, scheme }))
        } else {
            let instance = index(supporting);
            self.enter_repeatable(
                ubl::Namespace::Cac,
                "AdditionalDocumentReference",
                "supporting_documents",
                instance,
            )?;
            let reference = self.optional_leaf(ubl::Namespace::Cbc, "ID", "reference")?;
            let description =
                self.optional_leaf(ubl::Namespace::Cbc, "DocumentDescription", "description")?;
            let mut external_location = None;
            let mut attachment = None;
            if self.is_open(ubl::Namespace::Cac, "Attachment") {
                self.enter_structural(ubl::Namespace::Cac, "Attachment")?;
                if self.is_open(ubl::Namespace::Cac, "ExternalReference") {
                    self.enter_structural(ubl::Namespace::Cac, "ExternalReference")?;
                    external_location = self
                        .optional_text(ubl::Namespace::Cbc, "URI", "external_location")?
                        .map(|uri| parse_url(&uri))
                        .transpose()?;
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
        let (attributes, encoded) =
            self.derived(ubl::Namespace::Cbc, "EmbeddedDocumentBinaryObject")?;
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
        self.enter_structural(ubl::Namespace::Cac, "AccountingSupplierParty")?;
        self.enter_group(ubl::Namespace::Cac, "Party", "seller")?;
        let electronic_address = self.optional_endpoint("electronic_address")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let trading_name = self.optional_party_name("trading_name")?;
        let address = self.optional_address(ubl::Namespace::Cac, "PostalAddress")?;
        let mut vat = None;
        let mut tax_registration = None;
        while self.is_open(ubl::Namespace::Cac, "PartyTaxScheme") {
            match self.parse_party_tax_scheme()? {
                Some(PartyTaxScheme::Vat(value)) => vat = Some(value),
                Some(PartyTaxScheme::Other(value)) => tax_registration = Some(value),
                None => {}
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
        self.enter_structural(ubl::Namespace::Cac, "AccountingCustomerParty")?;
        self.enter_group(ubl::Namespace::Cac, "Party", "buyer")?;
        let electronic_address = self.optional_endpoint("electronic_address")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let trading_name = self.optional_party_name("trading_name")?;
        let address = self.optional_address(ubl::Namespace::Cac, "PostalAddress")?;
        let mut vat = None;
        while self.is_open(ubl::Namespace::Cac, "PartyTaxScheme") {
            if let Some(PartyTaxScheme::Vat(value)) = self.parse_party_tax_scheme()? {
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
        self.enter_group(ubl::Namespace::Cac, "PayeeParty", "payee")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.optional_party_name("name")?;
        let legal_entity = if self.is_open(ubl::Namespace::Cac, "PartyLegalEntity") {
            self.enter_structural(ubl::Namespace::Cac, "PartyLegalEntity")?;
            let entity = self.optional_company_id("legal_entity")?;
            self.leave_structural()?;
            entity
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
        self.enter_group(
            ubl::Namespace::Cac,
            "TaxRepresentativeParty",
            "tax_representative",
        )?;
        let name = self.optional_party_name("name")?;
        let address = self.optional_address(ubl::Namespace::Cac, "PostalAddress")?;
        let vat = if self.is_open(ubl::Namespace::Cac, "PartyTaxScheme") {
            match self.parse_party_tax_scheme()? {
                Some(PartyTaxScheme::Vat(value)) => Some(value),
                Some(PartyTaxScheme::Other(_)) => {
                    return Err(bad("a tax representative without a VAT scheme"));
                }
                None => None,
            }
        } else {
            None
        };
        self.leave_group()?;
        Ok(TaxRepresentative { name, vat, address })
    }

    // Parses the delivery information.
    fn parse_delivery(&mut self) -> Result<Delivery, Error> {
        self.enter_group(ubl::Namespace::Cac, "Delivery", "delivery")?;
        let date = self.optional_date(ubl::Namespace::Cbc, "ActualDeliveryDate", "date")?;
        let mut location = None;
        let mut address = None;
        if self.is_open(ubl::Namespace::Cac, "DeliveryLocation") {
            self.enter_structural(ubl::Namespace::Cac, "DeliveryLocation")?;
            if self.is_open(ubl::Namespace::Cbc, "ID") {
                let (attributes, id) = self.leaf_attr(ubl::Namespace::Cbc, "ID", "location")?;
                let issuer = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
                location = Some(LocationReference {
                    id: Some(id.parse()?),
                    issuer,
                });
            }
            address = self.optional_address(ubl::Namespace::Cac, "Address")?;
            self.leave_structural()?;
        }
        let name = if self.is_open(ubl::Namespace::Cac, "DeliveryParty") {
            self.enter_structural(ubl::Namespace::Cac, "DeliveryParty")?;
            let name = self.optional_party_name("name")?;
            self.leave_structural()?;
            name
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
        self.enter_group(ubl::Namespace::Cac, "PaymentMeans", "payment")?;
        let mut means = None;
        let mut means_text = None;
        if let Some((attributes, code)) =
            self.optional_leaf_attr(ubl::Namespace::Cbc, "PaymentMeansCode", "means")?
        {
            means = Some(code.parse()?);
            means_text = match attr(&attributes, "name") {
                Some(value) => Some(value.parse()?),
                None => None,
            };
        }
        let remittance_information =
            self.optional_leaf(ubl::Namespace::Cbc, "PaymentID", "remittance_information")?;
        let details = if self.is_open(ubl::Namespace::Cac, "PayeeFinancialAccount") {
            let mut transfers = Vec::new();
            while self.is_open(ubl::Namespace::Cac, "PayeeFinancialAccount") {
                transfers.push(self.parse_credit_transfer()?);
            }
            Some(PaymentDetails::CreditTransfers(transfers))
        } else if self.is_open(ubl::Namespace::Cac, "CardAccount") {
            Some(PaymentDetails::Card(self.parse_card()?))
        } else if self.is_open(ubl::Namespace::Cac, "PaymentMandate") {
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
        self.enter_structural(ubl::Namespace::Cac, "PayeeFinancialAccount")?;
        let account = self
            .optional_text(ubl::Namespace::Cbc, "ID", "account")?
            .map(|value| value.parse())
            .transpose()?;
        let account_name = self.optional_leaf(ubl::Namespace::Cbc, "Name", "account_name")?;
        let provider = if self.is_open(ubl::Namespace::Cac, "FinancialInstitutionBranch") {
            self.enter_structural(ubl::Namespace::Cac, "FinancialInstitutionBranch")?;
            let bic = self
                .optional_text(ubl::Namespace::Cbc, "ID", "provider")?
                .map(|value| value.parse())
                .transpose()?;
            self.leave_structural()?;
            bic
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
        self.enter_structural(ubl::Namespace::Cac, "CardAccount")?;
        let primary_account_number = self.optional_leaf(
            ubl::Namespace::Cbc,
            "PrimaryAccountNumberID",
            "primary_account_number",
        )?;
        let holder_name = self.optional_leaf(ubl::Namespace::Cbc, "HolderName", "holder_name")?;
        self.leave_structural()?;
        Ok(PaymentCard {
            primary_account_number,
            holder_name,
        })
    }

    fn parse_direct_debit(&mut self) -> Result<DirectDebit, Error> {
        self.enter_structural(ubl::Namespace::Cac, "PaymentMandate")?;
        let mandate_reference =
            self.optional_leaf(ubl::Namespace::Cbc, "ID", "mandate_reference")?;
        let creditor_identifier =
            self.optional_leaf(ubl::Namespace::Cbc, "PayerPartyID", "creditor_identifier")?;
        let debited_account = if self.is_open(ubl::Namespace::Cac, "PayerFinancialAccount") {
            self.enter_structural(ubl::Namespace::Cac, "PayerFinancialAccount")?;
            let account = self
                .optional_text(ubl::Namespace::Cbc, "ID", "debited_account")?
                .map(|value| value.parse())
                .transpose()?;
            self.leave_structural()?;
            account
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
    fn parse_payment_terms(&mut self) -> Result<Option<NonEmptyString>, Error> {
        self.enter_group(ubl::Namespace::Cac, "PaymentTerms", "payment_terms")?;
        let note = self.optional_leaf(ubl::Namespace::Cbc, "Note", "payment_terms")?;
        self.leave_group()?;
        Ok(note)
    }

    // Parses one document-level allowance or charge.
    fn parse_adjustment(&mut self, instance: NonZeroUsize) -> Result<Adjustment, Error> {
        self.enter_repeatable(
            ubl::Namespace::Cac,
            "AllowanceCharge",
            "adjustments",
            instance,
        )?;
        let charge = self.optional_charge_indicator()?;
        let reason = self.parse_reason(charge)?;
        let amount = self.parse_adjustment_amount()?;
        let vat = if self.is_open(ubl::Namespace::Cac, "TaxCategory") {
            Some(self.parse_tax_category()?)
        } else {
            None
        };
        self.leave_repeatable()?;
        Ok(Adjustment {
            amount,
            vat,
            reason,
        })
    }

    // Parses the charge indicator of an adjustment, when present.
    fn optional_charge_indicator(&mut self) -> Result<Option<bool>, Error> {
        Ok(self
            .optional_derived(ubl::Namespace::Cbc, "ChargeIndicator")?
            .map(|(_, text)| text.trim() == "true"))
    }

    // Parses the reason code and text of an adjustment, dropped without the direction.
    fn parse_reason(&mut self, charge: Option<bool>) -> Result<Option<AdjustmentReason>, Error> {
        let code = if self.is_open(ubl::Namespace::Cbc, "AllowanceChargeReasonCode") {
            Some(
                self.derived(ubl::Namespace::Cbc, "AllowanceChargeReasonCode")?
                    .1,
            )
        } else {
            None
        };
        let text = if self.is_open(ubl::Namespace::Cbc, "AllowanceChargeReason") {
            Some(
                self.derived(ubl::Namespace::Cbc, "AllowanceChargeReason")?
                    .1
                    .parse()?,
            )
        } else {
            None
        };
        Ok(match charge {
            Some(true) => Some(AdjustmentReason::Charge {
                code: match code {
                    Some(code) => Some(code.parse()?),
                    None => None,
                },
                text,
            }),
            Some(false) => Some(AdjustmentReason::Allowance {
                code: match code {
                    Some(code) => Some(code.parse()?),
                    None => None,
                },
                text,
            }),
            None => None,
        })
    }

    // Parses the amount of an adjustment, absolute or relative, absent without the amount.
    fn parse_adjustment_amount(&mut self) -> Result<Option<AdjustmentAmount>, Error> {
        let factor =
            self.optional_text(ubl::Namespace::Cbc, "MultiplierFactorNumeric", "amount")?;
        let amount = self
            .optional_text(ubl::Namespace::Cbc, "Amount", "amount")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        let base = self
            .optional_text(ubl::Namespace::Cbc, "BaseAmount", "amount")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        Ok(match (factor, amount, base) {
            (Some(factor), Some(amount), Some(base)) => Some(AdjustmentAmount::Relative {
                amount,
                rate: factor.parse()?,
                base,
            }),
            (None, Some(amount), _) => Some(AdjustmentAmount::Absolute(amount)),
            _ => None,
        })
    }

    // Parses an allowance or charge VAT category.
    fn parse_tax_category(&mut self) -> Result<VatTreatment, Error> {
        self.enter_nested(ubl::Namespace::Cac, "TaxCategory")?;
        let category: VatCategory = self.derived(ubl::Namespace::Cbc, "ID")?.1.parse()?;
        let rate = self.derived(ubl::Namespace::Cbc, "Percent")?.1.parse()?;
        self.enter_nested(ubl::Namespace::Cac, "TaxScheme")?;
        self.derived(ubl::Namespace::Cbc, "ID")?;
        self.leave_nested()?;
        self.leave_nested()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    // Parses the tax total: the VAT total (`BT-110`) and the breakdown (`BG-23`),
    // collecting exemption reasons per category.
    fn parse_tax_total(&mut self) -> Result<TaxTotal, Error> {
        let mut exemptions = ExemptionMap::new();
        let mut breakdown = Vec::new();
        self.enter_structural(ubl::Namespace::Cac, "TaxTotal")?;
        let vat_total = self
            .optional_text(ubl::Namespace::Cbc, "TaxAmount", "vat_total")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        while self.is_open(ubl::Namespace::Cac, "TaxSubtotal") {
            let instance = index(breakdown.len());
            self.enter_repeatable(
                ubl::Namespace::Cac,
                "TaxSubtotal",
                "vat_breakdown",
                instance,
            )?;
            let taxable = self
                .optional_text(ubl::Namespace::Cbc, "TaxableAmount", "taxable")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            let tax = self
                .optional_text(ubl::Namespace::Cbc, "TaxAmount", "tax")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            let treatment = if self.is_open(ubl::Namespace::Cac, "TaxCategory") {
                self.parse_breakdown_category()?
            } else {
                None
            };
            self.leave_repeatable()?;
            if let Some(VatTreatment::Exempt { code, text }) = &treatment {
                exemptions.set(*code, text.clone());
            }
            breakdown.push(VatBreakdown {
                treatment,
                taxable,
                tax,
            });
        }
        self.leave_structural()?;
        Ok(TaxTotal {
            exemptions,
            vat_total,
            breakdown,
        })
    }

    // Parses the VAT category of a breakdown group, absent without the category code.
    fn parse_breakdown_category(&mut self) -> Result<Option<VatTreatment>, Error> {
        self.enter_group(ubl::Namespace::Cac, "TaxCategory", "treatment")?;
        let category: Option<VatCategory> = self
            .optional_derived(ubl::Namespace::Cbc, "ID")?
            .map(|(_, text)| text.parse())
            .transpose()?;
        let rate: Option<Percentage> = self
            .optional_derived(ubl::Namespace::Cbc, "Percent")?
            .map(|(_, text)| text.parse())
            .transpose()?;
        let mut code = None;
        let mut text = None;
        if self.is_open(ubl::Namespace::Cbc, "TaxExemptionReasonCode") {
            code = Some(
                self.derived(ubl::Namespace::Cbc, "TaxExemptionReasonCode")?
                    .1
                    .parse()?,
            );
        }
        if self.is_open(ubl::Namespace::Cbc, "TaxExemptionReason") {
            text = Some(
                self.derived(ubl::Namespace::Cbc, "TaxExemptionReason")?
                    .1
                    .parse()?,
            );
        }
        if self.is_open(ubl::Namespace::Cac, "TaxScheme") {
            self.enter_nested(ubl::Namespace::Cac, "TaxScheme")?;
            self.optional_derived(ubl::Namespace::Cbc, "ID")?;
            self.leave_nested()?;
        }
        self.leave_group()?;
        let Some(category) = category else {
            return Ok(None);
        };
        let rate = match rate {
            Some(rate) => rate,
            None => Percentage::try_from(Decimal::ZERO)?,
        };
        Ok(Some(match VatTreatment::from_category(category, rate) {
            VatTreatment::Exempt { .. } => VatTreatment::Exempt { code, text },
            treatment => treatment,
        }))
    }

    // Parses the accounting-currency tax total, returning its amount (`BT-111`) when present.
    fn parse_accounting_tax_total(&mut self) -> Result<Option<Decimal>, Error> {
        self.enter_structural(ubl::Namespace::Cac, "TaxTotal")?;
        let value = self
            .optional_text(ubl::Namespace::Cbc, "TaxAmount", "vat_accounting_total")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        self.leave_structural()?;
        Ok(value)
    }

    // Walks the legal monetary total, keeping every amount the document states.
    fn parse_legal_monetary_total(&mut self) -> Result<MonetaryTotal, Error> {
        self.enter_structural(ubl::Namespace::Cac, "LegalMonetaryTotal")?;
        let mut total = MonetaryTotal::default();
        while self.is_open_namespace(ubl::Namespace::Cbc) {
            let (_, name) = self.head()?;
            let (field, slot) = match name.as_str() {
                "LineExtensionAmount" => ("line_net_total", &mut total.line_net_total),
                "TaxExclusiveAmount" => ("net_total", &mut total.net_total),
                "TaxInclusiveAmount" => ("gross_total", &mut total.gross_total),
                "AllowanceTotalAmount" => ("allowances_total", &mut total.allowances_total),
                "ChargeTotalAmount" => ("charges_total", &mut total.charges_total),
                "PrepaidAmount" => ("paid", &mut total.paid),
                "PayableRoundingAmount" => ("rounding", &mut total.rounding),
                "PayableAmount" => ("due", &mut total.due),
                _ => {
                    self.derived(ubl::Namespace::Cbc, &name)?;
                    continue;
                }
            };
            let text = self.leaf(ubl::Namespace::Cbc, &name, field)?;
            *slot = Some(parse_decimal(&text)?);
        }
        self.leave_structural()?;
        Ok(total)
    }

    // Parses one invoice line.
    fn parse_line(&mut self, instance: NonZeroUsize) -> Result<InvoiceLine, Error> {
        self.enter_repeatable(ubl::Namespace::Cac, "InvoiceLine", "lines", instance)?;
        let id = self.optional_leaf(ubl::Namespace::Cbc, "ID", "id")?;
        let note = self.optional_leaf(ubl::Namespace::Cbc, "Note", "note")?;
        let quantity = self
            .optional_leaf_attr(ubl::Namespace::Cbc, "InvoicedQuantity", "quantity")?
            .map(|(attributes, text)| parse_quantity(&attributes, &text))
            .transpose()?;
        let net_amount = self
            .optional_text(ubl::Namespace::Cbc, "LineExtensionAmount", "net_amount")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        let buyer_accounting_reference = self.optional_leaf(
            ubl::Namespace::Cbc,
            "AccountingCost",
            "buyer_accounting_reference",
        )?;
        let period = if self.is_open(ubl::Namespace::Cac, "InvoicePeriod") {
            self.enter_group(ubl::Namespace::Cac, "InvoicePeriod", "period")?;
            let start = self.optional_date(ubl::Namespace::Cbc, "StartDate", "period")?;
            let end = self.optional_date(ubl::Namespace::Cbc, "EndDate", "period")?;
            self.leave_group()?;
            period_from(start, end)
        } else {
            None
        };
        let order_line_reference = if self.is_open(ubl::Namespace::Cac, "OrderLineReference") {
            self.enter_structural(ubl::Namespace::Cac, "OrderLineReference")?;
            let reference =
                self.optional_leaf(ubl::Namespace::Cbc, "LineID", "order_line_reference")?;
            self.leave_structural()?;
            reference
        } else {
            None
        };
        let object = if self.is_open(ubl::Namespace::Cac, "DocumentReference") {
            self.enter_structural(ubl::Namespace::Cac, "DocumentReference")?;
            let mut reference = ObjectReference::default();
            if let Some((attributes, id)) =
                self.optional_leaf_attr(ubl::Namespace::Cbc, "ID", "object")?
            {
                reference.id = Some(id.parse()?);
                reference.scheme = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.leave_structural()?;
            Some(reference)
        } else {
            None
        };
        let mut adjustments = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "AllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_line_adjustment(instance)?);
        }
        let (item, vat) = if self.is_open(ubl::Namespace::Cac, "Item") {
            let (item, vat) = self.parse_item()?;
            (Some(item), vat)
        } else {
            (None, None)
        };
        let price = if self.is_open(ubl::Namespace::Cac, "Price") {
            Some(self.parse_price()?)
        } else {
            None
        };
        self.leave_repeatable()?;
        Ok(InvoiceLine {
            id,
            note,
            object,
            quantity,
            net_amount,
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
        self.enter_repeatable(
            ubl::Namespace::Cac,
            "AllowanceCharge",
            "adjustments",
            instance,
        )?;
        let charge = self.optional_charge_indicator()?;
        let reason = self.parse_reason(charge)?;
        let amount = self.parse_adjustment_amount()?;
        self.leave_repeatable()?;
        Ok(LineAdjustment { amount, reason })
    }

    // Parses the item, whose classified tax category yields the line VAT.
    fn parse_item(&mut self) -> Result<(Item, Option<VatTreatment>), Error> {
        self.take_open(ubl::Namespace::Cac, "Item")?;
        self.trace.enter(ubl::Namespace::Cac, "Item");
        self.trace.push_field("item");
        self.trace.record_context();

        let description = self.optional_leaf(ubl::Namespace::Cbc, "Description", "description")?;
        let name = self.optional_leaf(ubl::Namespace::Cbc, "Name", "name")?;
        let buyer_id =
            self.optional_identifier(ubl::Namespace::Cac, "BuyersItemIdentification", "buyer_id")?;
        let seller_id = self.optional_identifier(
            ubl::Namespace::Cac,
            "SellersItemIdentification",
            "seller_id",
        )?;
        let standard_id = if self.is_open(ubl::Namespace::Cac, "StandardItemIdentification") {
            self.enter_structural(ubl::Namespace::Cac, "StandardItemIdentification")?;
            let mut reference = crate::ItemReference::default();
            if let Some((attributes, id)) =
                self.optional_leaf_attr(ubl::Namespace::Cbc, "ID", "standard_id")?
            {
                reference.id = Some(id.parse()?);
                reference.issuer = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.leave_structural()?;
            Some(reference)
        } else {
            None
        };
        let country_of_origin = if self.is_open(ubl::Namespace::Cac, "OriginCountry") {
            self.enter_structural(ubl::Namespace::Cac, "OriginCountry")?;
            let code = self.optional_text(
                ubl::Namespace::Cbc,
                "IdentificationCode",
                "country_of_origin",
            )?;
            self.leave_structural()?;
            code.map(|code| parse_country(&code)).transpose()?
        } else {
            None
        };
        let mut classifications = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "CommodityClassification") {
            self.enter_structural(ubl::Namespace::Cac, "CommodityClassification")?;
            let mut classification = Classification::default();
            if let Some((attributes, id)) = self.optional_leaf_attr(
                ubl::Namespace::Cbc,
                "ItemClassificationCode",
                "classifications",
            )? {
                classification.id = Some(id.parse()?);
                classification.scheme = match attr(&attributes, "listID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
                classification.version = match attr(&attributes, "listVersionID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.leave_structural()?;
            classifications.push(classification);
        }

        // The classified tax category is a sibling of the item in the model.
        self.trace.pop_context();
        let vat = if self.is_open(ubl::Namespace::Cac, "ClassifiedTaxCategory") {
            Some(self.parse_classified_tax_category()?)
        } else {
            None
        };

        let mut attributes = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "AdditionalItemProperty") {
            self.enter_structural(ubl::Namespace::Cac, "AdditionalItemProperty")?;
            let name = self.optional_leaf(ubl::Namespace::Cbc, "Name", "attributes")?;
            let value = self.optional_leaf(ubl::Namespace::Cbc, "Value", "attributes")?;
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
        self.enter_group(ubl::Namespace::Cac, "ClassifiedTaxCategory", "vat")?;
        let category: VatCategory = self.leaf(ubl::Namespace::Cbc, "ID", "vat")?.parse()?;
        let rate = self.leaf(ubl::Namespace::Cbc, "Percent", "vat")?.parse()?;
        self.enter_structural(ubl::Namespace::Cac, "TaxScheme")?;
        self.field_or_leaf(ubl::Namespace::Cbc, "ID", "vat")?;
        self.leave_structural()?;
        self.leave_group()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    // Parses the line price.
    fn parse_price(&mut self) -> Result<Price, Error> {
        self.enter_group(ubl::Namespace::Cac, "Price", "price")?;
        let net = self
            .optional_text(ubl::Namespace::Cbc, "PriceAmount", "net")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        let base_quantity = self
            .optional_leaf_attr(ubl::Namespace::Cbc, "BaseQuantity", "price")?
            .map(|(attributes, text)| parse_quantity(&attributes, &text))
            .transpose()?;
        let (gross, discount) = if self.is_open(ubl::Namespace::Cac, "AllowanceCharge") {
            self.enter_structural(ubl::Namespace::Cac, "AllowanceCharge")?;
            self.optional_text(ubl::Namespace::Cbc, "ChargeIndicator", "price")?;
            let amount = self
                .optional_text(ubl::Namespace::Cbc, "Amount", "price")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            let base = self
                .optional_text(ubl::Namespace::Cbc, "BaseAmount", "price")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            self.leave_structural()?;
            (base, amount)
        } else {
            (None, None)
        };
        self.leave_group()?;
        Ok(Price {
            net,
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
        if !self.is_open(ubl::Namespace::Cbc, "EndpointID") {
            return Ok(None);
        }
        let (attributes, id) = self.leaf_attr(ubl::Namespace::Cbc, "EndpointID", field)?;
        let scheme = match attr(&attributes, "schemeID") {
            Some(value) => Some(value.parse()?),
            None => None,
        };
        Ok(Some(ElectronicAddress {
            id: Some(id.parse()?),
            scheme,
        }))
    }

    fn parse_identifiers(&mut self, field: &'static str) -> Result<Vec<OperationalEntity>, Error> {
        let mut identifiers = Vec::new();
        while self.is_open(ubl::Namespace::Cac, "PartyIdentification") {
            self.enter_group(ubl::Namespace::Cac, "PartyIdentification", field)?;
            let mut identifier = OperationalEntity::default();
            if let Some((attributes, id)) =
                self.optional_leaf_attr(ubl::Namespace::Cbc, "ID", field)?
            {
                identifier.id = Some(id.parse()?);
                identifier.issuer = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.leave_group()?;
            identifiers.push(identifier);
        }
        Ok(identifiers)
    }

    fn optional_party_name(
        &mut self,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if !self.is_open(ubl::Namespace::Cac, "PartyName") {
            return Ok(None);
        }
        self.enter_group(ubl::Namespace::Cac, "PartyName", field)?;
        let name = self.optional_leaf(ubl::Namespace::Cbc, "Name", field)?;
        self.leave_group()?;
        Ok(name)
    }

    fn parse_party_tax_scheme(&mut self) -> Result<Option<PartyTaxScheme>, Error> {
        // The serializer chose the context field from the scheme, so peek it first.
        let field: &'static str = if self.peek_tax_scheme() == "VAT" {
            "vat"
        } else {
            "tax_registration"
        };
        self.enter_group(ubl::Namespace::Cac, "PartyTaxScheme", field)?;
        let company = self.optional_text(ubl::Namespace::Cbc, "CompanyID", field)?;
        let mut scheme = None;
        if self.is_open(ubl::Namespace::Cac, "TaxScheme") {
            self.enter_structural(ubl::Namespace::Cac, "TaxScheme")?;
            scheme = self.optional_text(ubl::Namespace::Cbc, "ID", field)?;
            self.leave_structural()?;
        }
        self.leave_group()?;
        let Some(company) = company else {
            return Ok(None);
        };
        if scheme.as_deref() == Some("VAT") {
            Ok(Some(PartyTaxScheme::Vat(company.parse()?)))
        } else {
            Ok(Some(PartyTaxScheme::Other(company.parse()?)))
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

    // Parses the party legal entity, when present: the name, the legal id, and the legal form.
    #[allow(clippy::type_complexity)]
    fn parse_legal_entity(
        &mut self,
        name_field: &'static str,
    ) -> Result<
        (
            Option<NonEmptyString>,
            Option<LegalEntity>,
            Option<NonEmptyString>,
        ),
        Error,
    > {
        if !self.is_open(ubl::Namespace::Cac, "PartyLegalEntity") {
            return Ok((None, None, None));
        }
        self.enter_structural(ubl::Namespace::Cac, "PartyLegalEntity")?;
        let name = self.optional_leaf(ubl::Namespace::Cbc, "RegistrationName", name_field)?;
        let legal_entity = self.optional_company_id("legal_entity")?;
        let legal_form = self.optional_leaf(
            ubl::Namespace::Cbc,
            "CompanyLegalForm",
            "additional_legal_information",
        )?;
        self.leave_structural()?;
        Ok((name, legal_entity, legal_form))
    }

    fn optional_company_id(&mut self, field: &'static str) -> Result<Option<LegalEntity>, Error> {
        let Some((attributes, id)) =
            self.optional_leaf_attr(ubl::Namespace::Cbc, "CompanyID", field)?
        else {
            return Ok(None);
        };
        let issuer = match attr(&attributes, "schemeID") {
            Some(value) => Some(value.parse()?),
            None => None,
        };
        Ok(Some(LegalEntity {
            id: Some(id.parse()?),
            issuer,
        }))
    }

    fn optional_contact(&mut self, field: &'static str) -> Result<Option<Contact>, Error> {
        if !self.is_open(ubl::Namespace::Cac, "Contact") {
            return Ok(None);
        }
        self.enter_group(ubl::Namespace::Cac, "Contact", field)?;
        let name = self.optional_leaf(ubl::Namespace::Cbc, "Name", "name")?;
        let telephone = self.optional_leaf(ubl::Namespace::Cbc, "Telephone", "telephone")?;
        let email = if self.is_open(ubl::Namespace::Cbc, "ElectronicMail") {
            Some(parse_email(&self.leaf(
                ubl::Namespace::Cbc,
                "ElectronicMail",
                "email",
            )?)?)
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

    fn optional_address(
        &mut self,
        namespace: ubl::Namespace,
        element: &str,
    ) -> Result<Option<PostalAddress>, Error> {
        if !self.is_open(namespace, element) {
            return Ok(None);
        }
        self.enter_group(namespace, element, "address")?;
        let line1 = self.optional_leaf(ubl::Namespace::Cbc, "StreetName", "line1")?;
        let line2 = self.optional_leaf(ubl::Namespace::Cbc, "AdditionalStreetName", "line2")?;
        let city = self.optional_leaf(ubl::Namespace::Cbc, "CityName", "city")?;
        let postal_code = self.optional_leaf(ubl::Namespace::Cbc, "PostalZone", "postal_code")?;
        let country_subdivision = self.optional_leaf(
            ubl::Namespace::Cbc,
            "CountrySubentity",
            "country_subdivision",
        )?;
        let line3 = if self.is_open(ubl::Namespace::Cac, "AddressLine") {
            self.enter_structural(ubl::Namespace::Cac, "AddressLine")?;
            let line = self.optional_leaf(ubl::Namespace::Cbc, "Line", "line3")?;
            self.leave_structural()?;
            line
        } else {
            None
        };
        let country = if self.is_open(ubl::Namespace::Cac, "Country") {
            self.enter_structural(ubl::Namespace::Cac, "Country")?;
            let code = self.optional_text(ubl::Namespace::Cbc, "IdentificationCode", "country")?;
            self.leave_structural()?;
            code.map(|code| parse_country(&code)).transpose()?
        } else {
            None
        };
        self.leave_group()?;
        Ok(Some(PostalAddress {
            line1,
            line2,
            line3,
            city,
            country,
            country_subdivision,
            postal_code,
        }))
    }

    fn optional_identifier(
        &mut self,
        namespace: ubl::Namespace,
        element: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if !self.is_open(namespace, element) {
            return Ok(None);
        }
        self.enter_structural(namespace, element)?;
        let id = self.optional_leaf(ubl::Namespace::Cbc, "ID", field)?;
        self.leave_structural()?;
        Ok(id)
    }

    // ---- mirror primitives ----------------------------------------------

    // Reads a leaf mapped to a model field, returning its text.
    fn leaf(
        &mut self,
        namespace: ubl::Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        Ok(self.leaf_attr(namespace, name, field)?.1)
    }

    // Reads a leaf mapped to a model field, returning its attributes and text.
    fn leaf_attr(
        &mut self,
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        self.leaf(namespace, name, field)
    }

    // Reads an optional leaf mapped to a model field.
    fn optional_leaf(
        &mut self,
        namespace: ubl::Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf(namespace, name, field)?.parse()?))
        } else {
            Ok(None)
        }
    }

    // Reads the text of an optional leaf mapped to a model field.
    fn optional_text(
        &mut self,
        namespace: ubl::Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<Option<String>, Error> {
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf(namespace, name, field)?))
        } else {
            Ok(None)
        }
    }

    // Reads an optional leaf mapped to a model field, returning its attributes and text.
    #[allow(clippy::type_complexity)]
    fn optional_leaf_attr(
        &mut self,
        namespace: ubl::Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<Option<(Vec<(String, String)>, String)>, Error> {
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf_attr(namespace, name, field)?))
        } else {
            Ok(None)
        }
    }

    // Reads an optional derived leaf with no model field, returning its attributes and text.
    #[allow(clippy::type_complexity)]
    fn optional_derived(
        &mut self,
        namespace: ubl::Namespace,
        name: &str,
    ) -> Result<Option<(Vec<(String, String)>, String)>, Error> {
        if self.is_open(namespace, name) {
            Ok(Some(self.derived(namespace, name)?))
        } else {
            Ok(None)
        }
    }

    // Reads an optional date leaf mapped to a model field.
    fn optional_date(
        &mut self,
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
        element: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if !self.is_open(namespace, element) {
            return Ok(None);
        }
        self.enter_group(namespace, element, field)?;
        let id = self.leaf(ubl::Namespace::Cbc, "ID", field)?.parse()?;
        self.leave_group()?;
        Ok(Some(id))
    }

    // Reads a repeatable single-value element.
    fn repeatable_leaf(
        &mut self,
        namespace: ubl::Namespace,
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
    fn rooted(&mut self, namespace: ubl::Namespace, name: &str) -> Result<String, Error> {
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
        namespace: ubl::Namespace,
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

    fn enter_structural(&mut self, namespace: ubl::Namespace, name: &str) -> Result<(), Error> {
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

    fn enter_nested(&mut self, namespace: ubl::Namespace, name: &str) -> Result<(), Error> {
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
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
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

    fn head(&self) -> Result<(ubl::Namespace, String), Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace, name, ..
            }) => Ok((*namespace, name.clone())),
            _ => Err(bad("expected an element")),
        }
    }

    fn is_open(&self, namespace: ubl::Namespace, name: &str) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, name: local, .. }) if *found == namespace && local == name
        )
    }

    fn is_open_namespace(&self, namespace: ubl::Namespace) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, .. }) if *found == namespace
        )
    }

    fn take_open(
        &mut self,
        namespace: ubl::Namespace,
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

// The content of the tax total: the VAT total, the breakdown, and the exemption reasons.
struct TaxTotal {
    exemptions: ExemptionMap,
    vat_total: Option<Decimal>,
    breakdown: Vec<VatBreakdown>,
}

// The amounts of the legal monetary total, as the document states them.
#[derive(Default)]
struct MonetaryTotal {
    line_net_total: Option<Decimal>,
    allowances_total: Option<Decimal>,
    charges_total: Option<Decimal>,
    net_total: Option<Decimal>,
    gross_total: Option<Decimal>,
    paid: Option<Decimal>,
    rounding: Option<Decimal>,
    due: Option<Decimal>,
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
        for vat in invoice
            .lines
            .iter_mut()
            .filter_map(|line| line.vat.as_mut())
        {
            fill(vat);
        }
        for vat in invoice
            .adjustments
            .iter_mut()
            .filter_map(|adjustment| adjustment.vat.as_mut())
        {
            fill(vat);
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
                text: Some(text.parse()?),
            });
        }
    }
    Ok(Note {
        subject_code: None,
        text: Some(value.parse()?),
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
