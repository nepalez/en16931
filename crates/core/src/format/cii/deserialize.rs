use crate::format::cii;
use crate::format::trace::Trace;
use crate::prelude::*;
use crate::{
    Abbreviations, Adjustment, AdjustmentAmount, AdjustmentReason, Amount, Buyer, Cii,
    Classification, Contact, CreditTransfer, Delivery, Dictionary, DirectDebit, DocumentBuilder,
    ElectronicAddress, Error, ExemptionReason, Format, Invoice, InvoiceLine, Item, ItemAttribute,
    ItemReference, LegalEntity, LineAdjustment, LocationReference, Namespace, NonEmptyString, Note,
    ObjectReference, OperationalEntity, Payee, PaymentCard, PaymentDetails, PaymentInstructions,
    Period, PostalAddress, PrecedingInvoice, Price, Quantity, Seller, SupportingDocument,
    TaxRepresentative, Unit, VatCategory, VatPoint, VatTreatment,
};

/// Parses a CII document from XML,
/// rebuilding the dictionary and the abbreviations on the inverse path.
pub(crate) fn deserialize(
    xml: &str,
) -> Result<
    (
        DocumentBuilder,
        Dictionary<cii::Namespace>,
        Abbreviations<cii::Namespace>,
    ),
    Error,
> {
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
        namespace: cii::Namespace,
        name: String,
        attributes: Vec<(String, String)>,
    },
    Text(String),
    Close,
}

