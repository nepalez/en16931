use crate::format::cii::{RAM, RSM};
use crate::format::trace::Trace;
use crate::prelude::*;
use crate::{
    Abbreviations, Adjustment, AdjustmentAmount, AdjustmentReason, Amount, Buyer, Classification,
    Contact, CreditTransfer, Delivery, Dictionary, DirectDebit, DocumentBuilder, ElectronicAddress,
    Error, ExemptionReason, Invoice, InvoiceLine, Item, ItemAttribute, ItemReference, LegalEntity,
    LineAdjustment, LocationReference, Namespace, NonEmptyString, Note, ObjectReference,
    OperationalEntity, Payee, PaymentCard, PaymentDetails, PaymentInstructions, Period,
    PostalAddress, PrecedingInvoice, Price, Quantity, Seller, SupportingDocument,
    TaxRepresentative, Unit, VatCategory, VatPoint, VatTreatment,
};

/// Parses a CII document from XML,
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
    // A record-form namespace element (`namespace: Some`) or a datatype carrier
    // from `udt`/`qdt` (`namespace: None`), which is never recorded.
    Open {
        namespace: Option<Namespace>,
        name: String,
        attributes: Vec<(String, String)>,
    },
    Text(String),
    Close,
}

fn tokenize(xml: &str) -> Result<(Vec<Token>, Abbreviations), Error> {
    let mut reader = NsReader::from_str(xml);
    let mut tokens = Vec::new();
    let mut abbreviations = Abbreviations::default();
    loop {
        let (resolved, event) = reader
            .read_resolved_event()
            .map_err(|error| Error::MalformedXml(error.to_string()))?;
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

fn open_token(
    resolved: ResolveResult<'_>,
    start: &BytesStart<'_>,
    abbreviations: &mut Abbreviations,
) -> Result<Token, Error> {
    let ResolveResult::Bound(uri) = resolved else {
        return Err(Error::MalformedXml(
            "an element has no namespace".to_owned(),
        ));
    };
    let uri = String::from_utf8_lossy(uri.into_inner());
    let namespace = Namespace::from_uri(&uri);
    let name = String::from_utf8_lossy(start.local_name().as_ref()).into_owned();
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|error| Error::MalformedXml(error.to_string()))?;
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
        self.take_open(RSM, "CrossIndustryInvoice")?;
        self.trace.enter(RSM, "CrossIndustryInvoice");
        self.trace.record_root();

        let (profile, business_process) = self.exchanged_document_context()?;
        let (number, type_code, issue_date, notes) = self.exchanged_document()?;

        self.enter_structural(RSM, "SupplyChainTradeTransaction")?;
        let mut lines = Vec::new();
        while self.is_open(RAM, "IncludedSupplyChainTradeLineItem") {
            let instance = index(lines.len());
            lines.push(self.parse_line(instance)?);
        }
        let agreement = self.header_trade_agreement()?;
        let delivery = self.header_trade_delivery()?;
        let settlement = self.header_trade_settlement()?;
        self.leave_structural()?;

        self.take_close()?;
        self.trace.leave();

        let mut invoice = Invoice {
            number,
            issue_date,
            type_code,
            currency: settlement.currency,
            vat_accounting_total: settlement.vat_accounting_total,
            vat_point: settlement.vat_point,
            payment_due_date: settlement.payment_due_date,
            buyer_reference: agreement.buyer_reference,
            project_reference: agreement.project_reference,
            contract_reference: agreement.contract_reference,
            purchase_order_reference: agreement.purchase_order_reference,
            sales_order_reference: agreement.sales_order_reference,
            receiving_advice_reference: delivery.receiving_advice_reference,
            despatch_advice_reference: delivery.despatch_advice_reference,
            tender_or_lot_reference: agreement.tender_or_lot_reference,
            object: agreement.object,
            buyer_accounting_reference: settlement.buyer_accounting_reference,
            payment_terms: settlement.payment_terms,
            notes,
            preceding_invoices: settlement.preceding_invoices,
            seller: agreement.seller,
            buyer: agreement.buyer,
            payee: settlement.payee,
            tax_representative: agreement.tax_representative,
            delivery: delivery.delivery,
            invoicing_period: settlement.invoicing_period,
            adjustments: settlement.adjustments,
            rounding: settlement.rounding,
            payment: settlement.payment,
            paid: settlement.paid,
            supporting_documents: agreement.supporting_documents,
            lines,
        };
        settlement.exemptions.apply(&mut invoice);

        Ok(DocumentBuilder {
            invoice,
            profile,
            binding: crate::Binding::Cii,
            business_process,
        })
    }

    // Parses the document context: the profile (`BT-24`) and business process (`BT-23`).
    fn exchanged_document_context(
        &mut self,
    ) -> Result<(crate::Profile, Option<crate::BusinessProcess>), Error> {
        self.enter_structural(RSM, "ExchangedDocumentContext")?;
        let business_process =
            if self.is_open(RAM, "BusinessProcessSpecifiedDocumentContextParameter") {
                self.enter_structural(RAM, "BusinessProcessSpecifiedDocumentContextParameter")?;
                let value = self.rooted(RAM, "ID")?.parse()?;
                self.leave_structural()?;
                Some(value)
            } else {
                None
            };
        self.enter_structural(RAM, "GuidelineSpecifiedDocumentContextParameter")?;
        let profile = self.rooted(RAM, "ID")?.parse()?;
        self.leave_structural()?;
        self.leave_structural()?;
        Ok((profile, business_process))
    }

    // Parses the exchanged document header.
    fn exchanged_document(
        &mut self,
    ) -> Result<(NonEmptyString, crate::InvoiceType, Date, Vec<Note>), Error> {
        self.enter_structural(RSM, "ExchangedDocument")?;
        let number = self.leaf(RAM, "ID", "number")?.parse()?;
        let type_code = self.leaf(RAM, "TypeCode", "type_code")?.parse()?;
        let issue_date = self.datetime("IssueDateTime", "issue_date")?;
        let mut notes = Vec::new();
        while self.is_open(RAM, "IncludedNote") {
            let instance = index(notes.len());
            self.enter_repeatable(RAM, "IncludedNote", "notes", instance)?;
            let text = self.derived(RAM, "Content")?.1.parse()?;
            let subject_code = if self.is_open(RAM, "SubjectCode") {
                Some(self.derived(RAM, "SubjectCode")?.1.parse()?)
            } else {
                None
            };
            self.leave_repeatable()?;
            notes.push(Note { subject_code, text });
        }
        self.leave_structural()?;
        Ok((number, type_code, issue_date, notes))
    }

    // Parses one invoice line.
    fn parse_line(&mut self, instance: NonZeroUsize) -> Result<InvoiceLine, Error> {
        self.enter_repeatable(RAM, "IncludedSupplyChainTradeLineItem", "lines", instance)?;

        self.enter_structural(RAM, "AssociatedDocumentLineDocument")?;
        let id = self.leaf(RAM, "LineID", "id")?.parse()?;
        let note = if self.is_open(RAM, "IncludedNote") {
            self.enter_group(RAM, "IncludedNote", "note")?;
            let content = self.derived(RAM, "Content")?.1.parse()?;
            self.leave_group()?;
            Some(content)
        } else {
            None
        };
        self.leave_structural()?;

        let item = self.parse_product()?;
        let (order_line_reference, price) = self.parse_agreement()?;

        self.enter_structural(RAM, "SpecifiedLineTradeDelivery")?;
        let (quantity_attrs, quantity_text) = self.leaf_attr(RAM, "BilledQuantity", "quantity")?;
        let quantity = parse_quantity(&quantity_attrs, &quantity_text)?;
        self.leave_structural()?;

        let (vat, period, adjustments, object, buyer_accounting_reference) =
            self.parse_line_settlement()?;

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

    // Parses the line product into an item.
    fn parse_product(&mut self) -> Result<Item, Error> {
        self.take_open(RAM, "SpecifiedTradeProduct")?;
        self.trace.enter(RAM, "SpecifiedTradeProduct");
        self.trace.push_field("item");
        self.trace.record_context();

        let standard_id = if self.is_open(RAM, "GlobalID") {
            let (attributes, id) = self.leaf_attr(RAM, "GlobalID", "standard_id")?;
            let issuer = attr(&attributes, "schemeID")
                .ok_or_else(|| bad("a standard item id without a scheme"))?
                .parse()?;
            Some(ItemReference {
                id: id.parse()?,
                issuer,
            })
        } else {
            None
        };
        let seller_id = self.optional_leaf(RAM, "SellerAssignedID", "seller_id")?;
        let buyer_id = self.optional_leaf(RAM, "BuyerAssignedID", "buyer_id")?;
        let name = self.leaf(RAM, "Name", "name")?.parse()?;
        let description = self.optional_leaf(RAM, "Description", "description")?;
        let mut attributes = Vec::new();
        while self.is_open(RAM, "ApplicableProductCharacteristic") {
            self.enter_structural(RAM, "ApplicableProductCharacteristic")?;
            let name = self.leaf(RAM, "Description", "attributes")?.parse()?;
            let value = self.leaf(RAM, "Value", "attributes")?.parse()?;
            self.leave_structural()?;
            attributes.push(ItemAttribute { name, value });
        }
        let mut classifications = Vec::new();
        while self.is_open(RAM, "DesignatedProductClassification") {
            self.enter_structural(RAM, "DesignatedProductClassification")?;
            let (class_attrs, id) = self.leaf_attr(RAM, "ClassCode", "classifications")?;
            let scheme = attr(&class_attrs, "listID")
                .ok_or_else(|| bad("a classification without a scheme"))?
                .parse()?;
            let version = match attr(&class_attrs, "listVersionID") {
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
        let country_of_origin = if self.is_open(RAM, "OriginTradeCountry") {
            self.enter_structural(RAM, "OriginTradeCountry")?;
            let code = self.leaf(RAM, "ID", "country_of_origin")?;
            self.leave_structural()?;
            Some(parse_country(&code)?)
        } else {
            None
        };

        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();

        Ok(Item {
            name,
            description,
            seller_id,
            buyer_id,
            standard_id,
            classifications,
            country_of_origin,
            attributes,
        })
    }

    // Parses the line trade agreement, returning the order line reference and the price.
    fn parse_agreement(&mut self) -> Result<(Option<NonEmptyString>, Price), Error> {
        self.enter_structural(RAM, "SpecifiedLineTradeAgreement")?;
        let order_line_reference = if self.is_open(RAM, "BuyerOrderReferencedDocument") {
            self.enter_structural(RAM, "BuyerOrderReferencedDocument")?;
            let reference = self.leaf(RAM, "LineID", "order_line_reference")?.parse()?;
            self.leave_structural()?;
            Some(reference)
        } else {
            None
        };

        self.enter_group(RAM, "GrossPriceProductTradePrice", "price")?;
        let gross = parse_decimal(&self.derived(RAM, "ChargeAmount")?.1)?;
        let base_quantity = if self.is_open(RAM, "BasisQuantity") {
            let (attributes, text) = self.derived(RAM, "BasisQuantity")?;
            Some(parse_quantity(&attributes, &text)?)
        } else {
            None
        };
        let discount = if self.is_open(RAM, "AppliedTradeAllowanceCharge") {
            self.enter_nested(RAM, "AppliedTradeAllowanceCharge")?;
            self.indicator()?;
            let amount = parse_decimal(&self.derived(RAM, "ActualAmount")?.1)?;
            self.leave_nested()?;
            Some(amount)
        } else {
            None
        };
        self.leave_group()?;

        self.enter_group(RAM, "NetPriceProductTradePrice", "price")?;
        self.derived(RAM, "ChargeAmount")?;
        if self.is_open(RAM, "BasisQuantity") {
            self.derived(RAM, "BasisQuantity")?;
        }
        self.leave_group()?;

        self.leave_structural()?;
        Ok((
            order_line_reference,
            Price {
                gross,
                discount,
                base_quantity,
            },
        ))
    }

    // Parses the line trade settlement.
    #[allow(clippy::type_complexity)]
    fn parse_line_settlement(
        &mut self,
    ) -> Result<
        (
            VatTreatment,
            Option<Period>,
            Vec<LineAdjustment>,
            Option<ObjectReference>,
            Option<NonEmptyString>,
        ),
        Error,
    > {
        self.enter_structural(RAM, "SpecifiedLineTradeSettlement")?;
        let vat = self.parse_line_tax()?;
        let period = if self.is_open(RAM, "BillingSpecifiedPeriod") {
            Some(self.billing_period("period")?)
        } else {
            None
        };
        let mut adjustments = Vec::new();
        while self.is_open(RAM, "SpecifiedTradeAllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_line_adjustment(instance)?);
        }
        self.enter_structural(RAM, "SpecifiedTradeSettlementLineMonetarySummation")?;
        self.derived(RAM, "LineTotalAmount")?;
        self.leave_structural()?;
        let object = if self.is_open(RAM, "AdditionalReferencedDocument") {
            self.enter_group(RAM, "AdditionalReferencedDocument", "object")?;
            let id = self.derived(RAM, "IssuerAssignedID")?.1.parse()?;
            self.derived(RAM, "TypeCode")?;
            let scheme = if self.is_open(RAM, "ReferenceTypeCode") {
                Some(self.derived(RAM, "ReferenceTypeCode")?.1.parse()?)
            } else {
                None
            };
            self.leave_group()?;
            Some(ObjectReference { id, scheme })
        } else {
            None
        };
        let buyer_accounting_reference =
            if self.is_open(RAM, "ReceivableSpecifiedTradeAccountingAccount") {
                self.enter_group(
                    RAM,
                    "ReceivableSpecifiedTradeAccountingAccount",
                    "buyer_accounting_reference",
                )?;
                let reference = self.derived(RAM, "ID")?.1.parse()?;
                self.leave_group()?;
                Some(reference)
            } else {
                None
            };
        self.leave_structural()?;
        Ok((vat, period, adjustments, object, buyer_accounting_reference))
    }

    fn parse_line_tax(&mut self) -> Result<VatTreatment, Error> {
        self.enter_group(RAM, "ApplicableTradeTax", "vat")?;
        self.derived(RAM, "TypeCode")?;
        let category: VatCategory = self.derived(RAM, "CategoryCode")?.1.parse()?;
        let rate = self.derived(RAM, "RateApplicablePercent")?.1.parse()?;
        self.leave_group()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    fn parse_line_adjustment(&mut self, instance: NonZeroUsize) -> Result<LineAdjustment, Error> {
        self.enter_repeatable(
            RAM,
            "SpecifiedTradeAllowanceCharge",
            "adjustments",
            instance,
        )?;
        let charge = self.indicator()?;
        let amount = self.parse_adjustment_amount()?;
        let reason = self.parse_reason(charge)?;
        self.leave_repeatable()?;
        Ok(LineAdjustment { amount, reason })
    }

    // Parses the header trade agreement.
    fn header_trade_agreement(&mut self) -> Result<Agreement, Error> {
        self.enter_structural(RAM, "ApplicableHeaderTradeAgreement")?;
        let buyer_reference = self.optional_leaf(RAM, "BuyerReference", "buyer_reference")?;
        let seller = self.parse_seller()?;
        let buyer = self.parse_buyer()?;
        let tax_representative = if self.is_open(RAM, "SellerTaxRepresentativeTradeParty") {
            Some(self.parse_tax_representative()?)
        } else {
            None
        };
        let sales_order_reference = self.optional_reference(
            RAM,
            "SellerOrderReferencedDocument",
            "sales_order_reference",
        )?;
        let purchase_order_reference = self.optional_reference(
            RAM,
            "BuyerOrderReferencedDocument",
            "purchase_order_reference",
        )?;
        let contract_reference =
            self.optional_reference(RAM, "ContractReferencedDocument", "contract_reference")?;
        let mut object = None;
        let mut tender_or_lot_reference = None;
        let mut supporting_documents = Vec::new();
        while self.is_open(RAM, "AdditionalReferencedDocument") {
            match self.parse_additional_document(supporting_documents.len())? {
                Additional::Object(reference) => object = Some(reference),
                Additional::Tender(reference) => tender_or_lot_reference = Some(reference),
                Additional::Supporting(document) => supporting_documents.push(document),
            }
        }
        let project_reference = if self.is_open(RAM, "SpecifiedProcuringProject") {
            self.enter_group(RAM, "SpecifiedProcuringProject", "project_reference")?;
            let id = self.derived(RAM, "ID")?.1.parse()?;
            self.derived(RAM, "Name")?;
            self.leave_group()?;
            Some(id)
        } else {
            None
        };
        self.leave_structural()?;
        Ok(Agreement {
            buyer_reference,
            seller,
            buyer,
            tax_representative,
            sales_order_reference,
            purchase_order_reference,
            contract_reference,
            object,
            tender_or_lot_reference,
            supporting_documents,
            project_reference,
        })
    }

    fn parse_additional_document(&mut self, supporting: usize) -> Result<Additional, Error> {
        let kind = self.additional_kind();
        match kind {
            AdditionalKind::Object => {
                self.enter_group(RAM, "AdditionalReferencedDocument", "object")?;
                let id = self.derived(RAM, "IssuerAssignedID")?.1.parse()?;
                self.derived(RAM, "TypeCode")?;
                let scheme = if self.is_open(RAM, "ReferenceTypeCode") {
                    Some(self.derived(RAM, "ReferenceTypeCode")?.1.parse()?)
                } else {
                    None
                };
                self.leave_group()?;
                Ok(Additional::Object(ObjectReference { id, scheme }))
            }
            AdditionalKind::Tender => {
                self.enter_group(
                    RAM,
                    "AdditionalReferencedDocument",
                    "tender_or_lot_reference",
                )?;
                let id = self.derived(RAM, "IssuerAssignedID")?.1.parse()?;
                self.derived(RAM, "TypeCode")?;
                self.leave_group()?;
                Ok(Additional::Tender(id))
            }
            AdditionalKind::Supporting => {
                let instance = index(supporting);
                self.enter_repeatable(
                    RAM,
                    "AdditionalReferencedDocument",
                    "supporting_documents",
                    instance,
                )?;
                let reference = self.derived(RAM, "IssuerAssignedID")?.1.parse()?;
                let external_location = if self.is_open(RAM, "URIID") {
                    Some(parse_url(&self.leaf(RAM, "URIID", "external_location")?)?)
                } else {
                    None
                };
                self.derived(RAM, "TypeCode")?;
                let description = self.optional_leaf(RAM, "Name", "description")?;
                self.leave_repeatable()?;
                Ok(Additional::Supporting(SupportingDocument {
                    reference,
                    description,
                    external_location,
                    attachment: None,
                }))
            }
        }
    }

    // Classifies the current additional referenced document by its fixed type code.
    fn additional_kind(&self) -> AdditionalKind {
        match self.peek_child_text("TypeCode").as_deref() {
            Some("130") => AdditionalKind::Object,
            Some("50") => AdditionalKind::Tender,
            _ => AdditionalKind::Supporting,
        }
    }

    fn parse_seller(&mut self) -> Result<Seller, Error> {
        self.enter_group(RAM, "SellerTradeParty", "seller")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.leaf(RAM, "Name", "name")?.parse()?;
        let additional_legal_information =
            self.optional_leaf(RAM, "Description", "additional_legal_information")?;
        let (legal_entity, trading_name) = self.parse_legal_organization()?;
        let contact = if self.is_open(RAM, "DefinedTradeContact") {
            Some(self.parse_contact()?)
        } else {
            None
        };
        let address = self.parse_address()?;
        let electronic_address = self.optional_electronic_address()?;
        let mut vat = None;
        let mut tax_registration = None;
        while self.is_open(RAM, "SpecifiedTaxRegistration") {
            match self.parse_tax_registration()? {
                TaxRegistration::Vat(value) => vat = Some(value),
                TaxRegistration::Other(value) => tax_registration = Some(value),
            }
        }
        self.leave_group()?;
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

    fn parse_buyer(&mut self) -> Result<Buyer, Error> {
        self.enter_group(RAM, "BuyerTradeParty", "buyer")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.leaf(RAM, "Name", "name")?.parse()?;
        let (legal_entity, trading_name) = self.parse_legal_organization()?;
        let contact = if self.is_open(RAM, "DefinedTradeContact") {
            Some(self.parse_contact()?)
        } else {
            None
        };
        let address = self.parse_address()?;
        let electronic_address = self.optional_electronic_address()?;
        let mut vat = None;
        while self.is_open(RAM, "SpecifiedTaxRegistration") {
            if let TaxRegistration::Vat(value) = self.parse_tax_registration()? {
                vat = Some(value);
            }
        }
        self.leave_group()?;
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

    fn parse_tax_representative(&mut self) -> Result<TaxRepresentative, Error> {
        self.enter_group(
            RAM,
            "SellerTaxRepresentativeTradeParty",
            "tax_representative",
        )?;
        let name = self.leaf(RAM, "Name", "name")?.parse()?;
        let address = self.parse_address()?;
        let vat = match self.parse_tax_registration()? {
            TaxRegistration::Vat(value) => value,
            TaxRegistration::Other(_) => return Err(bad("a tax representative without a VAT id")),
        };
        self.leave_group()?;
        Ok(TaxRepresentative { name, vat, address })
    }

    fn parse_identifiers(&mut self, field: &'static str) -> Result<Vec<OperationalEntity>, Error> {
        let mut identifiers = Vec::new();
        loop {
            if self.is_open(RAM, "ID") {
                let id = self.leaf(RAM, "ID", field)?.parse()?;
                identifiers.push(OperationalEntity { id, issuer: None });
            } else if self.is_open(RAM, "GlobalID") {
                let (attributes, id) = self.leaf_attr(RAM, "GlobalID", field)?;
                let issuer = attr(&attributes, "schemeID")
                    .map(|value| value.parse())
                    .transpose()?;
                identifiers.push(OperationalEntity {
                    id: id.parse()?,
                    issuer,
                });
            } else {
                break;
            }
        }
        Ok(identifiers)
    }

    fn parse_legal_organization(
        &mut self,
    ) -> Result<(Option<LegalEntity>, Option<NonEmptyString>), Error> {
        if !self.is_open(RAM, "SpecifiedLegalOrganization") {
            return Ok((None, None));
        }
        self.enter_structural(RAM, "SpecifiedLegalOrganization")?;
        let legal_entity = if self.is_open(RAM, "ID") {
            let (attributes, id) = self.leaf_attr(RAM, "ID", "legal_entity")?;
            let issuer = attr(&attributes, "schemeID")
                .map(|value| value.parse())
                .transpose()?;
            Some(LegalEntity {
                id: id.parse()?,
                issuer,
            })
        } else {
            None
        };
        let trading_name = self.optional_leaf(RAM, "TradingBusinessName", "trading_name")?;
        self.leave_structural()?;
        Ok((legal_entity, trading_name))
    }

    fn parse_contact(&mut self) -> Result<Contact, Error> {
        self.enter_group(RAM, "DefinedTradeContact", "contact")?;
        let name = self.optional_leaf(RAM, "PersonName", "name")?;
        let telephone = if self.is_open(RAM, "TelephoneUniversalCommunication") {
            self.enter_structural(RAM, "TelephoneUniversalCommunication")?;
            let number = self.leaf(RAM, "CompleteNumber", "telephone")?.parse()?;
            self.leave_structural()?;
            Some(number)
        } else {
            None
        };
        let email = if self.is_open(RAM, "EmailURIUniversalCommunication") {
            self.enter_structural(RAM, "EmailURIUniversalCommunication")?;
            let address = parse_email(&self.leaf(RAM, "URIID", "email")?)?;
            self.leave_structural()?;
            Some(address)
        } else {
            None
        };
        self.leave_group()?;
        Ok(Contact {
            name,
            telephone,
            email,
        })
    }

    fn parse_address(&mut self) -> Result<PostalAddress, Error> {
        self.enter_group(RAM, "PostalTradeAddress", "address")?;
        let postal_code = self.optional_leaf(RAM, "PostcodeCode", "postal_code")?;
        let line1 = self.optional_leaf(RAM, "LineOne", "line1")?;
        let line2 = self.optional_leaf(RAM, "LineTwo", "line2")?;
        let line3 = self.optional_leaf(RAM, "LineThree", "line3")?;
        let city = self.optional_leaf(RAM, "CityName", "city")?;
        let country = parse_country(&self.leaf(RAM, "CountryID", "country")?)?;
        let country_subdivision =
            self.optional_leaf(RAM, "CountrySubDivisionName", "country_subdivision")?;
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

    fn optional_electronic_address(&mut self) -> Result<Option<ElectronicAddress>, Error> {
        if !self.is_open(RAM, "URIUniversalCommunication") {
            return Ok(None);
        }
        self.enter_structural(RAM, "URIUniversalCommunication")?;
        let (attributes, id) = self.leaf_attr(RAM, "URIID", "electronic_address")?;
        let scheme = attr(&attributes, "schemeID")
            .ok_or_else(|| bad("an electronic address without a scheme"))?
            .parse()?;
        self.leave_structural()?;
        Ok(Some(ElectronicAddress {
            id: id.parse()?,
            scheme,
        }))
    }

    fn parse_tax_registration(&mut self) -> Result<TaxRegistration, Error> {
        // The serializer chose the field from the scheme, so peek it first.
        let scheme = self.peek_scheme_id();
        let field: &'static str = if scheme == "FC" {
            "tax_registration"
        } else {
            "vat"
        };
        self.enter_group(RAM, "SpecifiedTaxRegistration", field)?;
        let (_, id) = self.leaf_attr(RAM, "ID", field)?;
        self.leave_group()?;
        if scheme == "FC" {
            Ok(TaxRegistration::Other(id.parse()?))
        } else {
            Ok(TaxRegistration::Vat(id.parse()?))
        }
    }

    // Parses the header trade delivery.
    fn header_trade_delivery(&mut self) -> Result<DeliveryParts, Error> {
        if !self.is_open(RAM, "ApplicableHeaderTradeDelivery") {
            return Ok(DeliveryParts::default());
        }
        self.enter_structural(RAM, "ApplicableHeaderTradeDelivery")?;
        let mut ship_to = None;
        if self.is_open(RAM, "ShipToTradeParty") {
            ship_to = Some(self.parse_ship_to()?);
        }
        let date = if self.is_open(RAM, "ActualDeliverySupplyChainEvent") {
            self.enter_structural(RAM, "ActualDeliverySupplyChainEvent")?;
            let date = self.datetime("OccurrenceDateTime", "date")?;
            self.leave_structural()?;
            Some(date)
        } else {
            None
        };
        let despatch_advice_reference = self.optional_reference(
            RAM,
            "DespatchAdviceReferencedDocument",
            "despatch_advice_reference",
        )?;
        let receiving_advice_reference = self.optional_reference(
            RAM,
            "ReceivingAdviceReferencedDocument",
            "receiving_advice_reference",
        )?;
        self.leave_structural()?;

        let delivery = match (ship_to, date) {
            (Some((name, location, address)), date) => Some(Delivery {
                name,
                location,
                date,
                address,
            }),
            (None, Some(date)) => Some(Delivery {
                name: None,
                location: None,
                date: Some(date),
                address: None,
            }),
            (None, None) => None,
        };
        Ok(DeliveryParts {
            delivery,
            despatch_advice_reference,
            receiving_advice_reference,
        })
    }

    #[allow(clippy::type_complexity)]
    fn parse_ship_to(
        &mut self,
    ) -> Result<
        (
            Option<NonEmptyString>,
            Option<LocationReference>,
            Option<PostalAddress>,
        ),
        Error,
    > {
        self.enter_group(RAM, "ShipToTradeParty", "delivery")?;
        let location = if self.is_open(RAM, "ID") {
            let (attributes, id) = self.leaf_attr(RAM, "ID", "location")?;
            let issuer = attr(&attributes, "schemeID")
                .map(|value| value.parse())
                .transpose()?;
            Some(LocationReference {
                id: id.parse()?,
                issuer,
            })
        } else {
            None
        };
        let name = self.optional_leaf(RAM, "Name", "name")?;
        let address = if self.is_open(RAM, "PostalTradeAddress") {
            Some(self.parse_address()?)
        } else {
            None
        };
        self.leave_group()?;
        Ok((name, location, address))
    }

    // Parses the header trade settlement.
    fn header_trade_settlement(&mut self) -> Result<Settlement, Error> {
        self.enter_structural(RAM, "ApplicableHeaderTradeSettlement")?;
        let creditor = self.optional_leaf(RAM, "CreditorReferenceID", "creditor_identifier")?;
        let remittance = self.optional_leaf(RAM, "PaymentReference", "remittance_information")?;
        let accounting_currency = if self.is_open(RAM, "TaxCurrencyCode") {
            Some(parse_currency(&self.leaf(
                RAM,
                "TaxCurrencyCode",
                "vat_accounting_total",
            )?)?)
        } else {
            None
        };
        let currency = parse_currency(&self.leaf(RAM, "InvoiceCurrencyCode", "currency")?)?;
        let payee = if self.is_open(RAM, "PayeeTradeParty") {
            Some(self.parse_payee()?)
        } else {
            None
        };
        let (means, means_text, mut details) =
            if self.is_open(RAM, "SpecifiedTradeSettlementPaymentMeans") {
                let (means, text, details) = self.parse_payment_means()?;
                (Some(means), text, details)
            } else {
                (None, None, None)
            };

        let (exemptions, vat_point) = self.parse_tax_breakdown()?;
        let invoicing_period = if self.is_open(RAM, "BillingSpecifiedPeriod") {
            Some(self.billing_period("invoicing_period")?)
        } else {
            None
        };
        let mut adjustments = Vec::new();
        while self.is_open(RAM, "SpecifiedTradeAllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_adjustment(instance)?);
        }
        let (payment_terms, payment_due_date, mandate) =
            if self.is_open(RAM, "SpecifiedTradePaymentTerms") {
                self.parse_payment_terms()?
            } else {
                (None, None, None)
            };
        let (paid, rounding, accounting_value) = self.parse_monetary_summation()?;
        let mut preceding_invoices = Vec::new();
        while self.is_open(RAM, "InvoiceReferencedDocument") {
            let instance = index(preceding_invoices.len());
            preceding_invoices.push(self.parse_preceding_invoice(instance)?);
        }
        let buyer_accounting_reference =
            if self.is_open(RAM, "ReceivableSpecifiedTradeAccountingAccount") {
                self.enter_group(
                    RAM,
                    "ReceivableSpecifiedTradeAccountingAccount",
                    "buyer_accounting_reference",
                )?;
                let reference = self.derived(RAM, "ID")?.1.parse()?;
                self.leave_group()?;
                Some(reference)
            } else {
                None
            };
        self.leave_structural()?;

        // Inject the direct-debit creditor and mandate collected across the settlement.
        if let Some(PaymentDetails::DirectDebit(debit)) = &mut details {
            debit.creditor_identifier = creditor;
            debit.mandate_reference = mandate;
        }
        let payment = means.map(|means| PaymentInstructions {
            means,
            means_text,
            remittance_information: remittance,
            details,
        });
        let vat_accounting_total = match (accounting_currency, accounting_value) {
            (Some(currency), Some(value)) => Some(Amount { value, currency }),
            _ => None,
        };

        Ok(Settlement {
            currency,
            vat_accounting_total,
            vat_point,
            payment_due_date,
            payee,
            payment,
            exemptions,
            invoicing_period,
            adjustments,
            payment_terms,
            paid,
            rounding,
            preceding_invoices,
            buyer_accounting_reference,
        })
    }

    fn parse_payee(&mut self) -> Result<Payee, Error> {
        self.enter_group(RAM, "PayeeTradeParty", "payee")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.leaf(RAM, "Name", "name")?.parse()?;
        let legal_entity = if self.is_open(RAM, "SpecifiedLegalOrganization") {
            self.enter_structural(RAM, "SpecifiedLegalOrganization")?;
            let (attributes, id) = self.leaf_attr(RAM, "ID", "legal_entity")?;
            let issuer = attr(&attributes, "schemeID")
                .map(|value| value.parse())
                .transpose()?;
            self.leave_structural()?;
            Some(LegalEntity {
                id: id.parse()?,
                issuer,
            })
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

    #[allow(clippy::type_complexity)]
    fn parse_payment_means(
        &mut self,
    ) -> Result<
        (
            crate::PaymentMeans,
            Option<NonEmptyString>,
            Option<PaymentDetails>,
        ),
        Error,
    > {
        self.enter_group(RAM, "SpecifiedTradeSettlementPaymentMeans", "payment")?;
        let means = self.derived(RAM, "TypeCode")?.1.parse()?;
        let means_text = if self.is_open(RAM, "Information") {
            Some(self.derived(RAM, "Information")?.1.parse()?)
        } else {
            None
        };
        let details = if self.is_open(RAM, "ApplicableTradeSettlementFinancialCard") {
            self.enter_nested(RAM, "ApplicableTradeSettlementFinancialCard")?;
            let primary_account_number = self.derived(RAM, "ID")?.1.parse()?;
            let holder_name = if self.is_open(RAM, "CardholderName") {
                Some(self.derived(RAM, "CardholderName")?.1.parse()?)
            } else {
                None
            };
            self.leave_nested()?;
            Some(PaymentDetails::Card(PaymentCard {
                primary_account_number,
                holder_name,
            }))
        } else if self.is_open(RAM, "PayerPartyDebtorFinancialAccount") {
            self.enter_nested(RAM, "PayerPartyDebtorFinancialAccount")?;
            let account = self.derived(RAM, "IBANID")?.1.parse()?;
            self.leave_nested()?;
            Some(PaymentDetails::DirectDebit(DirectDebit {
                mandate_reference: None,
                creditor_identifier: None,
                debited_account: Some(account),
            }))
        } else if self.is_open(RAM, "PayeePartyCreditorFinancialAccount") {
            let mut transfers = Vec::new();
            while self.is_open(RAM, "PayeePartyCreditorFinancialAccount") {
                self.enter_nested(RAM, "PayeePartyCreditorFinancialAccount")?;
                let account = self.derived(RAM, "IBANID")?.1.parse()?;
                let account_name = if self.is_open(RAM, "AccountName") {
                    Some(self.derived(RAM, "AccountName")?.1.parse()?)
                } else {
                    None
                };
                self.leave_nested()?;
                let provider = if self.is_open(RAM, "PayeeSpecifiedCreditorFinancialInstitution") {
                    self.enter_nested(RAM, "PayeeSpecifiedCreditorFinancialInstitution")?;
                    let bic = self.derived(RAM, "BICID")?.1.parse()?;
                    self.leave_nested()?;
                    Some(bic)
                } else {
                    None
                };
                transfers.push(CreditTransfer {
                    account,
                    account_name,
                    provider,
                });
            }
            Some(PaymentDetails::CreditTransfers(transfers))
        } else {
            None
        };
        self.leave_group()?;
        Ok((means, means_text, details))
    }

    // Parses the VAT breakdown, collecting exemption reasons and the VAT point event.
    fn parse_tax_breakdown(&mut self) -> Result<(ExemptionMap, Option<VatPoint>), Error> {
        let mut exemptions = ExemptionMap::new();
        let mut vat_point = None;
        while self.is_open(RAM, "ApplicableTradeTax") {
            self.enter_structural(RAM, "ApplicableTradeTax")?;
            self.derived(RAM, "CalculatedAmount")?;
            self.derived(RAM, "TypeCode")?;
            let mut text = None;
            if self.is_open(RAM, "ExemptionReason") {
                text = Some(self.derived(RAM, "ExemptionReason")?.1.parse()?);
            }
            self.derived(RAM, "BasisAmount")?;
            let category: VatCategory = self.derived(RAM, "CategoryCode")?.1.parse()?;
            let mut code: Option<ExemptionReason> = None;
            if self.is_open(RAM, "ExemptionReasonCode") {
                code = Some(self.derived(RAM, "ExemptionReasonCode")?.1.parse()?);
            }
            if self.is_open(RAM, "DueDateTypeCode") {
                let event = self.derived(RAM, "DueDateTypeCode")?.1;
                vat_point = Some(VatPoint::Event(event.parse()?));
            }
            self.derived(RAM, "RateApplicablePercent")?;
            self.leave_structural()?;
            if category == VatCategory::Exempt {
                exemptions.set(code, text);
            }
        }
        Ok((exemptions, vat_point))
    }

    fn parse_adjustment(&mut self, instance: NonZeroUsize) -> Result<Adjustment, Error> {
        self.enter_repeatable(
            RAM,
            "SpecifiedTradeAllowanceCharge",
            "adjustments",
            instance,
        )?;
        let charge = self.indicator()?;
        let amount = self.parse_adjustment_amount()?;
        let reason = self.parse_reason(charge)?;
        self.enter_nested(RAM, "CategoryTradeTax")?;
        self.derived(RAM, "TypeCode")?;
        let category: VatCategory = self.derived(RAM, "CategoryCode")?.1.parse()?;
        let rate = self.derived(RAM, "RateApplicablePercent")?.1.parse()?;
        self.leave_nested()?;
        self.leave_repeatable()?;
        Ok(Adjustment {
            amount,
            vat: VatTreatment::from_category(category, rate),
            reason,
        })
    }

    fn parse_adjustment_amount(&mut self) -> Result<AdjustmentAmount, Error> {
        if self.is_open(RAM, "CalculationPercent") {
            let rate = self.derived(RAM, "CalculationPercent")?.1.parse()?;
            let base = parse_decimal(&self.derived(RAM, "BasisAmount")?.1)?;
            self.derived(RAM, "ActualAmount")?;
            Ok(AdjustmentAmount::Relative { rate, base })
        } else {
            let amount = parse_decimal(&self.derived(RAM, "ActualAmount")?.1)?;
            Ok(AdjustmentAmount::Absolute(amount))
        }
    }

    fn parse_reason(&mut self, charge: bool) -> Result<AdjustmentReason, Error> {
        let code = if self.is_open(RAM, "ReasonCode") {
            Some(self.derived(RAM, "ReasonCode")?.1)
        } else {
            None
        };
        let text = if self.is_open(RAM, "Reason") {
            Some(self.derived(RAM, "Reason")?.1.parse()?)
        } else {
            None
        };
        Ok(if charge {
            AdjustmentReason::Charge {
                code: code.map(|code| code.parse()).transpose()?,
                text,
            }
        } else {
            AdjustmentReason::Allowance {
                code: code.map(|code| code.parse()).transpose()?,
                text,
            }
        })
    }

    #[allow(clippy::type_complexity)]
    fn parse_payment_terms(
        &mut self,
    ) -> Result<(Option<NonEmptyString>, Option<Date>, Option<NonEmptyString>), Error> {
        self.enter_structural(RAM, "SpecifiedTradePaymentTerms")?;
        let terms = self.optional_leaf(RAM, "Description", "payment_terms")?;
        let due = if self.is_open(RAM, "DueDateDateTime") {
            Some(self.datetime("DueDateDateTime", "payment_due_date")?)
        } else {
            None
        };
        let mandate = self.optional_leaf(RAM, "DirectDebitMandateID", "mandate_reference")?;
        self.leave_structural()?;
        Ok((terms, due, mandate))
    }

    // Parses the header monetary summation, returning the paid and rounding amounts.
    #[allow(clippy::type_complexity)]
    fn parse_monetary_summation(
        &mut self,
    ) -> Result<(Option<Decimal>, Option<Decimal>, Option<Decimal>), Error> {
        self.enter_structural(RAM, "SpecifiedTradeSettlementHeaderMonetarySummation")?;
        let mut paid = None;
        let mut rounding = None;
        let mut tax_total = None;
        while self.is_open_namespace(RAM) {
            let (_, name) = self.head()?;
            let (_, text) = self.derived(RAM, &name)?;
            match name.as_str() {
                "TotalPrepaidAmount" => paid = Some(parse_decimal(&text)?),
                "RoundingAmount" => rounding = Some(parse_decimal(&text)?),
                "TaxTotalAmount" => tax_total = Some(parse_decimal(&text)?),
                _ => {}
            }
        }
        self.leave_structural()?;
        let _ = tax_total;
        Ok((paid, rounding, None))
    }

    fn parse_preceding_invoice(
        &mut self,
        instance: NonZeroUsize,
    ) -> Result<PrecedingInvoice, Error> {
        self.enter_repeatable(
            RAM,
            "InvoiceReferencedDocument",
            "preceding_invoices",
            instance,
        )?;
        let number = self.derived(RAM, "IssuerAssignedID")?.1.parse()?;
        let issue_date = if self.is_open(RAM, "FormattedIssueDateTime") {
            self.enter_nested(RAM, "FormattedIssueDateTime")?;
            let date = self.take_foreign_datetime()?;
            self.leave_nested()?;
            Some(parse_date(&date)?)
        } else {
            None
        };
        self.leave_repeatable()?;
        Ok(PrecedingInvoice { number, issue_date })
    }

    // Parses a billing period (`BG-14`/`BG-26`).
    fn billing_period(&mut self, field: &'static str) -> Result<Period, Error> {
        self.enter_group(RAM, "BillingSpecifiedPeriod", field)?;
        let start = if self.is_open(RAM, "StartDateTime") {
            Some(self.datetime("StartDateTime", field)?)
        } else {
            None
        };
        let end = if self.is_open(RAM, "EndDateTime") {
            Some(self.datetime("EndDateTime", field)?)
        } else {
            None
        };
        self.leave_group()?;
        period_from(start, end).ok_or_else(|| bad("an empty billing period"))
    }

    // Reads a single-identifier reference document when present.
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
        let id = self.derived(RAM, "IssuerAssignedID")?.1.parse()?;
        self.leave_group()?;
        Ok(Some(id))
    }

    // ---- datatype carriers ----------------------------------------------

    // Reads a date wrapper whose value carrier is a `udt`/`qdt` DateTimeString.
    fn datetime(&mut self, element: &str, field: &'static str) -> Result<Date, Error> {
        self.take_open(RAM, element)?;
        self.trace.enter(RAM, element);
        self.trace.push_field(field);
        self.trace.record_context();
        let text = self.take_foreign_datetime()?;
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        parse_date(&text)
    }

    // Reads a charge indicator whose value carrier is a `udt:Indicator`.
    fn indicator(&mut self) -> Result<bool, Error> {
        self.take_open(RAM, "ChargeIndicator")?;
        self.trace.enter(RAM, "ChargeIndicator");
        self.trace.record_context();
        let text = self.take_foreign()?;
        self.take_close()?;
        self.trace.leave();
        Ok(text.trim() == "true")
    }

    // Reads a `DateTimeString` value carrier from a datatype namespace, not recorded.
    fn take_foreign_datetime(&mut self) -> Result<String, Error> {
        self.take_foreign()
    }

    // Reads the text of a foreign datatype-carrier element, without a trace entry.
    fn take_foreign(&mut self) -> Result<String, Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace: None, ..
            }) => {
                self.cursor += 1;
                let text = self.take_text();
                self.take_close()?;
                Ok(text)
            }
            _ => Err(bad("expected a datatype carrier element")),
        }
    }

    // ---- mirror primitives ----------------------------------------------

    fn leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        Ok(self.leaf_attr(namespace, name, field)?.1)
    }

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

    fn rooted(&mut self, namespace: Namespace, name: &str) -> Result<String, Error> {
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_root();
        let text = self.take_text();
        self.take_close()?;
        self.trace.leave();
        Ok(text)
    }

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

    // Peeks the text of a direct child element by local name, without consuming.
    fn peek_child_text(&self, child: &str) -> Option<String> {
        let mut depth = 0usize;
        let mut cursor = self.cursor;
        while let Some(token) = self.tokens.get(cursor) {
            match token {
                Token::Open { name, .. } => {
                    if depth == 1 && name == child {
                        if let Some(Token::Text(text)) = self.tokens.get(cursor + 1) {
                            return Some(text.trim().to_owned());
                        }
                    }
                    depth += 1;
                }
                Token::Close => {
                    if depth <= 1 {
                        return None;
                    }
                    depth -= 1;
                }
                Token::Text(_) => {}
            }
            cursor += 1;
        }
        None
    }

    // Peeks the scheme id of the current tax registration, without consuming.
    fn peek_scheme_id(&self) -> String {
        if let Some(Token::Open { .. }) = self.tokens.get(self.cursor) {
            if let Some(Token::Open { attributes, .. }) = self.tokens.get(self.cursor + 1) {
                if let Some(value) = attr(attributes, "schemeID") {
                    return value.to_owned();
                }
            }
        }
        String::new()
    }

    fn head(&self) -> Result<(Namespace, String), Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace: Some(namespace),
                name,
                ..
            }) => Ok((*namespace, name.clone())),
            _ => Err(bad("expected an element")),
        }
    }

    fn is_open(&self, namespace: Namespace, name: &str) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: Some(found), name: local, .. }) if *found == namespace && local == name
        )
    }

    fn is_open_namespace(&self, namespace: Namespace) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: Some(found), .. }) if *found == namespace
        )
    }

    fn take_open(
        &mut self,
        namespace: Namespace,
        name: &str,
    ) -> Result<Vec<(String, String)>, Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace: Some(found),
                name: local,
                attributes,
            }) if *found == namespace && local == name => {
                let attributes = attributes.clone();
                self.cursor += 1;
                Ok(attributes)
            }
            _ => Err(Error::MalformedXml(format!("expected <{name}>"))),
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

