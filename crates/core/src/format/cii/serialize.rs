use crate::format::cii::{QDT_PREFIX, QDT_URI, RAM, RSM, UDT_PREFIX, UDT_URI, prefix};
use crate::format::trace::Trace;
use crate::prelude::*;
use crate::{
    Abbreviations, Adjustment, AdjustmentAmount, AdjustmentReason, Buyer, Contact, Delivery,
    Dictionary, DocumentBuilder, ElectronicAddress, Invoice, InvoiceLine, Item, LegalEntity,
    LineAdjustment, Namespace, NonEmptyString, OperationalEntity, Payee, PaymentDetails,
    PaymentInstructions, Period, PostalAddress, PrecedingInvoice, Seller, TaxRepresentative, Term,
    VatPoint, VatTreatment,
};

/// Serializes a `DocumentBuilder` to CII XML,
/// returning the document, its dictionary, and its abbreviations.
pub(crate) fn serialize(builder: &DocumentBuilder) -> (String, Dictionary, Abbreviations) {
    let mut serializer = Serializer::new(builder);
    serializer.document(builder);
    serializer.finish()
}

// Renders a date as an ISO-8601 basic date (`YYYYMMDD`), the CII form 102.
fn date(value: Date) -> String {
    format!(
        "{:04}{:02}{:02}",
        value.year(),
        u8::from(value.month()),
        value.day()
    )
}

// Renders a monetary amount with two fraction digits, the EN-16931 form.
fn money(value: Decimal) -> String {
    format!("{value:.2}")
}

// Renders a plain decimal (quantity, rate, factor) with its own scale.
fn plain(value: Decimal) -> String {
    value.to_string()
}

/// The stateful CII writer: an XML sink plus the trace that builds the dictionary.
struct Serializer {
    inner: Writer<Vec<u8>>,
    trace: Trace,
    abbreviations: Abbreviations,
    forbidden: &'static [Term],
    currency: &'static str,
}

impl Serializer {
    fn new(builder: &DocumentBuilder) -> Self {
        Self {
            inner: Writer::new(Vec::new()),
            trace: Trace::new(),
            abbreviations: Abbreviations::default(),
            forbidden: builder.profile.forbidden_terms(),
            currency: builder.invoice.currency.code(),
        }
    }

    fn finish(self) -> (String, Dictionary, Abbreviations) {
        let xml = String::from_utf8(self.inner.into_inner()).expect("quick-xml emits valid UTF-8");
        (xml, self.trace.into_dictionary(), self.abbreviations)
    }

    fn is_forbidden(&self, term: Term) -> bool {
        self.forbidden.contains(&term)
    }

    // Serializes the whole document under the CII root element.
    fn document(&mut self, builder: &DocumentBuilder) {
        let invoice = &builder.invoice;
        let root = BytesStart::new("rsm:CrossIndustryInvoice").with_attributes([
            ("xmlns:rsm", RSM.uri()),
            ("xmlns:ram", RAM.uri()),
            ("xmlns:udt", UDT_URI),
            ("xmlns:qdt", QDT_URI),
        ]);
        self.write(Event::Start(root));
        // The datatype carriers stay out: no record-form path names them.
        for namespace in [RSM, RAM] {
            self.abbreviations
                .declare(prefix(namespace), namespace)
                .expect("the writer binds each abbreviation to one namespace");
        }
        self.trace.enter(RSM, "CrossIndustryInvoice");
        self.trace.record_root();

        self.exchanged_document_context(builder);
        self.exchanged_document(invoice);
        self.structural(RSM, "SupplyChainTradeTransaction", |serializer| {
            serializer.lines(&invoice.lines);
            serializer.header_trade_agreement(invoice);
            serializer.header_trade_delivery(invoice);
            serializer.header_trade_settlement(invoice);
        });

        self.trace.leave();
        self.write(Event::End(BytesEnd::new("rsm:CrossIndustryInvoice")));
    }

    // Serializes the document context: the business process (BT-23) and the profile (BT-24).
    fn exchanged_document_context(&mut self, builder: &DocumentBuilder) {
        self.structural(RSM, "ExchangedDocumentContext", |serializer| {
            if let Some(process) = &builder.business_process {
                serializer.structural(
                    RAM,
                    "BusinessProcessSpecifiedDocumentContextParameter",
                    |serializer| {
                        serializer.rooted(RAM, "ID", &[], process.as_ref());
                    },
                );
            }
            serializer.structural(
                RAM,
                "GuidelineSpecifiedDocumentContextParameter",
                |serializer| {
                    serializer.rooted(RAM, "ID", &[], &builder.profile.to_string());
                },
            );
        });
    }

    // Serializes the exchanged document header: number, type, date, and notes.
    fn exchanged_document(&mut self, invoice: &Invoice) {
        self.structural(RSM, "ExchangedDocument", |serializer| {
            serializer.leaf(RAM, "ID", "number", Term::BT(1), invoice.number.as_ref());
            serializer.leaf(
                RAM,
                "TypeCode",
                "type_code",
                Term::BT(3),
                &invoice.type_code.to_string(),
            );
            serializer.datetime(
                "IssueDateTime",
                "issue_date",
                Term::BT(2),
                invoice.issue_date,
            );
            for (position, note) in invoice.notes.iter().enumerate() {
                let instance = index(position);
                serializer.repeatable(
                    RAM,
                    "IncludedNote",
                    "notes",
                    Term::BG(1),
                    instance,
                    |serializer| {
                        serializer.derived(RAM, "Content", &[], note.text.as_ref());
                        if let Some(code) = &note.subject_code {
                            if !serializer.is_forbidden(Term::BT(21)) {
                                serializer.derived(RAM, "SubjectCode", &[], code.as_ref());
                            }
                        }
                    },
                );
            }
        });
    }