fn tokenize(xml: &str) -> Result<(Vec<Token>, Abbreviations<cii::Namespace>), Error> {
    let mut reader = NsReader::from_str(xml);
    let mut tokens = Vec::new();
    let mut abbreviations = <Cii as Format>::Namespace::default_abbreviations();
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

fn open_token(
    resolved: ResolveResult<'_>,
    start: &BytesStart<'_>,
    abbreviations: &mut Abbreviations<cii::Namespace>,
) -> Result<Token, Error> {
    let ResolveResult::Bound(uri) = resolved else {
        return Err(Error::malformed_xml("an element has no namespace"));
    };
    let uri = String::from_utf8_lossy(uri.into_inner());
    let namespace = <Cii as Format>::Namespace::from_uri(&uri)
        .ok_or_else(|| Error::malformed_xml(format!("unknown namespace: {uri}")))?;
    let name = String::from_utf8_lossy(start.local_name().as_ref()).into_owned();
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        if key == b"xmlns" || key.starts_with(b"xmlns:") {
            let abbreviation = String::from_utf8_lossy(key.strip_prefix(b"xmlns:").unwrap_or(b""));
            let uri = String::from_utf8_lossy(&attribute.value);
            if let Some(namespace) = <Cii as Format>::Namespace::from_uri(&uri) {
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
    trace: Trace<cii::Namespace>,
}

impl Parser {
    fn document(&mut self) -> Result<DocumentBuilder, Error> {
        self.take_open(cii::Namespace::Rsm, "CrossIndustryInvoice")?;
        self.trace
            .enter(cii::Namespace::Rsm, "CrossIndustryInvoice");
        self.trace.record_root();

        let (profile, business_process) = self.exchanged_document_context()?;
        let (number, type_code, issue_date, notes) = self.exchanged_document()?;

        self.enter_structural(cii::Namespace::Rsm, "SupplyChainTradeTransaction")?;
        let mut lines = Vec::new();
        while self.is_open(cii::Namespace::Ram, "IncludedSupplyChainTradeLineItem") {
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
        self.enter_structural(cii::Namespace::Rsm, "ExchangedDocumentContext")?;
        let business_process = if self.is_open(
            cii::Namespace::Ram,
            "BusinessProcessSpecifiedDocumentContextParameter",
        ) {
            self.enter_structural(
                cii::Namespace::Ram,
                "BusinessProcessSpecifiedDocumentContextParameter",
            )?;
            let value = self.rooted(cii::Namespace::Ram, "ID")?.parse()?;
            self.leave_structural()?;
            Some(value)
        } else {
            None
        };
        self.enter_structural(
            cii::Namespace::Ram,
            "GuidelineSpecifiedDocumentContextParameter",
        )?;
        let profile = self.rooted(cii::Namespace::Ram, "ID")?.parse()?;
        self.leave_structural()?;
        self.leave_structural()?;
        Ok((profile, business_process))
    }

    // Parses the exchanged document header.
    fn exchanged_document(
        &mut self,
    ) -> Result<(NonEmptyString, crate::InvoiceType, Date, Vec<Note>), Error> {
        self.enter_structural(cii::Namespace::Rsm, "ExchangedDocument")?;
        let number = self.leaf(cii::Namespace::Ram, "ID", "number")?.parse()?;
        let type_code = self
            .leaf(cii::Namespace::Ram, "TypeCode", "type_code")?
            .parse()?;
        let issue_date = self.datetime("IssueDateTime", "issue_date")?;
        let mut notes = Vec::new();
        while self.is_open(cii::Namespace::Ram, "IncludedNote") {
            let instance = index(notes.len());
            self.enter_repeatable(cii::Namespace::Ram, "IncludedNote", "notes", instance)?;
            let text = self.derived(cii::Namespace::Ram, "Content")?.1.parse()?;
            let subject_code = if self.is_open(cii::Namespace::Ram, "SubjectCode") {
                Some(
                    self.derived(cii::Namespace::Ram, "SubjectCode")?
                        .1
                        .parse()?,
                )
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
        self.enter_repeatable(
            cii::Namespace::Ram,
            "IncludedSupplyChainTradeLineItem",
            "lines",
            instance,
        )?;

        self.enter_structural(cii::Namespace::Ram, "AssociatedDocumentLineDocument")?;
        let id = self.leaf(cii::Namespace::Ram, "LineID", "id")?.parse()?;
        let note = if self.is_open(cii::Namespace::Ram, "IncludedNote") {
            self.enter_group(cii::Namespace::Ram, "IncludedNote", "note")?;
            let content = self.derived(cii::Namespace::Ram, "Content")?.1.parse()?;
            self.leave_group()?;
            Some(content)
        } else {
            None
        };
        self.leave_structural()?;

        let item = self.parse_product()?;
        let (order_line_reference, price) = self.parse_agreement()?;

        self.enter_structural(cii::Namespace::Ram, "SpecifiedLineTradeDelivery")?;
        let (quantity_attrs, quantity_text) =
            self.leaf_attr(cii::Namespace::Ram, "BilledQuantity", "quantity")?;
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
        self.take_open(cii::Namespace::Ram, "SpecifiedTradeProduct")?;
        self.trace
            .enter(cii::Namespace::Ram, "SpecifiedTradeProduct");
        self.trace.push_field("item");
        self.trace.record_context();

        let standard_id = if self.is_open(cii::Namespace::Ram, "GlobalID") {
            let (attributes, id) =
                self.leaf_attr(cii::Namespace::Ram, "GlobalID", "standard_id")?;
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
        let seller_id = self.optional_leaf(cii::Namespace::Ram, "SellerAssignedID", "seller_id")?;
        let buyer_id = self.optional_leaf(cii::Namespace::Ram, "BuyerAssignedID", "buyer_id")?;
        let name = self.leaf(cii::Namespace::Ram, "Name", "name")?.parse()?;
        let description = self.optional_leaf(cii::Namespace::Ram, "Description", "description")?;
        let mut attributes = Vec::new();
        while self.is_open(cii::Namespace::Ram, "ApplicableProductCharacteristic") {
            self.enter_structural(cii::Namespace::Ram, "ApplicableProductCharacteristic")?;
            let name = self
                .leaf(cii::Namespace::Ram, "Description", "attributes")?
                .parse()?;
            let value = self
                .leaf(cii::Namespace::Ram, "Value", "attributes")?
                .parse()?;
            self.leave_structural()?;
            attributes.push(ItemAttribute { name, value });
        }
        let mut classifications = Vec::new();
        while self.is_open(cii::Namespace::Ram, "DesignatedProductClassification") {
            self.enter_structural(cii::Namespace::Ram, "DesignatedProductClassification")?;
            let (class_attrs, id) =
                self.leaf_attr(cii::Namespace::Ram, "ClassCode", "classifications")?;
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
        let country_of_origin = if self.is_open(cii::Namespace::Ram, "OriginTradeCountry") {
            self.enter_structural(cii::Namespace::Ram, "OriginTradeCountry")?;
            let code = self.leaf(cii::Namespace::Ram, "ID", "country_of_origin")?;
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
        self.enter_structural(cii::Namespace::Ram, "SpecifiedLineTradeAgreement")?;
        let order_line_reference =
            if self.is_open(cii::Namespace::Ram, "BuyerOrderReferencedDocument") {
                self.enter_structural(cii::Namespace::Ram, "BuyerOrderReferencedDocument")?;
                let reference = self
                    .leaf(cii::Namespace::Ram, "LineID", "order_line_reference")?
                    .parse()?;
                self.leave_structural()?;
                Some(reference)
            } else {
                None
            };

        self.enter_group(cii::Namespace::Ram, "GrossPriceProductTradePrice", "price")?;
        let gross = parse_decimal(&self.derived(cii::Namespace::Ram, "ChargeAmount")?.1)?;
        let base_quantity = if self.is_open(cii::Namespace::Ram, "BasisQuantity") {
            let (attributes, text) = self.derived(cii::Namespace::Ram, "BasisQuantity")?;
            Some(parse_quantity(&attributes, &text)?)
        } else {
            None
        };
        let discount = if self.is_open(cii::Namespace::Ram, "AppliedTradeAllowanceCharge") {
            self.enter_nested(cii::Namespace::Ram, "AppliedTradeAllowanceCharge")?;
            self.indicator()?;
            let amount = parse_decimal(&self.derived(cii::Namespace::Ram, "ActualAmount")?.1)?;
            self.leave_nested()?;
            Some(amount)
        } else {
            None
        };
        self.leave_group()?;

        self.enter_group(cii::Namespace::Ram, "NetPriceProductTradePrice", "price")?;
        self.derived(cii::Namespace::Ram, "ChargeAmount")?;
        if self.is_open(cii::Namespace::Ram, "BasisQuantity") {
            self.derived(cii::Namespace::Ram, "BasisQuantity")?;
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
        self.enter_structural(cii::Namespace::Ram, "SpecifiedLineTradeSettlement")?;
        let vat = self.parse_line_tax()?;
        let period = if self.is_open(cii::Namespace::Ram, "BillingSpecifiedPeriod") {
            Some(self.billing_period("period")?)
        } else {
            None
        };
        let mut adjustments = Vec::new();
        while self.is_open(cii::Namespace::Ram, "SpecifiedTradeAllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_line_adjustment(instance)?);
        }
        self.enter_structural(
            cii::Namespace::Ram,
            "SpecifiedTradeSettlementLineMonetarySummation",
        )?;
        self.derived(cii::Namespace::Ram, "LineTotalAmount")?;
        self.leave_structural()?;
        let object = if self.is_open(cii::Namespace::Ram, "AdditionalReferencedDocument") {
            self.enter_group(
                cii::Namespace::Ram,
                "AdditionalReferencedDocument",
                "object",
            )?;
            let id = self
                .derived(cii::Namespace::Ram, "IssuerAssignedID")?
                .1
                .parse()?;
            self.derived(cii::Namespace::Ram, "TypeCode")?;
            let scheme = if self.is_open(cii::Namespace::Ram, "ReferenceTypeCode") {
                Some(
                    self.derived(cii::Namespace::Ram, "ReferenceTypeCode")?
                        .1
                        .parse()?,
                )
            } else {
                None
            };
            self.leave_group()?;
            Some(ObjectReference { id, scheme })
        } else {
            None
        };
        let buyer_accounting_reference = if self.is_open(
            cii::Namespace::Ram,
            "ReceivableSpecifiedTradeAccountingAccount",
        ) {
            self.enter_group(
                cii::Namespace::Ram,
                "ReceivableSpecifiedTradeAccountingAccount",
                "buyer_accounting_reference",
            )?;
            let reference = self.derived(cii::Namespace::Ram, "ID")?.1.parse()?;
            self.leave_group()?;
            Some(reference)
        } else {
            None
        };
        self.leave_structural()?;
        Ok((vat, period, adjustments, object, buyer_accounting_reference))
    }

    fn parse_line_tax(&mut self) -> Result<VatTreatment, Error> {
        self.enter_group(cii::Namespace::Ram, "ApplicableTradeTax", "vat")?;
        self.derived(cii::Namespace::Ram, "TypeCode")?;
        let category: VatCategory = self
            .derived(cii::Namespace::Ram, "CategoryCode")?
            .1
            .parse()?;
        let rate = self
            .derived(cii::Namespace::Ram, "RateApplicablePercent")?
            .1
            .parse()?;
        self.leave_group()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    fn parse_line_adjustment(&mut self, instance: NonZeroUsize) -> Result<LineAdjustment, Error> {
        self.enter_repeatable(
            cii::Namespace::Ram,
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
        self.enter_structural(cii::Namespace::Ram, "ApplicableHeaderTradeAgreement")?;
        let buyer_reference =
            self.optional_leaf(cii::Namespace::Ram, "BuyerReference", "buyer_reference")?;
        let seller = self.parse_seller()?;
        let buyer = self.parse_buyer()?;
        let tax_representative =
            if self.is_open(cii::Namespace::Ram, "SellerTaxRepresentativeTradeParty") {
                Some(self.parse_tax_representative()?)
            } else {
                None
            };
        let sales_order_reference = self.optional_reference(
            cii::Namespace::Ram,
            "SellerOrderReferencedDocument",
            "sales_order_reference",
        )?;
        let purchase_order_reference = self.optional_reference(
            cii::Namespace::Ram,
            "BuyerOrderReferencedDocument",
            "purchase_order_reference",
        )?;
        let contract_reference = self.optional_reference(
            cii::Namespace::Ram,
            "ContractReferencedDocument",
            "contract_reference",
        )?;
        let mut object = None;
        let mut tender_or_lot_reference = None;
        let mut supporting_documents = Vec::new();
        while self.is_open(cii::Namespace::Ram, "AdditionalReferencedDocument") {
            match self.parse_additional_document(supporting_documents.len())? {
                Additional::Object(reference) => object = Some(reference),
                Additional::Tender(reference) => tender_or_lot_reference = Some(reference),
                Additional::Supporting(document) => supporting_documents.push(document),
            }
        }
        let project_reference = if self.is_open(cii::Namespace::Ram, "SpecifiedProcuringProject") {
            self.enter_group(
                cii::Namespace::Ram,
                "SpecifiedProcuringProject",
                "project_reference",
            )?;
            let id = self.derived(cii::Namespace::Ram, "ID")?.1.parse()?;
            self.derived(cii::Namespace::Ram, "Name")?;
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
                self.enter_group(
                    cii::Namespace::Ram,
                    "AdditionalReferencedDocument",
                    "object",
                )?;
                let id = self
                    .derived(cii::Namespace::Ram, "IssuerAssignedID")?
                    .1
                    .parse()?;
                self.derived(cii::Namespace::Ram, "TypeCode")?;
                let scheme = if self.is_open(cii::Namespace::Ram, "ReferenceTypeCode") {
                    Some(
                        self.derived(cii::Namespace::Ram, "ReferenceTypeCode")?
                            .1
                            .parse()?,
                    )
                } else {
                    None
                };
                self.leave_group()?;
                Ok(Additional::Object(ObjectReference { id, scheme }))
            }
            AdditionalKind::Tender => {
                self.enter_group(
                    cii::Namespace::Ram,
                    "AdditionalReferencedDocument",
                    "tender_or_lot_reference",
                )?;
                let id = self
                    .derived(cii::Namespace::Ram, "IssuerAssignedID")?
                    .1
                    .parse()?;
                self.derived(cii::Namespace::Ram, "TypeCode")?;
                self.leave_group()?;
                Ok(Additional::Tender(id))
            }
            AdditionalKind::Supporting => {
                let instance = index(supporting);
                self.enter_repeatable(
                    cii::Namespace::Ram,
                    "AdditionalReferencedDocument",
                    "supporting_documents",
                    instance,
                )?;
                let reference = self
                    .derived(cii::Namespace::Ram, "IssuerAssignedID")?
                    .1
                    .parse()?;
                let external_location = if self.is_open(cii::Namespace::Ram, "URIID") {
                    Some(parse_url(&self.leaf(
                        cii::Namespace::Ram,
                        "URIID",
                        "external_location",
                    )?)?)
                } else {
                    None
                };
                self.derived(cii::Namespace::Ram, "TypeCode")?;
                let description = self.optional_leaf(cii::Namespace::Ram, "Name", "description")?;
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
        self.enter_group(cii::Namespace::Ram, "SellerTradeParty", "seller")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.leaf(cii::Namespace::Ram, "Name", "name")?.parse()?;
        let additional_legal_information = self.optional_leaf(
            cii::Namespace::Ram,
            "Description",
            "additional_legal_information",
        )?;
        let (legal_entity, trading_name) = self.parse_legal_organization()?;
        let contact = if self.is_open(cii::Namespace::Ram, "DefinedTradeContact") {
            Some(self.parse_contact()?)
        } else {
            None
        };
        let address = self.parse_address()?;
        let electronic_address = self.optional_electronic_address()?;
        let mut vat = None;
        let mut tax_registration = None;
        while self.is_open(cii::Namespace::Ram, "SpecifiedTaxRegistration") {
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
        self.enter_group(cii::Namespace::Ram, "BuyerTradeParty", "buyer")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.leaf(cii::Namespace::Ram, "Name", "name")?.parse()?;
        let (legal_entity, trading_name) = self.parse_legal_organization()?;
        let contact = if self.is_open(cii::Namespace::Ram, "DefinedTradeContact") {
            Some(self.parse_contact()?)
        } else {
            None
        };
        let address = self.parse_address()?;
        let electronic_address = self.optional_electronic_address()?;
        let mut vat = None;
        while self.is_open(cii::Namespace::Ram, "SpecifiedTaxRegistration") {
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
            cii::Namespace::Ram,
            "SellerTaxRepresentativeTradeParty",
            "tax_representative",
        )?;
        let name = self.leaf(cii::Namespace::Ram, "Name", "name")?.parse()?;
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
            if self.is_open(cii::Namespace::Ram, "ID") {
                let id = self.leaf(cii::Namespace::Ram, "ID", field)?.parse()?;
                identifiers.push(OperationalEntity { id, issuer: None });
            } else if self.is_open(cii::Namespace::Ram, "GlobalID") {
                let (attributes, id) = self.leaf_attr(cii::Namespace::Ram, "GlobalID", field)?;
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
        if !self.is_open(cii::Namespace::Ram, "SpecifiedLegalOrganization") {
            return Ok((None, None));
        }
        self.enter_structural(cii::Namespace::Ram, "SpecifiedLegalOrganization")?;
        let legal_entity = if self.is_open(cii::Namespace::Ram, "ID") {
            let (attributes, id) = self.leaf_attr(cii::Namespace::Ram, "ID", "legal_entity")?;
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
        let trading_name =
            self.optional_leaf(cii::Namespace::Ram, "TradingBusinessName", "trading_name")?;
        self.leave_structural()?;
        Ok((legal_entity, trading_name))
    }

    fn parse_contact(&mut self) -> Result<Contact, Error> {
        self.enter_group(cii::Namespace::Ram, "DefinedTradeContact", "contact")?;
        let name = self.optional_leaf(cii::Namespace::Ram, "PersonName", "name")?;
        let telephone = if self.is_open(cii::Namespace::Ram, "TelephoneUniversalCommunication") {
            self.enter_structural(cii::Namespace::Ram, "TelephoneUniversalCommunication")?;
            let number = self
                .leaf(cii::Namespace::Ram, "CompleteNumber", "telephone")?
                .parse()?;
            self.leave_structural()?;
            Some(number)
        } else {
            None
        };
        let email = if self.is_open(cii::Namespace::Ram, "EmailURIUniversalCommunication") {
            self.enter_structural(cii::Namespace::Ram, "EmailURIUniversalCommunication")?;
            let address = parse_email(&self.leaf(cii::Namespace::Ram, "URIID", "email")?)?;
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
        self.enter_group(cii::Namespace::Ram, "PostalTradeAddress", "address")?;
        let postal_code = self.optional_leaf(cii::Namespace::Ram, "PostcodeCode", "postal_code")?;
        let line1 = self.optional_leaf(cii::Namespace::Ram, "LineOne", "line1")?;
        let line2 = self.optional_leaf(cii::Namespace::Ram, "LineTwo", "line2")?;
        let line3 = self.optional_leaf(cii::Namespace::Ram, "LineThree", "line3")?;
        let city = self.optional_leaf(cii::Namespace::Ram, "CityName", "city")?;
        let country = parse_country(&self.leaf(cii::Namespace::Ram, "CountryID", "country")?)?;
        let country_subdivision = self.optional_leaf(
            cii::Namespace::Ram,
            "CountrySubDivisionName",
            "country_subdivision",
        )?;
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
        if !self.is_open(cii::Namespace::Ram, "URIUniversalCommunication") {
            return Ok(None);
        }
        self.enter_structural(cii::Namespace::Ram, "URIUniversalCommunication")?;
        let (attributes, id) =
            self.leaf_attr(cii::Namespace::Ram, "URIID", "electronic_address")?;
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
        self.enter_group(cii::Namespace::Ram, "SpecifiedTaxRegistration", field)?;
        let (_, id) = self.leaf_attr(cii::Namespace::Ram, "ID", field)?;
        self.leave_group()?;
        if scheme == "FC" {
            Ok(TaxRegistration::Other(id.parse()?))
        } else {
            Ok(TaxRegistration::Vat(id.parse()?))
        }
    }

    // Parses the header trade delivery.
    fn header_trade_delivery(&mut self) -> Result<DeliveryParts, Error> {
        if !self.is_open(cii::Namespace::Ram, "ApplicableHeaderTradeDelivery") {
            return Ok(DeliveryParts::default());
        }
        self.enter_structural(cii::Namespace::Ram, "ApplicableHeaderTradeDelivery")?;
        let mut ship_to = None;
        if self.is_open(cii::Namespace::Ram, "ShipToTradeParty") {
            ship_to = Some(self.parse_ship_to()?);
        }
        let date = if self.is_open(cii::Namespace::Ram, "ActualDeliverySupplyChainEvent") {
            self.enter_structural(cii::Namespace::Ram, "ActualDeliverySupplyChainEvent")?;
            let date = self.datetime("OccurrenceDateTime", "date")?;
            self.leave_structural()?;
            Some(date)
        } else {
            None
        };
        let despatch_advice_reference = self.optional_reference(
            cii::Namespace::Ram,
            "DespatchAdviceReferencedDocument",
            "despatch_advice_reference",
        )?;
        let receiving_advice_reference = self.optional_reference(
            cii::Namespace::Ram,
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
        self.enter_group(cii::Namespace::Ram, "ShipToTradeParty", "delivery")?;
        let location = if self.is_open(cii::Namespace::Ram, "ID") {
            let (attributes, id) = self.leaf_attr(cii::Namespace::Ram, "ID", "location")?;
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
        let name = self.optional_leaf(cii::Namespace::Ram, "Name", "name")?;
        let address = if self.is_open(cii::Namespace::Ram, "PostalTradeAddress") {
            Some(self.parse_address()?)
        } else {
            None
        };
        self.leave_group()?;
        Ok((name, location, address))
    }

    // Parses the header trade settlement.
    fn header_trade_settlement(&mut self) -> Result<Settlement, Error> {
        self.enter_structural(cii::Namespace::Ram, "ApplicableHeaderTradeSettlement")?;
        let creditor = self.optional_leaf(
            cii::Namespace::Ram,
            "CreditorReferenceID",
            "creditor_identifier",
        )?;
        let remittance = self.optional_leaf(
            cii::Namespace::Ram,
            "PaymentReference",
            "remittance_information",
        )?;
        let accounting_currency = if self.is_open(cii::Namespace::Ram, "TaxCurrencyCode") {
            Some(parse_currency(&self.leaf(
                cii::Namespace::Ram,
                "TaxCurrencyCode",
                "vat_accounting_total",
            )?)?)
        } else {
            None
        };
        let currency =
            parse_currency(&self.leaf(cii::Namespace::Ram, "InvoiceCurrencyCode", "currency")?)?;
        let payee = if self.is_open(cii::Namespace::Ram, "PayeeTradeParty") {
            Some(self.parse_payee()?)
        } else {
            None
        };
        let (means, means_text, mut details) =
            if self.is_open(cii::Namespace::Ram, "SpecifiedTradeSettlementPaymentMeans") {
                let (means, text, details) = self.parse_payment_means()?;
                (Some(means), text, details)
            } else {
                (None, None, None)
            };

        let (exemptions, vat_point) = self.parse_tax_breakdown()?;
        let invoicing_period = if self.is_open(cii::Namespace::Ram, "BillingSpecifiedPeriod") {
            Some(self.billing_period("invoicing_period")?)
        } else {
            None
        };
        let mut adjustments = Vec::new();
        while self.is_open(cii::Namespace::Ram, "SpecifiedTradeAllowanceCharge") {
            let instance = index(adjustments.len());
            adjustments.push(self.parse_adjustment(instance)?);
        }
        let (payment_terms, payment_due_date, mandate) =
            if self.is_open(cii::Namespace::Ram, "SpecifiedTradePaymentTerms") {
                self.parse_payment_terms()?
            } else {
                (None, None, None)
            };
        let (paid, rounding, accounting_value) = self.parse_monetary_summation()?;
        let mut preceding_invoices = Vec::new();
        while self.is_open(cii::Namespace::Ram, "InvoiceReferencedDocument") {
            let instance = index(preceding_invoices.len());
            preceding_invoices.push(self.parse_preceding_invoice(instance)?);
        }
        let buyer_accounting_reference = if self.is_open(
            cii::Namespace::Ram,
            "ReceivableSpecifiedTradeAccountingAccount",
        ) {
            self.enter_group(
                cii::Namespace::Ram,
                "ReceivableSpecifiedTradeAccountingAccount",
                "buyer_accounting_reference",
            )?;
            let reference = self.derived(cii::Namespace::Ram, "ID")?.1.parse()?;
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
        self.enter_group(cii::Namespace::Ram, "PayeeTradeParty", "payee")?;
        let identifiers = self.parse_identifiers("identifiers")?;
        let name = self.leaf(cii::Namespace::Ram, "Name", "name")?.parse()?;
        let legal_entity = if self.is_open(cii::Namespace::Ram, "SpecifiedLegalOrganization") {
            self.enter_structural(cii::Namespace::Ram, "SpecifiedLegalOrganization")?;
            let (attributes, id) = self.leaf_attr(cii::Namespace::Ram, "ID", "legal_entity")?;
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
        self.enter_group(
            cii::Namespace::Ram,
            "SpecifiedTradeSettlementPaymentMeans",
            "payment",
        )?;
        let means = self.derived(cii::Namespace::Ram, "TypeCode")?.1.parse()?;
        let means_text = if self.is_open(cii::Namespace::Ram, "Information") {
            Some(
                self.derived(cii::Namespace::Ram, "Information")?
                    .1
                    .parse()?,
            )
        } else {
            None
        };
        let details = if self.is_open(
            cii::Namespace::Ram,
            "ApplicableTradeSettlementFinancialCard",
        ) {
            self.enter_nested(
                cii::Namespace::Ram,
                "ApplicableTradeSettlementFinancialCard",
            )?;
            let primary_account_number = self.derived(cii::Namespace::Ram, "ID")?.1.parse()?;
            let holder_name = if self.is_open(cii::Namespace::Ram, "CardholderName") {
                Some(
                    self.derived(cii::Namespace::Ram, "CardholderName")?
                        .1
                        .parse()?,
                )
            } else {
                None
            };
            self.leave_nested()?;
            Some(PaymentDetails::Card(PaymentCard {
                primary_account_number,
                holder_name,
            }))
        } else if self.is_open(cii::Namespace::Ram, "PayerPartyDebtorFinancialAccount") {
            self.enter_nested(cii::Namespace::Ram, "PayerPartyDebtorFinancialAccount")?;
            let account = self.derived(cii::Namespace::Ram, "IBANID")?.1.parse()?;
            self.leave_nested()?;
            Some(PaymentDetails::DirectDebit(DirectDebit {
                mandate_reference: None,
                creditor_identifier: None,
                debited_account: Some(account),
            }))
        } else if self.is_open(cii::Namespace::Ram, "PayeePartyCreditorFinancialAccount") {
            let mut transfers = Vec::new();
            while self.is_open(cii::Namespace::Ram, "PayeePartyCreditorFinancialAccount") {
                self.enter_nested(cii::Namespace::Ram, "PayeePartyCreditorFinancialAccount")?;
                let account = self.derived(cii::Namespace::Ram, "IBANID")?.1.parse()?;
                let account_name = if self.is_open(cii::Namespace::Ram, "AccountName") {
                    Some(
                        self.derived(cii::Namespace::Ram, "AccountName")?
                            .1
                            .parse()?,
                    )
                } else {
                    None
                };
                self.leave_nested()?;
                let provider = if self.is_open(
                    cii::Namespace::Ram,
                    "PayeeSpecifiedCreditorFinancialInstitution",
                ) {
                    self.enter_nested(
                        cii::Namespace::Ram,
                        "PayeeSpecifiedCreditorFinancialInstitution",
                    )?;
                    let bic = self.derived(cii::Namespace::Ram, "BICID")?.1.parse()?;
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
        while self.is_open(cii::Namespace::Ram, "ApplicableTradeTax") {
            self.enter_structural(cii::Namespace::Ram, "ApplicableTradeTax")?;
            self.derived(cii::Namespace::Ram, "CalculatedAmount")?;
            self.derived(cii::Namespace::Ram, "TypeCode")?;
            let mut text = None;
            if self.is_open(cii::Namespace::Ram, "ExemptionReason") {
                text = Some(
                    self.derived(cii::Namespace::Ram, "ExemptionReason")?
                        .1
                        .parse()?,
                );
            }
            self.derived(cii::Namespace::Ram, "BasisAmount")?;
            let category: VatCategory = self
                .derived(cii::Namespace::Ram, "CategoryCode")?
                .1
                .parse()?;
            let mut code: Option<ExemptionReason> = None;
            if self.is_open(cii::Namespace::Ram, "ExemptionReasonCode") {
                code = Some(
                    self.derived(cii::Namespace::Ram, "ExemptionReasonCode")?
                        .1
                        .parse()?,
                );
            }
            if self.is_open(cii::Namespace::Ram, "DueDateTypeCode") {
                let event = self.derived(cii::Namespace::Ram, "DueDateTypeCode")?.1;
                vat_point = Some(VatPoint::Event(event.parse()?));
            }
            self.derived(cii::Namespace::Ram, "RateApplicablePercent")?;
            self.leave_structural()?;
            if category == VatCategory::Exempt {
                exemptions.set(code, text);
            }
        }
        Ok((exemptions, vat_point))
    }

    fn parse_adjustment(&mut self, instance: NonZeroUsize) -> Result<Adjustment, Error> {
        self.enter_repeatable(
            cii::Namespace::Ram,
            "SpecifiedTradeAllowanceCharge",
            "adjustments",
            instance,
        )?;
        let charge = self.indicator()?;
        let amount = self.parse_adjustment_amount()?;
        let reason = self.parse_reason(charge)?;
        self.enter_nested(cii::Namespace::Ram, "CategoryTradeTax")?;
        self.derived(cii::Namespace::Ram, "TypeCode")?;
        let category: VatCategory = self
            .derived(cii::Namespace::Ram, "CategoryCode")?
            .1
            .parse()?;
        let rate = self
            .derived(cii::Namespace::Ram, "RateApplicablePercent")?
            .1
            .parse()?;
        self.leave_nested()?;
        self.leave_repeatable()?;
        Ok(Adjustment {
            amount,
            vat: VatTreatment::from_category(category, rate),
            reason,
        })
    }

    fn parse_adjustment_amount(&mut self) -> Result<AdjustmentAmount, Error> {
        if self.is_open(cii::Namespace::Ram, "CalculationPercent") {
            let rate = self
                .derived(cii::Namespace::Ram, "CalculationPercent")?
                .1
                .parse()?;
            let base = parse_decimal(&self.derived(cii::Namespace::Ram, "BasisAmount")?.1)?;
            self.derived(cii::Namespace::Ram, "ActualAmount")?;
            Ok(AdjustmentAmount::Relative { rate, base })
        } else {
            let amount = parse_decimal(&self.derived(cii::Namespace::Ram, "ActualAmount")?.1)?;
            Ok(AdjustmentAmount::Absolute(amount))
        }
    }

    fn parse_reason(&mut self, charge: bool) -> Result<AdjustmentReason, Error> {
        let code = if self.is_open(cii::Namespace::Ram, "ReasonCode") {
            Some(self.derived(cii::Namespace::Ram, "ReasonCode")?.1)
        } else {
            None
        };
        let text = if self.is_open(cii::Namespace::Ram, "Reason") {
            Some(self.derived(cii::Namespace::Ram, "Reason")?.1.parse()?)
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
        self.enter_structural(cii::Namespace::Ram, "SpecifiedTradePaymentTerms")?;
        let terms = self.optional_leaf(cii::Namespace::Ram, "Description", "payment_terms")?;
        let due = if self.is_open(cii::Namespace::Ram, "DueDateDateTime") {
            Some(self.datetime("DueDateDateTime", "payment_due_date")?)
        } else {
            None
        };
        let mandate = self.optional_leaf(
            cii::Namespace::Ram,
            "DirectDebitMandateID",
            "mandate_reference",
        )?;
        self.leave_structural()?;
        Ok((terms, due, mandate))
    }

    // Parses the header monetary summation, returning the paid and rounding amounts.
    #[allow(clippy::type_complexity)]
    fn parse_monetary_summation(
        &mut self,
    ) -> Result<(Option<Decimal>, Option<Decimal>, Option<Decimal>), Error> {
        self.enter_structural(
            cii::Namespace::Ram,
            "SpecifiedTradeSettlementHeaderMonetarySummation",
        )?;
        let mut paid = None;
        let mut rounding = None;
        let mut tax_total = None;
        while self.is_open_namespace(cii::Namespace::Ram) {
            let (_, name) = self.head()?;
            let (_, text) = self.derived(cii::Namespace::Ram, &name)?;
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
            cii::Namespace::Ram,
            "InvoiceReferencedDocument",
            "preceding_invoices",
            instance,
        )?;
        let number = self
            .derived(cii::Namespace::Ram, "IssuerAssignedID")?
            .1
            .parse()?;
        let issue_date = if self.is_open(cii::Namespace::Ram, "FormattedIssueDateTime") {
            self.enter_nested(cii::Namespace::Ram, "FormattedIssueDateTime")?;
            self.take_open(cii::Namespace::Qdt, "DateTimeString")?;
            let date = self.take_text();
            self.take_close()?;
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
        self.enter_group(cii::Namespace::Ram, "BillingSpecifiedPeriod", field)?;
        let start = if self.is_open(cii::Namespace::Ram, "StartDateTime") {
            Some(self.datetime("StartDateTime", field)?)
        } else {
            None
        };
        let end = if self.is_open(cii::Namespace::Ram, "EndDateTime") {
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
        namespace: cii::Namespace,
        element: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if !self.is_open(namespace, element) {
            return Ok(None);
        }
        self.enter_group(namespace, element, field)?;
        let id = self
            .derived(cii::Namespace::Ram, "IssuerAssignedID")?
            .1
            .parse()?;
        self.leave_group()?;
        Ok(Some(id))
    }

    // ---- datatype carriers ----------------------------------------------

    // Reads a date wrapper whose value carrier is a `udt:DateTimeString`.
    fn datetime(&mut self, element: &str, field: &'static str) -> Result<Date, Error> {
        self.take_open(cii::Namespace::Ram, element)?;
        self.trace.enter(cii::Namespace::Ram, element);
        self.trace.push_field(field);
        self.trace.record_context();
        self.take_open(cii::Namespace::Udt, "DateTimeString")?;
        let text = self.take_text();
        self.take_close()?;
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        parse_date(&text)
    }

    // Reads a charge indicator whose value carrier is a `udt:Indicator`.
    fn indicator(&mut self) -> Result<bool, Error> {
        self.take_open(cii::Namespace::Ram, "ChargeIndicator")?;
        self.trace.enter(cii::Namespace::Ram, "ChargeIndicator");
        self.trace.record_context();
        self.take_open(cii::Namespace::Udt, "Indicator")?;
        let text = self.take_text();
        self.take_close()?;
        self.take_close()?;
        self.trace.leave();
        Ok(text.trim() == "true")
    }

    // ---- mirror primitives ----------------------------------------------

    fn leaf(
        &mut self,
        namespace: cii::Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        Ok(self.leaf_attr(namespace, name, field)?.1)
    }

    fn leaf_attr(
        &mut self,
        namespace: cii::Namespace,
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
        namespace: cii::Namespace,
        name: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf(namespace, name, field)?.parse()?))
        } else {
            Ok(None)
        }
    }

    fn rooted(&mut self, namespace: cii::Namespace, name: &str) -> Result<String, Error> {
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
        namespace: cii::Namespace,
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

    fn enter_structural(&mut self, namespace: cii::Namespace, name: &str) -> Result<(), Error> {
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

    fn enter_nested(&mut self, namespace: cii::Namespace, name: &str) -> Result<(), Error> {
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
        namespace: cii::Namespace,
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
        namespace: cii::Namespace,
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

    fn head(&self) -> Result<(cii::Namespace, String), Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace, name, ..
            }) => Ok((*namespace, name.clone())),
            _ => Err(bad("expected an element")),
        }
    }

    fn is_open(&self, namespace: cii::Namespace, name: &str) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, name: local, .. }) if *found == namespace && local == name
        )
    }

    fn is_open_namespace(&self, namespace: cii::Namespace) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, .. }) if *found == namespace
        )
    }

    fn take_open(
        &mut self,
        namespace: cii::Namespace,
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
        .ok_or_else(|| Error::malformed_xml(format!("invalid unit: {code}")))?;
    Ok(Quantity {
        unit,
        value: parse_decimal(text)?,
    })
}

fn parse_date(value: &str) -> Result<Date, Error> {
    let value = value.trim();
    if value.len() != 8 {
        return Err(Error::malformed_xml(format!("invalid date: {value}")));
    }
    let year = value[0..4].parse::<i32>().ok();
    let month = value[4..6]
        .parse::<u8>()
        .ok()
        .and_then(|month| Month::try_from(month).ok());
    let day = value[6..8].parse::<u8>().ok();
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