// ---- collected parts -----------------------------------------------------

struct Agreement {
    buyer_reference: Option<NonEmptyString>,
    seller: Seller,
    buyer: Buyer,
    tax_representative: Option<TaxRepresentative>,
    sales_order_reference: Option<NonEmptyString>,
    purchase_order_reference: Option<NonEmptyString>,
    contract_reference: Option<NonEmptyString>,
    object: Option<ObjectReference>,
    tender_or_lot_reference: Option<NonEmptyString>,
    supporting_documents: Vec<SupportingDocument>,
    project_reference: Option<NonEmptyString>,
}

#[derive(Default)]
struct DeliveryParts {
    delivery: Option<Delivery>,
    despatch_advice_reference: Option<NonEmptyString>,
    receiving_advice_reference: Option<NonEmptyString>,
}

struct Settlement {
    currency: Currency,
    vat_accounting_total: Option<Amount>,
    vat_point: Option<VatPoint>,
    payment_due_date: Option<Date>,
    payee: Option<Payee>,
    payment: Option<PaymentInstructions>,
    exemptions: ExemptionMap,
    invoicing_period: Option<Period>,
    adjustments: Vec<Adjustment>,
    payment_terms: Option<NonEmptyString>,
    paid: Option<Decimal>,
    rounding: Option<Decimal>,
    preceding_invoices: Vec<PrecedingInvoice>,
    buyer_accounting_reference: Option<NonEmptyString>,
}