    // Serializes the invoice lines (`BG-25`).
    fn lines(&mut self, lines: &[InvoiceLine]) {
        for (position, line) in lines.iter().enumerate() {
            let instance = index(position);
            self.repeatable(
                RAM,
                "IncludedSupplyChainTradeLineItem",
                "lines",
                Term::BG(25),
                instance,
                |serializer| {
                    serializer.line(line);
                },
            );
        }
    }

    // Serializes one invoice line body.
    fn line(&mut self, line: &InvoiceLine) {
        self.structural(RAM, "AssociatedDocumentLineDocument", |serializer| {
            serializer.leaf(RAM, "LineID", "id", Term::BT(126), line.id.as_ref());
            if let Some(note) = &line.note {
                serializer.group(RAM, "IncludedNote", "note", Term::BT(127), |serializer| {
                    serializer.derived(RAM, "Content", &[], note.as_ref());
                });
            }
        });
        self.line_product(&line.item, &line.vat);
        self.line_agreement(line);
        self.structural(RAM, "SpecifiedLineTradeDelivery", |serializer| {
            serializer.leaf_attr(
                RAM,
                "BilledQuantity",
                "quantity",
                Term::BT(129),
                &[("unitCode", line.quantity.unit.code())],
                &plain(line.quantity.value),
            );
        });
        self.line_settlement(line);
    }

