//! The UBL parsing walk, generic over the interfaces of the semantic model.

use super::{Namespace, Token};
use crate::Format;
use crate::prelude::*;
use crate::{
    Adjustment, AdjustmentAmount, AdjustmentReason, Amount, BinaryObject, Buyer, Contact,
    CreditTransfer, Delivery, DirectDebit, DocumentBuilder, ElectronicAddress, Error, Invoice,
    InvoiceReference, Item, ItemAttribute, ItemClassification, LegalEntity, Line, LineAdjustment,
    LocationReference, MimeCode, NonEmptyString, Note, ObjectReference, OperationalEntity, Parser,
    Payee, PaymentCard, PaymentDetails, PaymentInstructions, Percentage, Period, PostalAddress,
    Price, Quantity, QuantityUnit, Seller, SupportingDocument, TaxRepresentative, Ubl,
    VatBreakdown, VatCategory, VatExemptionReason, VatPoint, VatTreatment,
};

/// Reads the whole document from the parser into a builder, from the root element down,
/// filling the dictionary in the same pass.
///
/// An implementation of `Deserializable<Ubl, N>` for a concrete invoice type calls this walk.
/// The walk builds every group from its default and fills it through the model interfaces,
/// so the invoice type and each of its groups must implement `Default`.
#[allow(clippy::type_complexity)]
pub fn deserialize<I, N>(parser: &mut Parser<Ubl, N>) -> Result<DocumentBuilder<I>, Error>
where
    I: Invoice + Default,
    I::Seller: Default,
    <I::Seller as Seller>::Contact: Default,
    I::Buyer: Default,
    <I::Buyer as Buyer>::Contact: Default,
    I::Payee: Default,
    I::TaxRepresentative: Default,
    I::Delivery: Default,
    I::Line: Default,
    <I::Line as Line>::Item: Default,
    N: crate::Namespace + From<Namespace>,
{
    parser.enter_structural(Ubl::root_namespace(), Ubl::ROOT_ELEMENT)?;

    let profile = parser.rooted(Namespace::Cbc, "CustomizationID")?.parse()?;
    let business_process = if parser.is_open(Namespace::Cbc, "ProfileID") {
        Some(parser.rooted(Namespace::Cbc, "ProfileID")?.parse()?)
    } else {
        None
    };

    let mut invoice = I::default();
    *invoice.number() = parser.optional_leaf(Namespace::Cbc, "ID", "number")?;
    *invoice.issue_date() = parser.optional_date(Namespace::Cbc, "IssueDate", "issue_date")?;
    *invoice.payment_due_date() =
        parser.optional_date(Namespace::Cbc, "DueDate", "payment_due_date")?;
    *invoice.type_code() = parser
        .leaf(Namespace::Cbc, "InvoiceTypeCode", "type_code")?
        .parse()?;

    while parser.is_open(Namespace::Cbc, "Note") {
        let instance = index(invoice.notes().len());
        let text = parser.repeatable_leaf(Namespace::Cbc, "Note", "notes", instance)?;
        let note = parse_note(&text)?;
        invoice.notes().push(note);
    }

    if parser.is_open(Namespace::Cbc, "TaxPointDate") {
        *invoice.vat_point() = Some(VatPoint::Date(parse_date(&parser.leaf(
            Namespace::Cbc,
            "TaxPointDate",
            "vat_point",
        )?)?));
    }

    *invoice.currency() = parser
        .optional_text(Namespace::Cbc, "DocumentCurrencyCode", "currency")?
        .map(|code| parse_currency(&code))
        .transpose()?;
    let accounting_currency = if parser.is_open(Namespace::Cbc, "TaxCurrencyCode") {
        Some(parse_currency(&parser.leaf(
            Namespace::Cbc,
            "TaxCurrencyCode",
            "vat_accounting_total",
        )?)?)
    } else {
        None
    };

    *invoice.buyer_accounting_reference() = parser.optional_leaf(
        Namespace::Cbc,
        "AccountingCost",
        "buyer_accounting_reference",
    )?;
    *invoice.buyer_reference() =
        parser.optional_leaf(Namespace::Cbc, "BuyerReference", "buyer_reference")?;

    if parser.is_open(Namespace::Cac, "InvoicePeriod") {
        let (period, event) = parser.parse_invoice_period()?;
        *invoice.invoicing_period() = period;
        if let Some(point) = event {
            *invoice.vat_point() = Some(point);
        }
    }

    if parser.is_open(Namespace::Cac, "OrderReference") {
        let (purchase, sales) = parser.parse_order_reference()?;
        *invoice.purchase_order_reference() = purchase;
        *invoice.sales_order_reference() = sales;
    }

    while parser.is_open(Namespace::Cac, "BillingReference") {
        let instance = index(invoice.preceding_invoices().len());
        let reference = parser.parse_billing_reference(instance)?;
        invoice.preceding_invoices().push(reference);
    }

    *invoice.despatch_advice_reference() = parser.optional_reference(
        Namespace::Cac,
        "DespatchDocumentReference",
        "despatch_advice_reference",
    )?;
    *invoice.receiving_advice_reference() = parser.optional_reference(
        Namespace::Cac,
        "ReceiptDocumentReference",
        "receiving_advice_reference",
    )?;
    *invoice.tender_or_lot_reference() = parser.optional_reference(
        Namespace::Cac,
        "OriginatorDocumentReference",
        "tender_or_lot_reference",
    )?;
    *invoice.contract_reference() = parser.optional_reference(
        Namespace::Cac,
        "ContractDocumentReference",
        "contract_reference",
    )?;

    while parser.is_open(Namespace::Cac, "AdditionalDocumentReference") {
        parser.parse_additional_document(&mut invoice)?;
    }

    *invoice.project_reference() =
        parser.optional_reference(Namespace::Cac, "ProjectReference", "project_reference")?;

    if parser.is_open(Namespace::Cac, "AccountingSupplierParty") {
        *invoice.seller() = Some(parser.parse_supplier_party()?);
    }
    if parser.is_open(Namespace::Cac, "AccountingCustomerParty") {
        *invoice.buyer() = Some(parser.parse_customer_party()?);
    }
    if parser.is_open(Namespace::Cac, "PayeeParty") {
        *invoice.payee() = Some(parser.parse_payee_party()?);
    }
    if parser.is_open(Namespace::Cac, "TaxRepresentativeParty") {
        *invoice.tax_representative() = Some(parser.parse_tax_representative_party()?);
    }
    if parser.is_open(Namespace::Cac, "Delivery") {
        *invoice.delivery() = Some(parser.parse_delivery()?);
    }
    if parser.is_open(Namespace::Cac, "PaymentMeans") {
        *invoice.payment() = Some(parser.parse_payment_means()?);
    }
    if parser.is_open(Namespace::Cac, "PaymentTerms") {
        *invoice.payment_terms() = parser.parse_payment_terms()?;
    }

    while parser.is_open(Namespace::Cac, "AllowanceCharge") {
        let instance = index(invoice.adjustments().len());
        let adjustment = parser.parse_adjustment(instance)?;
        invoice.adjustments().push(adjustment);
    }

    let mut exemptions = ExemptionMap::new();
    if parser.is_open(Namespace::Cac, "TaxTotal") {
        exemptions = parser.parse_tax_total(&mut invoice)?;
    }
    let mut accounting_value = None;
    if parser.is_open(Namespace::Cac, "TaxTotal") {
        accounting_value = parser.parse_accounting_tax_total()?;
    }
    if parser.is_open(Namespace::Cac, "LegalMonetaryTotal") {
        parser.parse_legal_monetary_total(&mut invoice)?;
    }

    while parser.is_open(Namespace::Cac, "InvoiceLine") {
        let instance = index(invoice.lines().len());
        let line = parser.parse_line(instance)?;
        invoice.lines().push(line);
    }

    parser.leave_structural()?;

    *invoice.vat_accounting_total() = match (accounting_currency, accounting_value) {
        (Some(currency), Some(value)) => Some(Amount { value, currency }),
        _ => None,
    };
    exemptions.apply(&mut invoice);

    Ok(DocumentBuilder {
        invoice,
        profile,
        business_process,
    })
}