enum Additional {
    Object(ObjectReference),
    Tender(NonEmptyString),
    Supporting(SupportingDocument),
}

enum AdditionalKind {
    Object,
    Tender,
    Supporting,
}

enum TaxRegistration {
    Vat(crate::VatIdentifier),
    Other(NonEmptyString),
}

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

// ---- value helpers -------------------------------------------------------

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
        .ok_or_else(|| Error::MalformedXml(format!("invalid unit: {code}")))?;
    Ok(Quantity {
        unit,
        value: parse_decimal(text)?,
    })
}

fn parse_date(value: &str) -> Result<Date, Error> {
    let value = value.trim();
    if value.len() != 8 {
        return Err(Error::MalformedXml(format!("invalid date: {value}")));
    }
    let year = value[0..4].parse::<i32>().ok();
    let month = value[4..6]
        .parse::<u8>()
        .ok()
        .and_then(|month| Month::try_from(month).ok());
    let day = value[6..8].parse::<u8>().ok();
    match (year, month, day) {
        (Some(year), Some(month), Some(day)) => Date::from_calendar_date(year, month, day)
            .map_err(|_| Error::MalformedXml(format!("invalid date: {value}"))),
        _ => Err(Error::MalformedXml(format!("invalid date: {value}"))),
    }
}

fn parse_decimal(value: &str) -> Result<Decimal, Error> {
    value
        .trim()
        .parse::<Decimal>()
        .map_err(|_| Error::MalformedXml(format!("invalid decimal: {value}")))
}

fn parse_currency(value: &str) -> Result<Currency, Error> {
    Currency::from_code(value.trim())
        .ok_or_else(|| Error::MalformedXml(format!("invalid currency: {value}")))
}

fn parse_country(value: &str) -> Result<CountryCode, Error> {
    CountryCode::for_alpha2(value.trim())
        .map_err(|_| Error::MalformedXml(format!("invalid country: {value}")))
}

fn parse_email(value: &str) -> Result<EmailAddress, Error> {
    value
        .trim()
        .parse::<EmailAddress>()
        .map_err(|_| Error::MalformedXml(format!("invalid email: {value}")))
}

fn parse_url(value: &str) -> Result<Url, Error> {
    value
        .trim()
        .parse::<Url>()
        .map_err(|_| Error::MalformedXml(format!("invalid url: {value}")))
}

fn bad(message: &str) -> Error {
    Error::MalformedXml(message.to_owned())
}