    // Serializes the line product (`BG-31`), rebasing the tax onto the line VAT elsewhere.
    fn line_product(&mut self, item: &Item, _vat: &VatTreatment) {
        self.trace.enter(RAM, "SpecifiedTradeProduct");
        self.trace.push_field("item");
        self.trace.record_context();
        self.write_start(RAM, "SpecifiedTradeProduct");

        if let Some(standard) = &item.standard_id {
            self.field_leaf_attr(
                RAM,
                "GlobalID",
                "standard_id",
                &[("schemeID", &standard.issuer.to_string())],
                standard.id.as_ref(),
            );
        }
        if let Some(id) = &item.seller_id {
            self.field_leaf(RAM, "SellerAssignedID", "seller_id", id.as_ref());
        }
        if let Some(id) = &item.buyer_id {
            self.field_leaf(RAM, "BuyerAssignedID", "buyer_id", id.as_ref());
        }
        self.field_leaf(RAM, "Name", "name", item.name.as_ref());
        if let Some(description) = &item.description {
            self.field_leaf(RAM, "Description", "description", description.as_ref());
        }
        for attribute in &item.attributes {
            self.structural(RAM, "ApplicableProductCharacteristic", |serializer| {
                serializer.field_leaf(RAM, "Description", "attributes", attribute.name.as_ref());
                serializer.field_leaf(RAM, "Value", "attributes", attribute.value.as_ref());
            });
        }
        for classification in &item.classifications {
            self.structural(RAM, "DesignatedProductClassification", |serializer| {
                let mut attributes = vec![("listID".to_owned(), classification.scheme.to_string())];
                if let Some(version) = &classification.version {
                    attributes.push(("listVersionID".to_owned(), version.as_ref().to_owned()));
                }
                let borrowed: Vec<(&str, &str)> = attributes
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str()))
                    .collect();
                serializer.field_leaf_attr(
                    RAM,
                    "ClassCode",
                    "classifications",
                    &borrowed,
                    classification.id.as_ref(),
                );
            });
        }
        if let Some(country) = item.country_of_origin {
            self.structural(RAM, "OriginTradeCountry", |serializer| {
                serializer.field_leaf(RAM, "ID", "country_of_origin", country.alpha2());
            });
        }

        self.write_end(RAM, "SpecifiedTradeProduct");
        self.trace.pop_context();
        self.trace.leave();
    }

    // Serializes the line trade agreement: order line reference and prices.
    fn line_agreement(&mut self, line: &InvoiceLine) {
        self.structural(RAM, "SpecifiedLineTradeAgreement", |serializer| {
            if let Some(order) = &line.order_line_reference {
                serializer.structural(RAM, "BuyerOrderReferencedDocument", |serializer| {
                    serializer.leaf(
                        RAM,
                        "LineID",
                        "order_line_reference",
                        Term::BT(132),
                        order.as_ref(),
                    );
                });
            }
            serializer.field_group(RAM, "GrossPriceProductTradePrice", "price", |serializer| {
                serializer.derived(RAM, "ChargeAmount", &[], &money(line.price.gross));
                if let Some(base) = line.price.base_quantity {
                    serializer.derived(
                        RAM,
                        "BasisQuantity",
                        &[("unitCode", base.unit.code())],
                        &plain(base.value),
                    );
                }
                if let Some(discount) = line.price.discount {
                    serializer.nested(RAM, "AppliedTradeAllowanceCharge", |serializer| {
                        serializer.indicator(false);
                        serializer.derived(RAM, "ActualAmount", &[], &money(discount));
                    });
                }
            });
            serializer.field_group(RAM, "NetPriceProductTradePrice", "price", |serializer| {
                serializer.derived(RAM, "ChargeAmount", &[], &money(line.price.net()));
                if let Some(base) = line.price.base_quantity {
                    serializer.derived(
                        RAM,
                        "BasisQuantity",
                        &[("unitCode", base.unit.code())],
                        &plain(base.value),
                    );
                }
            });
        });
    }

    // Serializes the line trade settlement: tax, period, adjustments, totals, object, account.
    fn line_settlement(&mut self, line: &InvoiceLine) {
        self.structural(RAM, "SpecifiedLineTradeSettlement", |serializer| {
            serializer.line_tax(&line.vat);
            if let Some(period) = line.period {
                serializer.billing_period(period, "period", Term::BG(26));
            }
            for (position, adjustment) in line.adjustments.iter().enumerate() {
                let instance = index(position);
                serializer.line_adjustment(adjustment, instance);
            }
            serializer.structural(
                RAM,
                "SpecifiedTradeSettlementLineMonetarySummation",
                |serializer| {
                    serializer.derived(RAM, "LineTotalAmount", &[], &money(line.net_amount()));
                },
            );
            if let Some(object) = &line.object {
                serializer.field_group(
                    RAM,
                    "AdditionalReferencedDocument",
                    "object",
                    |serializer| {
                        serializer.derived(RAM, "IssuerAssignedID", &[], object.id.as_ref());
                        serializer.derived(RAM, "TypeCode", &[], "130");
                        if let Some(scheme) = &object.scheme {
                            serializer.derived(RAM, "ReferenceTypeCode", &[], &scheme.to_string());
                        }
                    },
                );
            }
            if let Some(reference) = &line.buyer_accounting_reference {
                serializer.field_group(
                    RAM,
                    "ReceivableSpecifiedTradeAccountingAccount",
                    "buyer_accounting_reference",
                    |serializer| {
                        serializer.derived(RAM, "ID", &[], reference.as_ref());
                    },
                );
            }
        });
    }

    // Serializes the line VAT as a CII applicable trade tax, mapped to the line VAT.
    fn line_tax(&mut self, vat: &VatTreatment) {
        self.field_group(RAM, "ApplicableTradeTax", "vat", |serializer| {
            serializer.derived(RAM, "TypeCode", &[], "VAT");
            serializer.derived(RAM, "CategoryCode", &[], &vat.category().to_string());
            serializer.derived(RAM, "RateApplicablePercent", &[], &plain(vat.rate()));
        });
    }

    // Serializes one line-level allowance or charge.
    fn line_adjustment(&mut self, adjustment: &LineAdjustment, instance: NonZeroUsize) {
        let charge = matches!(adjustment.reason, AdjustmentReason::Charge { .. });
        let term = if charge { Term::BG(28) } else { Term::BG(27) };
        self.repeatable(
            RAM,
            "SpecifiedTradeAllowanceCharge",
            "adjustments",
            term,
            instance,
            |serializer| {
                serializer.indicator(charge);
                serializer.adjustment_amount(&adjustment.amount);
                serializer.adjustment_reason(&adjustment.reason);
            },
        );
    }

    // Serializes the header trade agreement (`BG-4`, `BG-7`, references, `BG-11`).
    fn header_trade_agreement(&mut self, invoice: &Invoice) {
        self.structural(RAM, "ApplicableHeaderTradeAgreement", |serializer| {
            if let Some(reference) = &invoice.buyer_reference {
                serializer.leaf(
                    RAM,
                    "BuyerReference",
                    "buyer_reference",
                    Term::BT(10),
                    reference.as_ref(),
                );
            }
            serializer.seller_party(&invoice.seller);
            serializer.buyer_party(&invoice.buyer);
            if let Some(representative) = &invoice.tax_representative {
                serializer.tax_representative_party(representative);
            }
            if let Some(sales) = &invoice.sales_order_reference {
                serializer.reference(
                    RAM,
                    "SellerOrderReferencedDocument",
                    Term::BT(14),
                    "sales_order_reference",
                    sales.as_ref(),
                );
            }
            if let Some(order) = &invoice.purchase_order_reference {
                serializer.reference(
                    RAM,
                    "BuyerOrderReferencedDocument",
                    Term::BT(13),
                    "purchase_order_reference",
                    order.as_ref(),
                );
            }
            if let Some(contract) = &invoice.contract_reference {
                serializer.reference(
                    RAM,
                    "ContractReferencedDocument",
                    Term::BT(12),
                    "contract_reference",
                    contract.as_ref(),
                );
            }
            serializer.additional_documents(invoice);
            if let Some(project) = &invoice.project_reference {
                serializer.group(
                    RAM,
                    "SpecifiedProcuringProject",
                    "project_reference",
                    Term::BT(11),
                    |serializer| {
                        serializer.derived(RAM, "ID", &[], project.as_ref());
                        serializer.derived(RAM, "Name", &[], "Project");
                    },
                );
            }
        });
    }

    // Serializes the invoiced object (`BT-18`), tender (`BT-17`), and supporting documents (`BG-24`).
    fn additional_documents(&mut self, invoice: &Invoice) {
        if let Some(object) = &invoice.object {
            self.group(
                RAM,
                "AdditionalReferencedDocument",
                "object",
                Term::BT(18),
                |serializer| {
                    serializer.derived(RAM, "IssuerAssignedID", &[], object.id.as_ref());
                    serializer.derived(RAM, "TypeCode", &[], "130");
                    if let Some(scheme) = &object.scheme {
                        serializer.derived(RAM, "ReferenceTypeCode", &[], &scheme.to_string());
                    }
                },
            );
        }
        if let Some(tender) = &invoice.tender_or_lot_reference {
            self.group(
                RAM,
                "AdditionalReferencedDocument",
                "tender_or_lot_reference",
                Term::BT(17),
                |serializer| {
                    serializer.derived(RAM, "IssuerAssignedID", &[], tender.as_ref());
                    serializer.derived(RAM, "TypeCode", &[], "50");
                },
            );
        }
        for (position, document) in invoice.supporting_documents.iter().enumerate() {
            let instance = index(position);
            self.repeatable(
                RAM,
                "AdditionalReferencedDocument",
                "supporting_documents",
                Term::BG(24),
                instance,
                |serializer| {
                    serializer.derived(RAM, "IssuerAssignedID", &[], document.reference.as_ref());
                    if let Some(location) = &document.external_location {
                        if !serializer.is_forbidden(Term::BT(124)) {
                            serializer.field_leaf(
                                RAM,
                                "URIID",
                                "external_location",
                                location.as_str(),
                            );
                        }
                    }
                    serializer.derived(RAM, "TypeCode", &[], "916");
                    if let Some(description) = &document.description {
                        serializer.field_leaf(RAM, "Name", "description", description.as_ref());
                    }
                },
            );
        }
    }

    // Serializes the seller party (`BG-4`).
    fn seller_party(&mut self, seller: &Seller) {
        self.group(
            RAM,
            "SellerTradeParty",
            "seller",
            Term::BG(4),
            |serializer| {
                serializer.party_identifiers(&seller.identifiers, "identifiers");
                serializer.field_leaf(RAM, "Name", "name", seller.name.as_ref());
                if let Some(info) = &seller.additional_legal_information {
                    serializer.field_leaf(
                        RAM,
                        "Description",
                        "additional_legal_information",
                        info.as_ref(),
                    );
                }
                serializer.legal_organization(
                    seller.legal_entity.as_ref(),
                    seller.trading_name.as_deref_ref(),
                );
                if let Some(contact) = &seller.contact {
                    serializer.contact(contact);
                }
                serializer.postal_address(&seller.address);
                if let Some(address) = &seller.electronic_address {
                    serializer.electronic_address(address);
                }
                if let Some(vat) = &seller.vat {
                    serializer.tax_registration(&vat.to_string(), "VA", "vat");
                }
                if let Some(registration) = &seller.tax_registration {
                    serializer.tax_registration(registration.as_ref(), "FC", "tax_registration");
                }
            },
        );
    }

    // Serializes the buyer party (`BG-7`).
    fn buyer_party(&mut self, buyer: &Buyer) {
        self.group(RAM, "BuyerTradeParty", "buyer", Term::BG(7), |serializer| {
            serializer.party_identifiers(&buyer.identifiers, "identifiers");
            serializer.field_leaf(RAM, "Name", "name", buyer.name.as_ref());
            serializer.legal_organization(
                buyer.legal_entity.as_ref(),
                buyer.trading_name.as_deref_ref(),
            );
            if let Some(contact) = &buyer.contact {
                serializer.contact(contact);
            }
            serializer.postal_address(&buyer.address);
            if let Some(address) = &buyer.electronic_address {
                serializer.electronic_address(address);
            }
            if let Some(vat) = &buyer.vat {
                serializer.tax_registration(&vat.to_string(), "VA", "vat");
            }
        });
    }

    // Serializes the seller tax representative (`BG-11`).
    fn tax_representative_party(&mut self, representative: &TaxRepresentative) {
        self.group(
            RAM,
            "SellerTaxRepresentativeTradeParty",
            "tax_representative",
            Term::BG(11),
            |serializer| {
                serializer.field_leaf(RAM, "Name", "name", representative.name.as_ref());
                serializer.postal_address(&representative.address);
                serializer.tax_registration(&representative.vat.to_string(), "VA", "vat");
            },
        );
    }

    // Serializes the party identifiers, with a scheme id when present.
    fn party_identifiers(&mut self, identifiers: &[OperationalEntity], field: &'static str) {
        for identifier in identifiers {
            match &identifier.issuer {
                Some(issuer) => self.field_leaf_attr(
                    RAM,
                    "GlobalID",
                    field,
                    &[("schemeID", &issuer.to_string())],
                    identifier.id.as_ref(),
                ),
                None => self.field_leaf(RAM, "ID", field, identifier.id.as_ref()),
            }
        }
    }

    // Serializes the specified legal organization: the legal id and the trading name.
    fn legal_organization(&mut self, entity: Option<&LegalEntity>, trading_name: Option<&str>) {
        if entity.is_none() && trading_name.is_none() {
            return;
        }
        self.structural(RAM, "SpecifiedLegalOrganization", |serializer| {
            if let Some(entity) = entity {
                match &entity.issuer {
                    Some(issuer) => serializer.field_leaf_attr(
                        RAM,
                        "ID",
                        "legal_entity",
                        &[("schemeID", &issuer.to_string())],
                        entity.id.as_ref(),
                    ),
                    None => serializer.field_leaf(RAM, "ID", "legal_entity", entity.id.as_ref()),
                }
            }
            if let Some(name) = trading_name {
                serializer.field_leaf(RAM, "TradingBusinessName", "trading_name", name);
            }
        });
    }

    // Serializes a defined trade contact (`BG-6`).
    fn contact(&mut self, contact: &Contact) {
        self.group(
            RAM,
            "DefinedTradeContact",
            "contact",
            Term::BG(6),
            |serializer| {
                if let Some(name) = &contact.name {
                    serializer.field_leaf(RAM, "PersonName", "name", name.as_ref());
                }
                if let Some(phone) = &contact.telephone {
                    serializer.structural(RAM, "TelephoneUniversalCommunication", |serializer| {
                        serializer.field_leaf(RAM, "CompleteNumber", "telephone", phone.as_ref());
                    });
                }
                if let Some(email) = &contact.email {
                    serializer.structural(RAM, "EmailURIUniversalCommunication", |serializer| {
                        serializer.field_leaf(RAM, "URIID", "email", email.as_str());
                    });
                }
            },
        );
    }

    // Serializes a postal trade address.
    fn postal_address(&mut self, address: &PostalAddress) {
        self.field_group(RAM, "PostalTradeAddress", "address", |serializer| {
            if let Some(zip) = &address.postal_code {
                serializer.field_leaf(RAM, "PostcodeCode", "postal_code", zip.as_ref());
            }
            if let Some(line) = &address.line1 {
                serializer.field_leaf(RAM, "LineOne", "line1", line.as_ref());
            }
            if let Some(line) = &address.line2 {
                serializer.field_leaf(RAM, "LineTwo", "line2", line.as_ref());
            }
            if let Some(line) = &address.line3 {
                serializer.field_leaf(RAM, "LineThree", "line3", line.as_ref());
            }
            if let Some(city) = &address.city {
                serializer.field_leaf(RAM, "CityName", "city", city.as_ref());
            }
            serializer.field_leaf(RAM, "CountryID", "country", address.country.alpha2());
            if let Some(subdivision) = &address.country_subdivision {
                serializer.field_leaf(
                    RAM,
                    "CountrySubDivisionName",
                    "country_subdivision",
                    subdivision.as_ref(),
                );
            }
        });
    }

    // Serializes a party electronic address (`BT-34`/`BT-49`).
    fn electronic_address(&mut self, address: &ElectronicAddress) {
        self.structural(RAM, "URIUniversalCommunication", |serializer| {
            serializer.field_leaf_attr(
                RAM,
                "URIID",
                "electronic_address",
                &[("schemeID", &address.scheme.to_string())],
                address.id.as_ref(),
            );
        });
    }

    // Serializes a specified tax registration under a scheme id.
    fn tax_registration(&mut self, id: &str, scheme: &str, field: &'static str) {
        self.field_group(RAM, "SpecifiedTaxRegistration", field, |serializer| {
            serializer.field_leaf_attr(RAM, "ID", field, &[("schemeID", scheme)], id);
        });
    }

    // Serializes the header trade delivery (`BG-13`). The wrapper is mandatory in
    // the CII schema, so it is always written, even when empty.
    fn header_trade_delivery(&mut self, invoice: &Invoice) {
        self.structural(RAM, "ApplicableHeaderTradeDelivery", |serializer| {
            if let Some(delivery) = &invoice.delivery {
                serializer.delivery_party(delivery);
                if let Some(date_value) = delivery.date {
                    serializer.structural(RAM, "ActualDeliverySupplyChainEvent", |serializer| {
                        serializer.datetime("OccurrenceDateTime", "date", Term::BT(72), date_value);
                    });
                }
            }
            if let Some(reference) = &invoice.despatch_advice_reference {
                serializer.reference(
                    RAM,
                    "DespatchAdviceReferencedDocument",
                    Term::BT(16),
                    "despatch_advice_reference",
                    reference.as_ref(),
                );
            }
            if let Some(reference) = &invoice.receiving_advice_reference {
                serializer.reference(
                    RAM,
                    "ReceivingAdviceReferencedDocument",
                    Term::BT(15),
                    "receiving_advice_reference",
                    reference.as_ref(),
                );
            }
        });
    }

    // Serializes the delivery ship-to party (`BG-13` name, location, address).
    fn delivery_party(&mut self, delivery: &Delivery) {
        if delivery.name.is_none() && delivery.location.is_none() && delivery.address.is_none() {
            return;
        }
        self.group(
            RAM,
            "ShipToTradeParty",
            "delivery",
            Term::BG(13),
            |serializer| {
                if let Some(location) = &delivery.location {
                    match &location.issuer {
                        Some(issuer) => serializer.field_leaf_attr(
                            RAM,
                            "ID",
                            "location",
                            &[("schemeID", &issuer.to_string())],
                            location.id.as_ref(),
                        ),
                        None => serializer.field_leaf(RAM, "ID", "location", location.id.as_ref()),
                    }
                }
                if let Some(name) = &delivery.name {
                    serializer.field_leaf(RAM, "Name", "name", name.as_ref());
                }
                if let Some(address) = &delivery.address {
                    serializer.postal_address(address);
                }
            },
        );
    }

    // Serializes the header trade settlement.
    fn header_trade_settlement(&mut self, invoice: &Invoice) {
        let currency = self.currency;
        self.structural(RAM, "ApplicableHeaderTradeSettlement", |serializer| {
            if let Some(creditor) = invoice.payment.as_ref().and_then(direct_debit_creditor) {
                serializer.leaf(
                    RAM,
                    "CreditorReferenceID",
                    "creditor_identifier",
                    Term::BT(90),
                    creditor,
                );
            }
            if let Some(payment) = &invoice.payment {
                if let Some(reference) = &payment.remittance_information {
                    serializer.leaf(
                        RAM,
                        "PaymentReference",
                        "remittance_information",
                        Term::BT(83),
                        reference.as_ref(),
                    );
                }
            }
            if let Some(accounting) = &invoice.vat_accounting_total {
                serializer.leaf(
                    RAM,
                    "TaxCurrencyCode",
                    "vat_accounting_total",
                    Term::BT(6),
                    accounting.currency.code(),
                );
            }
            serializer.leaf(
                RAM,
                "InvoiceCurrencyCode",
                "currency",
                Term::BT(5),
                currency,
            );
            if let Some(payee) = &invoice.payee {
                serializer.payee_party(payee);
            }
            if let Some(payment) = &invoice.payment {
                serializer.payment_means(payment);
            }
            serializer.tax_breakdown(invoice, currency);
            if let Some(period) = invoice.invoicing_period {
                serializer.billing_period(period, "invoicing_period", Term::BG(14));
            }
            serializer.adjustments(&invoice.adjustments);
            serializer.payment_terms(invoice);
            serializer.monetary_summation(invoice, currency);
            serializer.preceding_invoices(&invoice.preceding_invoices);
            if let Some(reference) = &invoice.buyer_accounting_reference {
                serializer.field_group(
                    RAM,
                    "ReceivableSpecifiedTradeAccountingAccount",
                    "buyer_accounting_reference",
                    |serializer| {
                        serializer.derived(RAM, "ID", &[], reference.as_ref());
                    },
                );
            }
        });
    }

    // Serializes the payee party (`BG-10`).
    fn payee_party(&mut self, payee: &Payee) {
        self.group(
            RAM,
            "PayeeTradeParty",
            "payee",
            Term::BG(10),
            |serializer| {
                serializer.party_identifiers(&payee.identifiers, "identifiers");
                serializer.field_leaf(RAM, "Name", "name", payee.name.as_ref());
                if let Some(entity) = &payee.legal_entity {
                    serializer.structural(RAM, "SpecifiedLegalOrganization", |serializer| {
                        match &entity.issuer {
                            Some(issuer) => serializer.field_leaf_attr(
                                RAM,
                                "ID",
                                "legal_entity",
                                &[("schemeID", &issuer.to_string())],
                                entity.id.as_ref(),
                            ),
                            None => {
                                serializer.field_leaf(RAM, "ID", "legal_entity", entity.id.as_ref())
                            }
                        }
                    });
                }
            },
        );
    }

    // Serializes the payment means (`BG-16`).
    fn payment_means(&mut self, payment: &PaymentInstructions) {
        self.group(
            RAM,
            "SpecifiedTradeSettlementPaymentMeans",
            "payment",
            Term::BG(16),
            |serializer| {
                serializer.derived(RAM, "TypeCode", &[], &payment.means.to_string());
                if let Some(text) = &payment.means_text {
                    serializer.derived(RAM, "Information", &[], text.as_ref());
                }
                match &payment.details {
                    Some(PaymentDetails::Card(card)) => {
                        serializer.nested(
                            RAM,
                            "ApplicableTradeSettlementFinancialCard",
                            |serializer| {
                                serializer.derived(
                                    RAM,
                                    "ID",
                                    &[],
                                    card.primary_account_number.as_ref(),
                                );
                                if let Some(holder) = &card.holder_name {
                                    serializer.derived(RAM, "CardholderName", &[], holder.as_ref());
                                }
                            },
                        );
                    }
                    Some(PaymentDetails::DirectDebit(debit)) => {
                        if let Some(account) = &debit.debited_account {
                            serializer.nested(
                                RAM,
                                "PayerPartyDebtorFinancialAccount",
                                |serializer| {
                                    serializer.derived(RAM, "IBANID", &[], account.as_ref());
                                },
                            );
                        }
                    }
                    Some(PaymentDetails::CreditTransfers(transfers)) => {
                        for transfer in transfers {
                            serializer.nested(
                                RAM,
                                "PayeePartyCreditorFinancialAccount",
                                |serializer| {
                                    serializer.derived(
                                        RAM,
                                        "IBANID",
                                        &[],
                                        transfer.account.as_ref(),
                                    );
                                    if let Some(name) = &transfer.account_name {
                                        serializer.derived(RAM, "AccountName", &[], name.as_ref());
                                    }
                                },
                            );
                            if let Some(provider) = &transfer.provider {
                                serializer.nested(
                                    RAM,
                                    "PayeeSpecifiedCreditorFinancialInstitution",
                                    |serializer| {
                                        serializer.derived(RAM, "BICID", &[], provider.as_ref());
                                    },
                                );
                            }
                        }
                    }
                    None => {}
                }
            },
        );
    }

    // Serializes the VAT breakdown (`BG-23`), all derived and mapped to the root.
    fn tax_breakdown(&mut self, invoice: &Invoice, currency: &str) {
        let event = match invoice.vat_point {
            Some(VatPoint::Event(event)) => Some(u16::from(event).to_string()),
            _ => None,
        };
        for group in invoice.vat_breakdown() {
            self.structural(RAM, "ApplicableTradeTax", |serializer| {
                serializer.derived(RAM, "CalculatedAmount", &[], &money(group.tax));
                serializer.derived(RAM, "TypeCode", &[], "VAT");
                if let VatTreatment::Exempt {
                    text: Some(text), ..
                } = &group.treatment
                {
                    serializer.derived(RAM, "ExemptionReason", &[], text.as_ref());
                }
                serializer.derived(RAM, "BasisAmount", &[], &money(group.taxable));
                serializer.derived(
                    RAM,
                    "CategoryCode",
                    &[],
                    &group.treatment.category().to_string(),
                );
                if let VatTreatment::Exempt {
                    code: Some(code), ..
                } = &group.treatment
                {
                    serializer.derived(RAM, "ExemptionReasonCode", &[], &code.to_string());
                }
                if let Some(event) = &event {
                    serializer.derived(RAM, "DueDateTypeCode", &[], event);
                }
                serializer.derived(
                    RAM,
                    "RateApplicablePercent",
                    &[],
                    &plain(group.treatment.rate()),
                );
            });
        }
        let _ = currency;
    }

    // Serializes a billing period (`BG-14`/`BG-26`).
    fn billing_period(&mut self, period: Period, field: &'static str, term: Term) {
        self.group(RAM, "BillingSpecifiedPeriod", field, term, |serializer| {
            if let Some(start) = period.start() {
                serializer.datetime("StartDateTime", field, term, start);
            }
            if let Some(end) = period.end() {
                serializer.datetime("EndDateTime", field, term, end);
            }
        });
    }

    // Serializes the document-level allowances and charges (`BG-20`/`BG-21`).
    fn adjustments(&mut self, adjustments: &[Adjustment]) {
        for (position, adjustment) in adjustments.iter().enumerate() {
            let instance = index(position);
            let charge = matches!(adjustment.reason, AdjustmentReason::Charge { .. });
            let term = if charge { Term::BG(21) } else { Term::BG(20) };
            self.repeatable(
                RAM,
                "SpecifiedTradeAllowanceCharge",
                "adjustments",
                term,
                instance,
                |serializer| {
                    serializer.indicator(charge);
                    serializer.adjustment_amount(&adjustment.amount);
                    serializer.adjustment_reason(&adjustment.reason);
                    serializer.nested(RAM, "CategoryTradeTax", |serializer| {
                        serializer.derived(RAM, "TypeCode", &[], "VAT");
                        serializer.derived(
                            RAM,
                            "CategoryCode",
                            &[],
                            &adjustment.vat.category().to_string(),
                        );
                        serializer.derived(
                            RAM,
                            "RateApplicablePercent",
                            &[],
                            &plain(adjustment.vat.rate()),
                        );
                    });
                },
            );
        }
    }

    // Serializes the amount of an adjustment, mapped to the enclosing adjustment.
    fn adjustment_amount(&mut self, amount: &AdjustmentAmount) {
        if let AdjustmentAmount::Relative { rate, base } = amount {
            self.derived(RAM, "CalculationPercent", &[], &plain(Decimal::from(*rate)));
            self.derived(RAM, "BasisAmount", &[], &money(*base));
        }
        self.derived(RAM, "ActualAmount", &[], &money(amount.value()));
    }

    // Serializes the reason code and text of an adjustment.
    fn adjustment_reason(&mut self, reason: &AdjustmentReason) {
        let (code, text) = match reason {
            AdjustmentReason::Allowance { code, text } => {
                (code.as_ref().map(|code| code.to_string()), text.as_ref())
            }
            AdjustmentReason::Charge { code, text } => {
                (code.as_ref().map(|code| code.to_string()), text.as_ref())
            }
        };
        if let Some(code) = code {
            self.derived(RAM, "ReasonCode", &[], &code);
        }
        if let Some(text) = text {
            self.derived(RAM, "Reason", &[], text.as_ref());
        }
    }

    // Serializes the payment terms (`BT-20`), due date (`BT-9`), and mandate (`BT-89`).
    fn payment_terms(&mut self, invoice: &Invoice) {
        let mandate = invoice.payment.as_ref().and_then(direct_debit_mandate);
        if invoice.payment_terms.is_none()
            && invoice.payment_due_date.is_none()
            && mandate.is_none()
        {
            return;
        }
        self.structural(RAM, "SpecifiedTradePaymentTerms", |serializer| {
            if let Some(terms) = &invoice.payment_terms {
                serializer.field_leaf(RAM, "Description", "payment_terms", terms.as_ref());
            }
            if let Some(due) = invoice.payment_due_date {
                serializer.datetime("DueDateDateTime", "payment_due_date", Term::BT(9), due);
            }
            if let Some(mandate) = mandate {
                serializer.field_leaf(RAM, "DirectDebitMandateID", "mandate_reference", mandate);
            }
        });
    }

    // Serializes the header monetary summation, all derived and mapped to the root.
    fn monetary_summation(&mut self, invoice: &Invoice, currency: &str) {
        self.structural(
            RAM,
            "SpecifiedTradeSettlementHeaderMonetarySummation",
            |serializer| {
                serializer.derived(
                    RAM,
                    "LineTotalAmount",
                    &[],
                    &money(invoice.line_net_total()),
                );
                if !invoice.charges_total().is_zero() {
                    serializer.derived(
                        RAM,
                        "ChargeTotalAmount",
                        &[],
                        &money(invoice.charges_total()),
                    );
                }
                if !invoice.allowances_total().is_zero() {
                    serializer.derived(
                        RAM,
                        "AllowanceTotalAmount",
                        &[],
                        &money(invoice.allowances_total()),
                    );
                }
                serializer.derived(RAM, "TaxBasisTotalAmount", &[], &money(invoice.net_total()));
                serializer.derived(
                    RAM,
                    "TaxTotalAmount",
                    &[("currencyID", currency)],
                    &money(invoice.vat_total()),
                );
                if let Some(rounding) = invoice.rounding {
                    serializer.derived(RAM, "RoundingAmount", &[], &money(rounding));
                }
                serializer.derived(RAM, "GrandTotalAmount", &[], &money(invoice.gross_total()));
                if let Some(paid) = invoice.paid {
                    serializer.derived(RAM, "TotalPrepaidAmount", &[], &money(paid));
                }
                serializer.derived(RAM, "DuePayableAmount", &[], &money(invoice.due()));
            },
        );
    }

    // Serializes the preceding invoice references (`BG-3`).
    fn preceding_invoices(&mut self, preceding: &[PrecedingInvoice]) {
        for (position, invoice) in preceding.iter().enumerate() {
            let instance = index(position);
            self.repeatable(
                RAM,
                "InvoiceReferencedDocument",
                "preceding_invoices",
                Term::BG(3),
                instance,
                |serializer| {
                    serializer.derived(RAM, "IssuerAssignedID", &[], invoice.number.as_ref());
                    if let Some(issued) = invoice.issue_date {
                        serializer.nested(RAM, "FormattedIssueDateTime", |serializer| {
                            serializer.write_raw_datetime(QDT_PREFIX, issued);
                        });
                    }
                },
            );
        }
    }

    // Serializes a single-identifier reference document.
    fn reference(
        &mut self,
        namespace: Namespace,
        element: &str,
        term: Term,
        field: &'static str,
        id: &str,
    ) {
        self.group(namespace, element, field, term, |serializer| {
            serializer.derived(RAM, "IssuerAssignedID", &[], id);
        });
    }

    // ---- datatype carriers (not recorded) --------------------------------

    // Writes a date wrapper whose value carrier is a `udt:DateTimeString`.
    fn datetime(&mut self, element: &str, field: &'static str, term: Term, value: Date) {
        if self.is_forbidden(term) {
            return;
        }
        self.trace.enter(RAM, element);
        self.trace.push_field(field);
        self.trace.record_context();
        self.write_start(RAM, element);
        self.write_raw_datetime(UDT_PREFIX, value);
        self.write_end(RAM, element);
        self.trace.pop_context();
        self.trace.leave();
    }

    // Writes a charge indicator whose value carrier is a `udt:Indicator`.
    fn indicator(&mut self, charge: bool) {
        self.trace.enter(RAM, "ChargeIndicator");
        self.trace.record_context();
        self.write_start(RAM, "ChargeIndicator");
        self.write_raw(
            UDT_PREFIX,
            "Indicator",
            if charge { "true" } else { "false" },
        );
        self.write_end(RAM, "ChargeIndicator");
        self.trace.leave();
    }

    // Writes a `DateTimeString` value carrier in the given datatype namespace.
    fn write_raw_datetime(&mut self, prefix: &str, value: Date) {
        let tag = format!("{prefix}:DateTimeString");
        let start = BytesStart::new(tag.clone()).with_attributes([("format", "102")]);
        self.write(Event::Start(start));
        self.write(Event::Text(BytesText::new(&date(value))));
        self.write(Event::End(BytesEnd::new(tag)));
    }

    // Writes a value-carrier element from a datatype namespace, never recorded.
    fn write_raw(&mut self, prefix: &str, name: &str, value: &str) {
        let tag = format!("{prefix}:{name}");
        self.write(Event::Start(BytesStart::new(tag.clone())));
        self.write(Event::Text(BytesText::new(value)));
        self.write(Event::End(BytesEnd::new(tag)));
    }

    // ---- element writers (mirror the UBL serializer) ---------------------

    fn leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        value: &str,
    ) {
        if self.is_forbidden(term) {
            return;
        }
        self.write_field_leaf(namespace, name, field, &[], value);
    }

    fn leaf_attr(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        if self.is_forbidden(term) {
            return;
        }
        self.write_field_leaf(namespace, name, field, attributes, value);
    }

    fn field_leaf(&mut self, namespace: Namespace, name: &str, field: &'static str, value: &str) {
        self.write_field_leaf(namespace, name, field, &[], value);
    }

    fn field_leaf_attr(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        self.write_field_leaf(namespace, name, field, attributes, value);
    }

    fn write_field_leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        self.trace.enter(namespace, name);
        self.trace.push_field(field);
        self.trace.record_context();
        self.write_element(namespace, name, attributes, value);
        self.trace.pop_context();
        self.trace.leave();
    }

    fn rooted(
        &mut self,
        namespace: Namespace,
        name: &str,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        self.trace.enter(namespace, name);
        self.trace.record_root();
        self.write_element(namespace, name, attributes, value);
        self.trace.leave();
    }

    fn derived(
        &mut self,
        namespace: Namespace,
        name: &str,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        self.trace.enter(namespace, name);
        self.trace.record_context();
        self.write_element(namespace, name, attributes, value);
        self.trace.leave();
    }

    fn group(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        body: impl FnOnce(&mut Self),
    ) {
        if self.is_forbidden(term) {
            return;
        }
        self.trace.enter(namespace, name);
        self.trace.push_field(field);
        self.trace.record_context();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.pop_context();
        self.trace.leave();
    }

    // Writes a group mapped to a model field, never filtered (its term is never forbidden).
    fn field_group(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        body: impl FnOnce(&mut Self),
    ) {
        self.trace.enter(namespace, name);
        self.trace.push_field(field);
        self.trace.record_context();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.pop_context();
        self.trace.leave();
    }

    fn repeatable(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        instance: NonZeroUsize,
        body: impl FnOnce(&mut Self),
    ) {
        if self.is_forbidden(term) {
            return;
        }
        self.trace.enter(namespace, name);
        self.trace.push_instance(field, instance);
        self.trace.record_context();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.pop_context();
        self.trace.leave();
    }

    fn structural(&mut self, namespace: Namespace, name: &str, body: impl FnOnce(&mut Self)) {
        self.trace.enter(namespace, name);
        self.trace.record_root();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    fn nested(&mut self, namespace: Namespace, name: &str, body: impl FnOnce(&mut Self)) {
        self.trace.enter(namespace, name);
        self.trace.record_context();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    fn write_element(
        &mut self,
        namespace: Namespace,
        name: &str,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        let tag = qname(namespace, name);
        let mut start = BytesStart::new(tag.clone());
        for (key, attribute_value) in attributes {
            start.push_attribute((*key, *attribute_value));
        }
        self.write(Event::Start(start));
        self.write(Event::Text(BytesText::new(value)));
        self.write(Event::End(BytesEnd::new(tag)));
    }

    fn write_start(&mut self, namespace: Namespace, name: &str) {
        self.write(Event::Start(BytesStart::new(qname(namespace, name))));
    }

    fn write_end(&mut self, namespace: Namespace, name: &str) {
        self.write(Event::End(BytesEnd::new(qname(namespace, name))));
    }

    fn write(&mut self, event: Event<'_>) {
        self.inner
            .write_event(event)
            .expect("writing XML to an in-memory buffer never fails");
    }
}

// The record-form qualified name of an element, prefixed for its namespace.
fn qname(namespace: Namespace, name: &str) -> String {
    format!("{}:{}", prefix(namespace), name)
}

fn index(position: usize) -> NonZeroUsize {
    NonZeroUsize::new(position + 1).expect("a positive index")
}

// The direct-debit creditor identifier (`BT-90`), when the payment is a direct debit.
fn direct_debit_creditor(payment: &PaymentInstructions) -> Option<&str> {
    match &payment.details {
        Some(PaymentDetails::DirectDebit(debit)) => debit
            .creditor_identifier
            .as_ref()
            .map(|value| value.as_ref()),
        _ => None,
    }
}

// The direct-debit mandate reference (`BT-89`), when the payment is a direct debit.
fn direct_debit_mandate(payment: &PaymentInstructions) -> Option<&str> {
    match &payment.details {
        Some(PaymentDetails::DirectDebit(debit)) => {
            debit.mandate_reference.as_ref().map(|value| value.as_ref())
        }
        _ => None,
    }
}

// A small extension to read an optional non-empty string as an optional `&str`.
trait AsDerefRef {
    fn as_deref_ref(&self) -> Option<&str>;
}

impl AsDerefRef for Option<NonEmptyString> {
    fn as_deref_ref(&self) -> Option<&str> {
        self.as_ref().map(|value| value.as_ref())
    }
}