// ---- parser --------------------------------------------------------------

impl<N: crate::Namespace + From<Namespace>> Parser<Ubl, N> {
    // Parses the invoice period, returning the period and the VAT point event.
    fn parse_invoice_period(&mut self) -> Result<(Option<Period>, Option<VatPoint>), Error> {
        self.enter_group(Namespace::Cac, "InvoicePeriod", "invoicing_period")?;
        let start = self.optional_date(Namespace::Cbc, "StartDate", "invoicing_period")?;
        let end = self.optional_date(Namespace::Cbc, "EndDate", "invoicing_period")?;
        let event = if self.is_open(Namespace::Cbc, "DescriptionCode") {
            Some(VatPoint::Event(
                self.leaf(Namespace::Cbc, "DescriptionCode", "vat_point")?
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
        self.enter_structural(Namespace::Cac, "OrderReference")?;
        let order = self.optional_leaf(Namespace::Cbc, "ID", "purchase_order_reference")?;
        let sales = self.optional_leaf(Namespace::Cbc, "SalesOrderID", "sales_order_reference")?;
        self.leave_structural()?;
        Ok((order, sales))
    }

    // Parses one preceding invoice reference.
    fn parse_billing_reference(
        &mut self,
        instance: NonZeroUsize,
    ) -> Result<InvoiceReference, Error> {
        self.enter_repeatable(
            Namespace::Cac,
            "BillingReference",
            "preceding_invoices",
            instance,
        )?;
        self.enter_structural(Namespace::Cac, "InvoiceDocumentReference")?;
        let number = self.optional_leaf(Namespace::Cbc, "ID", "number")?;
        let issue_date = self.optional_date(Namespace::Cbc, "IssueDate", "issue_date")?;
        self.leave_structural()?;
        self.leave_repeatable()?;
        Ok(InvoiceReference { number, issue_date })
    }

    // Parses one additional document reference into the object or a supporting document.
    fn parse_additional_document<I: Invoice>(&mut self, invoice: &mut I) -> Result<(), Error> {
        // Peek to know whether it is the invoiced object before committing an instance index.
        let is_object = self.peek_child_text("DocumentTypeCode").as_deref() == Some("130");
        if is_object {
            self.enter_group(Namespace::Cac, "AdditionalDocumentReference", "object")?;
            let mut id = None;
            let mut scheme = None;
            if let Some((attributes, value)) = self.optional_derived(Namespace::Cbc, "ID")? {
                id = Some(value.parse()?);
                scheme = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.derived(Namespace::Cbc, "DocumentTypeCode")?;
            self.leave_group()?;
            *invoice.object() = Some(ObjectReference { id, scheme });
        } else {
            let instance = index(invoice.supporting_documents().len());
            self.enter_repeatable(
                Namespace::Cac,
                "AdditionalDocumentReference",
                "supporting_documents",
                instance,
            )?;
            let reference = self.optional_leaf(Namespace::Cbc, "ID", "reference")?;
            let description =
                self.optional_leaf(Namespace::Cbc, "DocumentDescription", "description")?;
            let mut external_location = None;
            let mut attachment = None;
            if self.is_open(Namespace::Cac, "Attachment") {
                self.enter_structural(Namespace::Cac, "Attachment")?;
                if self.is_open(Namespace::Cac, "ExternalReference") {
                    self.enter_structural(Namespace::Cac, "ExternalReference")?;
                    external_location = self
                        .optional_text(Namespace::Cbc, "URI", "external_location")?
                        .map(|uri| parse_url(&uri))
                        .transpose()?;
                    self.leave_structural()?;
                } else {
                    attachment = Some(self.parse_binary()?);
                }
                self.leave_structural()?;
            }
            self.leave_repeatable()?;
            invoice.supporting_documents().push(SupportingDocument {
                reference,
                description,
                external_location,
                attachment,
            });
        }
        Ok(())
    }

    // Parses an embedded binary object attachment.
    fn parse_binary(&mut self) -> Result<BinaryObject, Error> {
        let (attributes, encoded) = self.derived(Namespace::Cbc, "EmbeddedDocumentBinaryObject")?;
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
    fn parse_supplier_party<S: Seller + Default>(&mut self) -> Result<S, Error>
    where
        S::Contact: Default,
    {
        self.enter_structural(Namespace::Cac, "AccountingSupplierParty")?;
        self.enter_group(Namespace::Cac, "Party", "seller")?;
        let mut seller = S::default();
        *seller.electronic_address() = self.optional_endpoint("electronic_address")?;
        *seller.identifiers() = self.parse_identifiers("identifiers")?;
        *seller.trading_name() = self.optional_party_name("trading_name")?;
        *seller.address() = self.optional_address(Namespace::Cac, "PostalAddress")?;
        while self.is_open(Namespace::Cac, "PartyTaxScheme") {
            match self.parse_party_tax_scheme()? {
                Some(PartyTaxScheme::Vat(value)) => *seller.vat() = Some(value),
                Some(PartyTaxScheme::Other(value)) => *seller.tax_registration() = Some(value),
                None => {}
            }
        }
        let (name, legal_entity, additional_legal_information) = self.parse_legal_entity("name")?;
        *seller.name() = name;
        *seller.legal_entity() = legal_entity;
        *seller.additional_legal_information() = additional_legal_information;
        *seller.contact() = self.optional_contact("contact")?;
        self.leave_group()?;
        self.leave_structural()?;
        Ok(seller)
    }

    // Parses the customer party into a buyer.
    fn parse_customer_party<B: Buyer + Default>(&mut self) -> Result<B, Error>
    where
        B::Contact: Default,
    {
        self.enter_structural(Namespace::Cac, "AccountingCustomerParty")?;
        self.enter_group(Namespace::Cac, "Party", "buyer")?;
        let mut buyer = B::default();
        *buyer.electronic_address() = self.optional_endpoint("electronic_address")?;
        *buyer.identifiers() = self.parse_identifiers("identifiers")?;
        *buyer.trading_name() = self.optional_party_name("trading_name")?;
        *buyer.address() = self.optional_address(Namespace::Cac, "PostalAddress")?;
        while self.is_open(Namespace::Cac, "PartyTaxScheme") {
            if let Some(PartyTaxScheme::Vat(value)) = self.parse_party_tax_scheme()? {
                *buyer.vat() = Some(value);
            }
        }
        let (name, legal_entity, _) = self.parse_legal_entity("name")?;
        *buyer.name() = name;
        *buyer.legal_entity() = legal_entity;
        *buyer.contact() = self.optional_contact("contact")?;
        self.leave_group()?;
        self.leave_structural()?;
        Ok(buyer)
    }

    // Parses the payee party.
    fn parse_payee_party<P: Payee + Default>(&mut self) -> Result<P, Error> {
        self.enter_group(Namespace::Cac, "PayeeParty", "payee")?;
        let mut payee = P::default();
        *payee.identifiers() = self.parse_identifiers("identifiers")?;
        *payee.name() = self.optional_party_name("name")?;
        if self.is_open(Namespace::Cac, "PartyLegalEntity") {
            self.enter_structural(Namespace::Cac, "PartyLegalEntity")?;
            *payee.legal_entity() = self.optional_company_id("legal_entity")?;
            self.leave_structural()?;
        }
        self.leave_group()?;
        Ok(payee)
    }

    // Parses the tax representative party.
    fn parse_tax_representative_party<T: TaxRepresentative + Default>(
        &mut self,
    ) -> Result<T, Error> {
        self.enter_group(
            Namespace::Cac,
            "TaxRepresentativeParty",
            "tax_representative",
        )?;
        let mut representative = T::default();
        *representative.name() = self.optional_party_name("name")?;
        *representative.address() = self.optional_address(Namespace::Cac, "PostalAddress")?;
        if self.is_open(Namespace::Cac, "PartyTaxScheme") {
            *representative.vat() = match self.parse_party_tax_scheme()? {
                Some(PartyTaxScheme::Vat(value)) => Some(value),
                Some(PartyTaxScheme::Other(_)) => {
                    return Err(bad("a tax representative without a VAT scheme"));
                }
                None => None,
            };
        }
        self.leave_group()?;
        Ok(representative)
    }

    // Parses the delivery information.
    fn parse_delivery<D: Delivery + Default>(&mut self) -> Result<D, Error> {
        self.enter_group(Namespace::Cac, "Delivery", "delivery")?;
        let mut delivery = D::default();
        *delivery.date() = self.optional_date(Namespace::Cbc, "ActualDeliveryDate", "date")?;
        if self.is_open(Namespace::Cac, "DeliveryLocation") {
            self.enter_structural(Namespace::Cac, "DeliveryLocation")?;
            if self.is_open(Namespace::Cbc, "ID") {
                let (attributes, id) = self.leaf_attr(Namespace::Cbc, "ID", "location")?;
                let issuer = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
                *delivery.location() = Some(LocationReference {
                    id: Some(id.parse()?),
                    issuer,
                });
            }
            *delivery.address() = self.optional_address(Namespace::Cac, "Address")?;
            self.leave_structural()?;
        }
        if self.is_open(Namespace::Cac, "DeliveryParty") {
            self.enter_structural(Namespace::Cac, "DeliveryParty")?;
            *delivery.name() = self.optional_party_name("name")?;
            self.leave_structural()?;
        }
        self.leave_group()?;
        Ok(delivery)
    }

    // Parses the payment means.
    fn parse_payment_means(&mut self) -> Result<PaymentInstructions, Error> {
        self.enter_group(Namespace::Cac, "PaymentMeans", "payment")?;
        let mut means = None;
        let mut means_text = None;
        if let Some((attributes, code)) =
            self.optional_leaf_attr(Namespace::Cbc, "PaymentMeansCode", "means")?
        {
            means = Some(code.parse()?);
            means_text = match attr(&attributes, "name") {
                Some(value) => Some(value.parse()?),
                None => None,
            };
        }
        let remittance_information =
            self.optional_leaf(Namespace::Cbc, "PaymentID", "remittance_information")?;
        let details = if self.is_open(Namespace::Cac, "PayeeFinancialAccount") {
            let mut transfers = Vec::new();
            while self.is_open(Namespace::Cac, "PayeeFinancialAccount") {
                transfers.push(self.parse_credit_transfer()?);
            }
            Some(PaymentDetails::CreditTransfers(transfers))
        } else if self.is_open(Namespace::Cac, "CardAccount") {
            Some(PaymentDetails::Card(self.parse_card()?))
        } else if self.is_open(Namespace::Cac, "PaymentMandate") {
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
        self.enter_structural(Namespace::Cac, "PayeeFinancialAccount")?;
        let account = self
            .optional_text(Namespace::Cbc, "ID", "account")?
            .map(|value| value.parse())
            .transpose()?;
        let account_name = self.optional_leaf(Namespace::Cbc, "Name", "account_name")?;
        let provider = if self.is_open(Namespace::Cac, "FinancialInstitutionBranch") {
            self.enter_structural(Namespace::Cac, "FinancialInstitutionBranch")?;
            let provider = self
                .optional_text(Namespace::Cbc, "ID", "provider")?
                .map(|value| value.parse())
                .transpose()?;
            self.leave_structural()?;
            provider
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
        self.enter_structural(Namespace::Cac, "CardAccount")?;
        let primary_account_number = self.optional_leaf(
            Namespace::Cbc,
            "PrimaryAccountNumberID",
            "primary_account_number",
        )?;
        let holder_name = self.optional_leaf(Namespace::Cbc, "HolderName", "holder_name")?;
        self.leave_structural()?;
        Ok(PaymentCard {
            primary_account_number,
            holder_name,
        })
    }

    fn parse_direct_debit(&mut self) -> Result<DirectDebit, Error> {
        self.enter_structural(Namespace::Cac, "PaymentMandate")?;
        let mandate_reference = self.optional_leaf(Namespace::Cbc, "ID", "mandate_reference")?;
        let creditor_identifier =
            self.optional_leaf(Namespace::Cbc, "PayerPartyID", "creditor_identifier")?;
        let debited_account = if self.is_open(Namespace::Cac, "PayerFinancialAccount") {
            self.enter_structural(Namespace::Cac, "PayerFinancialAccount")?;
            let account = self
                .optional_text(Namespace::Cbc, "ID", "debited_account")?
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
        self.enter_group(Namespace::Cac, "PaymentTerms", "payment_terms")?;
        let note = self.optional_leaf(Namespace::Cbc, "Note", "payment_terms")?;
        self.leave_group()?;
        Ok(note)
    }

    // Parses one document-level allowance or charge.
    fn parse_adjustment(&mut self, instance: NonZeroUsize) -> Result<Adjustment, Error> {
        self.enter_repeatable(Namespace::Cac, "AllowanceCharge", "adjustments", instance)?;
        let charge = self.optional_charge_indicator()?;
        let reason = self.parse_reason(charge)?;
        let amount = self.parse_adjustment_amount()?;
        let vat = if self.is_open(Namespace::Cac, "TaxCategory") {
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
            .optional_derived(Namespace::Cbc, "ChargeIndicator")?
            .map(|(_, text)| text.trim() == "true"))
    }

    // Parses the reason code and text of an adjustment, dropped without the direction.
    fn parse_reason(&mut self, charge: Option<bool>) -> Result<Option<AdjustmentReason>, Error> {
        let code = if self.is_open(Namespace::Cbc, "AllowanceChargeReasonCode") {
            Some(self.derived(Namespace::Cbc, "AllowanceChargeReasonCode")?.1)
        } else {
            None
        };
        let text = if self.is_open(Namespace::Cbc, "AllowanceChargeReason") {
            Some(
                self.derived(Namespace::Cbc, "AllowanceChargeReason")?
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
        let factor = self.optional_text(Namespace::Cbc, "MultiplierFactorNumeric", "amount")?;
        let amount = self
            .optional_text(Namespace::Cbc, "Amount", "amount")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        let base = self
            .optional_text(Namespace::Cbc, "BaseAmount", "amount")?
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
        self.enter_nested(Namespace::Cac, "TaxCategory")?;
        let category: VatCategory = self.derived(Namespace::Cbc, "ID")?.1.parse()?;
        let rate = self.derived(Namespace::Cbc, "Percent")?.1.parse()?;
        self.enter_nested(Namespace::Cac, "TaxScheme")?;
        self.derived(Namespace::Cbc, "ID")?;
        self.leave_nested()?;
        self.leave_nested()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    // Parses the tax total into the invoice: the VAT total (`BT-110`) and the breakdown
    // (`BG-23`), collecting exemption reasons per category.
    fn parse_tax_total<I: Invoice>(&mut self, invoice: &mut I) -> Result<ExemptionMap, Error> {
        let mut exemptions = ExemptionMap::new();
        self.enter_structural(Namespace::Cac, "TaxTotal")?;
        *invoice.vat_total() = self
            .optional_text(Namespace::Cbc, "TaxAmount", "vat_total")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        while self.is_open(Namespace::Cac, "TaxSubtotal") {
            let instance = index(invoice.vat_breakdown().len());
            self.enter_repeatable(Namespace::Cac, "TaxSubtotal", "vat_breakdown", instance)?;
            let taxable = self
                .optional_text(Namespace::Cbc, "TaxableAmount", "taxable")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            let tax = self
                .optional_text(Namespace::Cbc, "TaxAmount", "tax")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            let treatment = if self.is_open(Namespace::Cac, "TaxCategory") {
                self.parse_breakdown_category()?
            } else {
                None
            };
            self.leave_repeatable()?;
            if let Some(VatTreatment::Exempt { code, text }) = &treatment {
                exemptions.set(*code, text.clone());
            }
            invoice.vat_breakdown().push(VatBreakdown {
                treatment,
                taxable,
                tax,
            });
        }
        self.leave_structural()?;
        Ok(exemptions)
    }

    // Parses the VAT category of a breakdown group, absent without the category code.
    fn parse_breakdown_category(&mut self) -> Result<Option<VatTreatment>, Error> {
        self.enter_group(Namespace::Cac, "TaxCategory", "treatment")?;
        let category: Option<VatCategory> = self
            .optional_derived(Namespace::Cbc, "ID")?
            .map(|(_, text)| text.parse())
            .transpose()?;
        let rate: Option<Percentage> = self
            .optional_derived(Namespace::Cbc, "Percent")?
            .map(|(_, text)| text.parse())
            .transpose()?;
        let mut code = None;
        let mut text = None;
        if self.is_open(Namespace::Cbc, "TaxExemptionReasonCode") {
            code = Some(
                self.derived(Namespace::Cbc, "TaxExemptionReasonCode")?
                    .1
                    .parse()?,
            );
        }
        if self.is_open(Namespace::Cbc, "TaxExemptionReason") {
            text = Some(
                self.derived(Namespace::Cbc, "TaxExemptionReason")?
                    .1
                    .parse()?,
            );
        }
        if self.is_open(Namespace::Cac, "TaxScheme") {
            self.enter_nested(Namespace::Cac, "TaxScheme")?;
            self.optional_derived(Namespace::Cbc, "ID")?;
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
        self.enter_structural(Namespace::Cac, "TaxTotal")?;
        let value = self
            .optional_text(Namespace::Cbc, "TaxAmount", "vat_accounting_total")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        self.leave_structural()?;
        Ok(value)
    }

    // Walks the legal monetary total, keeping every amount the document states.
    fn parse_legal_monetary_total<I: Invoice>(&mut self, invoice: &mut I) -> Result<(), Error> {
        self.enter_structural(Namespace::Cac, "LegalMonetaryTotal")?;
        while self.is_open_namespace(Namespace::Cbc) {
            let (_, name) = self.head()?;
            let (field, slot): (&'static str, &mut Option<Decimal>) = match name.as_str() {
                "LineExtensionAmount" => ("line_net_total", invoice.line_net_total()),
                "TaxExclusiveAmount" => ("net_total", invoice.net_total()),
                "TaxInclusiveAmount" => ("gross_total", invoice.gross_total()),
                "AllowanceTotalAmount" => ("allowances_total", invoice.allowances_total()),
                "ChargeTotalAmount" => ("charges_total", invoice.charges_total()),
                "PrepaidAmount" => ("paid", invoice.paid()),
                "PayableRoundingAmount" => ("rounding", invoice.rounding()),
                "PayableAmount" => ("due", invoice.due()),
                _ => {
                    self.derived(Namespace::Cbc, &name)?;
                    continue;
                }
            };
            let text = self.leaf(Namespace::Cbc, &name, field)?;
            *slot = Some(parse_decimal(&text)?);
        }
        self.leave_structural()?;
        Ok(())
    }

    // Parses one invoice line.
    fn parse_line<L: Line + Default>(&mut self, instance: NonZeroUsize) -> Result<L, Error>
    where
        L::Item: Default,
    {
        self.enter_repeatable(Namespace::Cac, "InvoiceLine", "lines", instance)?;
        let mut line = L::default();
        *line.id() = self.optional_leaf(Namespace::Cbc, "ID", "id")?;
        *line.note() = self.optional_leaf(Namespace::Cbc, "Note", "note")?;
        *line.quantity() = self
            .optional_leaf_attr(Namespace::Cbc, "InvoicedQuantity", "quantity")?
            .map(|(attributes, text)| parse_quantity(&attributes, &text))
            .transpose()?;
        *line.net_amount() = self
            .optional_text(Namespace::Cbc, "LineExtensionAmount", "net_amount")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        *line.buyer_accounting_reference() = self.optional_leaf(
            Namespace::Cbc,
            "AccountingCost",
            "buyer_accounting_reference",
        )?;
        if self.is_open(Namespace::Cac, "InvoicePeriod") {
            self.enter_group(Namespace::Cac, "InvoicePeriod", "period")?;
            let start = self.optional_date(Namespace::Cbc, "StartDate", "period")?;
            let end = self.optional_date(Namespace::Cbc, "EndDate", "period")?;
            self.leave_group()?;
            *line.period() = period_from(start, end);
        }
        if self.is_open(Namespace::Cac, "OrderLineReference") {
            self.enter_structural(Namespace::Cac, "OrderLineReference")?;
            *line.order_line_reference() =
                self.optional_leaf(Namespace::Cbc, "LineID", "order_line_reference")?;
            self.leave_structural()?;
        }
        if self.is_open(Namespace::Cac, "DocumentReference") {
            self.enter_structural(Namespace::Cac, "DocumentReference")?;
            let mut reference = ObjectReference::default();
            if let Some((attributes, id)) =
                self.optional_leaf_attr(Namespace::Cbc, "ID", "object")?
            {
                reference.id = Some(id.parse()?);
                reference.scheme = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.leave_structural()?;
            *line.object() = Some(reference);
        }
        while self.is_open(Namespace::Cac, "AllowanceCharge") {
            let instance = index(line.adjustments().len());
            let adjustment = self.parse_line_adjustment(instance)?;
            line.adjustments().push(adjustment);
        }
        if self.is_open(Namespace::Cac, "Item") {
            self.parse_item(&mut line)?;
        }
        if self.is_open(Namespace::Cac, "Price") {
            *line.price() = Some(self.parse_price()?);
        }
        self.leave_repeatable()?;
        Ok(line)
    }

    // Parses one line-level allowance or charge.
    fn parse_line_adjustment(&mut self, instance: NonZeroUsize) -> Result<LineAdjustment, Error> {
        self.enter_repeatable(Namespace::Cac, "AllowanceCharge", "adjustments", instance)?;
        let charge = self.optional_charge_indicator()?;
        let reason = self.parse_reason(charge)?;
        let amount = self.parse_adjustment_amount()?;
        self.leave_repeatable()?;
        Ok(LineAdjustment { amount, reason })
    }

    // Parses the item into the line, whose classified tax category yields the line VAT.
    // The item is borrowed twice, because the classified tax category sits between its head
    // and its attributes in the UBL schema.
    fn parse_item<L: Line>(&mut self, line: &mut L) -> Result<(), Error>
    where
        L::Item: Default,
    {
        self.take_open(Namespace::Cac, "Item")?;
        self.trace.enter(Namespace::Cac.into(), "Item");
        self.trace.push_field("item");
        self.trace.record_context();

        let item = line.item().insert(Default::default());
        *item.description() = self.optional_leaf(Namespace::Cbc, "Description", "description")?;
        *item.name() = self.optional_leaf(Namespace::Cbc, "Name", "name")?;
        *item.buyer_id() =
            self.optional_identifier(Namespace::Cac, "BuyersItemIdentification", "buyer_id")?;
        *item.seller_id() =
            self.optional_identifier(Namespace::Cac, "SellersItemIdentification", "seller_id")?;
        if self.is_open(Namespace::Cac, "StandardItemIdentification") {
            self.enter_structural(Namespace::Cac, "StandardItemIdentification")?;
            let mut reference = crate::ItemReference::default();
            if let Some((attributes, id)) =
                self.optional_leaf_attr(Namespace::Cbc, "ID", "standard_id")?
            {
                reference.id = Some(id.parse()?);
                reference.issuer = match attr(&attributes, "schemeID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.leave_structural()?;
            *item.standard_id() = Some(reference);
        }
        if self.is_open(Namespace::Cac, "OriginCountry") {
            self.enter_structural(Namespace::Cac, "OriginCountry")?;
            let code =
                self.optional_text(Namespace::Cbc, "IdentificationCode", "country_of_origin")?;
            self.leave_structural()?;
            *item.country_of_origin() = code.map(|code| parse_country(&code)).transpose()?;
        }
        while self.is_open(Namespace::Cac, "CommodityClassification") {
            self.enter_structural(Namespace::Cac, "CommodityClassification")?;
            let mut classification = ItemClassification::default();
            if let Some((attributes, id)) = self.optional_leaf_attr(
                Namespace::Cbc,
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
            item.classifications().push(classification);
        }

        // The classified tax category is a sibling of the item in the model.
        self.trace.pop_context();
        if self.is_open(Namespace::Cac, "ClassifiedTaxCategory") {
            *line.vat() = Some(self.parse_classified_tax_category()?);
        }

        if let Some(item) = line.item() {
            while self.is_open(Namespace::Cac, "AdditionalItemProperty") {
                self.enter_structural(Namespace::Cac, "AdditionalItemProperty")?;
                let name = self.optional_leaf(Namespace::Cbc, "Name", "attributes")?;
                let value = self.optional_leaf(Namespace::Cbc, "Value", "attributes")?;
                self.leave_structural()?;
                item.attributes().push(ItemAttribute { name, value });
            }
        }

        self.take_close()?;
        self.trace.leave();
        Ok(())
    }

    // Parses the classified tax category into the line VAT treatment.
    fn parse_classified_tax_category(&mut self) -> Result<VatTreatment, Error> {
        self.enter_group(Namespace::Cac, "ClassifiedTaxCategory", "vat")?;
        let category: VatCategory = self.leaf(Namespace::Cbc, "ID", "vat")?.parse()?;
        let rate = self.leaf(Namespace::Cbc, "Percent", "vat")?.parse()?;
        self.enter_structural(Namespace::Cac, "TaxScheme")?;
        self.leaf(Namespace::Cbc, "ID", "vat")?;
        self.leave_structural()?;
        self.leave_group()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    // Parses the line price.
    fn parse_price(&mut self) -> Result<Price, Error> {
        self.enter_group(Namespace::Cac, "Price", "price")?;
        let net = self
            .optional_text(Namespace::Cbc, "PriceAmount", "net")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        let base_quantity = self
            .optional_leaf_attr(Namespace::Cbc, "BaseQuantity", "price")?
            .map(|(attributes, text)| parse_quantity(&attributes, &text))
            .transpose()?;
        let mut discount = None;
        let mut gross = None;
        if self.is_open(Namespace::Cac, "AllowanceCharge") {
            self.enter_structural(Namespace::Cac, "AllowanceCharge")?;
            self.optional_text(Namespace::Cbc, "ChargeIndicator", "price")?;
            discount = self
                .optional_text(Namespace::Cbc, "Amount", "price")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            gross = self
                .optional_text(Namespace::Cbc, "BaseAmount", "price")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            self.leave_structural()?;
        }
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
        if !self.is_open(Namespace::Cbc, "EndpointID") {
            return Ok(None);
        }
        let (attributes, id) = self.leaf_attr(Namespace::Cbc, "EndpointID", field)?;
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
        while self.is_open(Namespace::Cac, "PartyIdentification") {
            self.enter_group(Namespace::Cac, "PartyIdentification", field)?;
            let mut identifier = OperationalEntity::default();
            if let Some((attributes, id)) = self.optional_leaf_attr(Namespace::Cbc, "ID", field)? {
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
        if !self.is_open(Namespace::Cac, "PartyName") {
            return Ok(None);
        }
        self.enter_group(Namespace::Cac, "PartyName", field)?;
        let name = self.optional_leaf(Namespace::Cbc, "Name", field)?;
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
        self.enter_group(Namespace::Cac, "PartyTaxScheme", field)?;
        let company = self.optional_text(Namespace::Cbc, "CompanyID", field)?;
        let mut scheme = None;
        if self.is_open(Namespace::Cac, "TaxScheme") {
            self.enter_structural(Namespace::Cac, "TaxScheme")?;
            scheme = self.optional_text(Namespace::Cbc, "ID", field)?;
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
        if !self.is_open(Namespace::Cac, "PartyLegalEntity") {
            return Ok((None, None, None));
        }
        self.enter_structural(Namespace::Cac, "PartyLegalEntity")?;
        let name = self.optional_leaf(Namespace::Cbc, "RegistrationName", name_field)?;
        let legal_entity = self.optional_company_id("legal_entity")?;
        let legal_form = self.optional_leaf(
            Namespace::Cbc,
            "CompanyLegalForm",
            "additional_legal_information",
        )?;
        self.leave_structural()?;
        Ok((name, legal_entity, legal_form))
    }

    fn optional_company_id(&mut self, field: &'static str) -> Result<Option<LegalEntity>, Error> {
        let Some((attributes, id)) = self.optional_leaf_attr(Namespace::Cbc, "CompanyID", field)?
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

    fn optional_contact<C: Contact + Default>(
        &mut self,
        field: &'static str,
    ) -> Result<Option<C>, Error> {
        if !self.is_open(Namespace::Cac, "Contact") {
            return Ok(None);
        }
        self.enter_group(Namespace::Cac, "Contact", field)?;
        let mut contact = C::default();
        *contact.name() = self.optional_leaf(Namespace::Cbc, "Name", "name")?;
        *contact.telephone() = self.optional_leaf(Namespace::Cbc, "Telephone", "telephone")?;
        if self.is_open(Namespace::Cbc, "ElectronicMail") {
            *contact.email() = Some(parse_email(&self.leaf(
                Namespace::Cbc,
                "ElectronicMail",
                "email",
            )?)?);
        }
        self.leave_group()?;
        Ok(Some(contact))
    }

    fn optional_address(
        &mut self,
        namespace: Namespace,
        element: &str,
    ) -> Result<Option<PostalAddress>, Error> {
        if !self.is_open(namespace, element) {
            return Ok(None);
        }
        self.enter_group(namespace, element, "address")?;
        let line1 = self.optional_leaf(Namespace::Cbc, "StreetName", "line1")?;
        let line2 = self.optional_leaf(Namespace::Cbc, "AdditionalStreetName", "line2")?;
        let city = self.optional_leaf(Namespace::Cbc, "CityName", "city")?;
        let postal_code = self.optional_leaf(Namespace::Cbc, "PostalZone", "postal_code")?;
        let country_subdivision =
            self.optional_leaf(Namespace::Cbc, "CountrySubentity", "country_subdivision")?;
        let line3 = if self.is_open(Namespace::Cac, "AddressLine") {
            self.enter_structural(Namespace::Cac, "AddressLine")?;
            let line = self.optional_leaf(Namespace::Cbc, "Line", "line3")?;
            self.leave_structural()?;
            line
        } else {
            None
        };
        let country = if self.is_open(Namespace::Cac, "Country") {
            self.enter_structural(Namespace::Cac, "Country")?;
            let code = self.optional_text(Namespace::Cbc, "IdentificationCode", "country")?;
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
            postal_code,
            country_subdivision,
            country,
        }))
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
        let id = self.optional_leaf(Namespace::Cbc, "ID", field)?;
        self.leave_structural()?;
        Ok(id)
    }

    // ---- binding-specific readers ---------------------------------------

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
        let id = self.leaf(Namespace::Cbc, "ID", field)?.parse()?;
        self.leave_group()?;
        Ok(Some(id))
    }
}

// ---- helpers -------------------------------------------------------------

enum PartyTaxScheme {
    Vat(crate::VatIdentifier),
    Other(NonEmptyString),
}

// The exemption reason of the single exempt VAT group, applied back to the model.
struct ExemptionMap {
    exempt: Option<(Option<VatExemptionReason>, Option<NonEmptyString>)>,
}

impl ExemptionMap {
    fn new() -> Self {
        Self { exempt: None }
    }

    fn set(&mut self, code: Option<VatExemptionReason>, text: Option<NonEmptyString>) {
        self.exempt = Some((code, text));
    }

    // Fills the exemption reason into every exempt line and adjustment VAT.
    fn apply<I: Invoice>(&self, invoice: &mut I) {
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
        for line in invoice.lines().iter_mut() {
            if let Some(vat) = line.vat() {
                fill(vat);
            }
        }
        for adjustment in invoice.adjustments().iter_mut() {
            if let Some(vat) = &mut adjustment.vat {
                fill(vat);
            }
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
    let unit = QuantityUnit::from_code(code)
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
