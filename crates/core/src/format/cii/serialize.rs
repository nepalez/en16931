use super::Namespace;
use crate::Serializable;
use crate::prelude::*;
use crate::{
    Adjustment, AdjustmentAmount, AdjustmentReason, Buyer, Cii, Contact, Currency, Delivery,
    DocumentBuilder, ElectronicAddress, Invoice, InvoiceLine, Item, LegalEntity, LineAdjustment,
    NonEmptyString, OperationalEntity, Payee, PaymentDetails, PaymentInstructions, Period,
    PostalAddress, PrecedingInvoice, Price, Seller, Serializer, TaxRepresentative, Term, VatPoint,
    VatTreatment,
};

// Renders a date as an ISO-8601 basic date (`YYYYMMDD`), the CII form 102.
fn date(value: Date) -> String {
    format!(
        "{:04}{:02}{:02}",
        value.year(),
        u8::from(value.month()),
        value.day()
    )
}

// Renders a monetary amount rounded to two fraction digits, the EN-16931 form.
// It is the only place where the library rounds an amount.
fn money(value: Decimal) -> String {
    let rounded = value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    format!("{rounded:.2}")
}

// Renders a plain decimal (quantity, rate, factor, price) with its own scale.
fn plain(value: Decimal) -> String {
    value.to_string()
}

impl Serializable<Cii> for Invoice {
    // Serializes the whole document under the CII root element.
    fn serialize(serializer: &mut Serializer<Cii>, builder: &DocumentBuilder<Invoice>) {
        let invoice = &builder.invoice;
        serializer.root(|serializer| {
            serializer.exchanged_document_context(builder);
            serializer.exchanged_document(invoice);
            serializer.structural(
                Namespace::Rsm,
                "SupplyChainTradeTransaction",
                |serializer| {
                    serializer.lines(&invoice.lines);
                    serializer.header_trade_agreement(invoice);
                    serializer.header_trade_delivery(invoice);
                    serializer.header_trade_settlement(invoice);
                },
            );
        });
    }
}

impl Serializer<Cii> {
    // Serializes the document context: the business process (BT-23) and the profile (BT-24).
    fn exchanged_document_context(&mut self, builder: &DocumentBuilder<Invoice>) {
        self.structural(Namespace::Rsm, "ExchangedDocumentContext", |serializer| {
            if let Some(process) = &builder.business_process {
                serializer.structural(
                    Namespace::Ram,
                    "BusinessProcessSpecifiedDocumentContextParameter",
                    |serializer| {
                        serializer.rooted(Namespace::Ram, "ID", &[], process.as_ref());
                    },
                );
            }
            serializer.structural(
                Namespace::Ram,
                "GuidelineSpecifiedDocumentContextParameter",
                |serializer| {
                    serializer.rooted(Namespace::Ram, "ID", &[], &builder.profile.to_string());
                },
            );
        });
    }

    // Serializes the exchanged document header: number, type, date, and notes.
    fn exchanged_document(&mut self, invoice: &Invoice) {
        self.structural(Namespace::Rsm, "ExchangedDocument", |serializer| {
            if let Some(number) = &invoice.number {
                serializer.leaf(Namespace::Ram, "ID", "number", Term::BT(1), number.as_ref());
            }
            serializer.leaf(
                Namespace::Ram,
                "TypeCode",
                "type_code",
                Term::BT(3),
                &invoice.type_code.to_string(),
            );
            if let Some(issued) = invoice.issue_date {
                serializer.datetime("IssueDateTime", "issue_date", Term::BT(2), issued);
            }
            for (position, note) in invoice.notes.iter().enumerate() {
                let instance = index(position);
                serializer.repeatable(
                    Namespace::Ram,
                    "IncludedNote",
                    "notes",
                    Term::BG(1),
                    instance,
                    |serializer| {
                        if let Some(text) = &note.text {
                            serializer.derived(Namespace::Ram, "Content", &[], text.as_ref());
                        }
                        if let Some(code) = &note.subject_code {
                            if !serializer.forbids(Term::BT(21)) {
                                serializer.derived(
                                    Namespace::Ram,
                                    "SubjectCode",
                                    &[],
                                    code.as_ref(),
                                );
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
                Namespace::Ram,
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
        self.structural(
            Namespace::Ram,
            "AssociatedDocumentLineDocument",
            |serializer| {
                if let Some(id) = &line.id {
                    serializer.leaf(Namespace::Ram, "LineID", "id", Term::BT(126), id.as_ref());
                }
                if let Some(note) = &line.note {
                    serializer.group(
                        Namespace::Ram,
                        "IncludedNote",
                        "note",
                        Term::BT(127),
                        |serializer| {
                            serializer.derived(Namespace::Ram, "Content", &[], note.as_ref());
                        },
                    );
                }
            },
        );
        if let Some(item) = &line.item {
            self.line_product(item);
        }
        self.line_agreement(line);
        self.structural(Namespace::Ram, "SpecifiedLineTradeDelivery", |serializer| {
            if let Some(quantity) = &line.quantity {
                serializer.leaf_attr(
                    Namespace::Ram,
                    "BilledQuantity",
                    "quantity",
                    Term::BT(129),
                    &[("unitCode", quantity.unit.code())],
                    &plain(quantity.value),
                );
            }
        });
        self.line_settlement(line);
    }

    // Serializes the line product (`BG-31`), rebasing the tax onto the line VAT elsewhere.
    fn line_product(&mut self, item: &Item) {
        self.field_group(
            Namespace::Ram,
            "SpecifiedTradeProduct",
            "item",
            |serializer| {
                if let Some(standard) = &item.standard_id {
                    if let Some(id) = &standard.id {
                        match &standard.issuer {
                            Some(issuer) => serializer.field_leaf_attr(
                                Namespace::Ram,
                                "GlobalID",
                                "standard_id",
                                &[("schemeID", &issuer.to_string())],
                                id.as_ref(),
                            ),
                            None => serializer.field_leaf(
                                Namespace::Ram,
                                "GlobalID",
                                "standard_id",
                                id.as_ref(),
                            ),
                        }
                    }
                }
                if let Some(id) = &item.seller_id {
                    serializer.field_leaf(
                        Namespace::Ram,
                        "SellerAssignedID",
                        "seller_id",
                        id.as_ref(),
                    );
                }
                if let Some(id) = &item.buyer_id {
                    serializer.field_leaf(
                        Namespace::Ram,
                        "BuyerAssignedID",
                        "buyer_id",
                        id.as_ref(),
                    );
                }
                if let Some(name) = &item.name {
                    serializer.field_leaf(Namespace::Ram, "Name", "name", name.as_ref());
                }
                if let Some(description) = &item.description {
                    serializer.field_leaf(
                        Namespace::Ram,
                        "Description",
                        "description",
                        description.as_ref(),
                    );
                }
                for attribute in &item.attributes {
                    serializer.structural(
                        Namespace::Ram,
                        "ApplicableProductCharacteristic",
                        |serializer| {
                            if let Some(name) = &attribute.name {
                                serializer.field_leaf(
                                    Namespace::Ram,
                                    "Description",
                                    "attributes",
                                    name.as_ref(),
                                );
                            }
                            if let Some(value) = &attribute.value {
                                serializer.field_leaf(
                                    Namespace::Ram,
                                    "Value",
                                    "attributes",
                                    value.as_ref(),
                                );
                            }
                        },
                    );
                }
                for classification in &item.classifications {
                    serializer.structural(
                        Namespace::Ram,
                        "DesignatedProductClassification",
                        |serializer| {
                            let Some(id) = &classification.id else {
                                return;
                            };
                            let mut attributes = Vec::new();
                            if let Some(scheme) = &classification.scheme {
                                attributes.push(("listID".to_owned(), scheme.to_string()));
                            }
                            if let Some(version) = &classification.version {
                                attributes.push((
                                    "listVersionID".to_owned(),
                                    version.as_ref().to_owned(),
                                ));
                            }
                            let borrowed: Vec<(&str, &str)> = attributes
                                .iter()
                                .map(|(key, value)| (key.as_str(), value.as_str()))
                                .collect();
                            serializer.field_leaf_attr(
                                Namespace::Ram,
                                "ClassCode",
                                "classifications",
                                &borrowed,
                                id.as_ref(),
                            );
                        },
                    );
                }
                if let Some(country) = item.country_of_origin {
                    serializer.structural(Namespace::Ram, "OriginTradeCountry", |serializer| {
                        serializer.field_leaf(
                            Namespace::Ram,
                            "ID",
                            "country_of_origin",
                            country.alpha2(),
                        );
                    });
                }
            },
        );
    }

    // Serializes the line trade agreement: order line reference and prices.
    fn line_agreement(&mut self, line: &InvoiceLine) {
        self.structural(
            Namespace::Ram,
            "SpecifiedLineTradeAgreement",
            |serializer| {
                if let Some(order) = &line.order_line_reference {
                    serializer.structural(
                        Namespace::Ram,
                        "BuyerOrderReferencedDocument",
                        |serializer| {
                            serializer.leaf(
                                Namespace::Ram,
                                "LineID",
                                "order_line_reference",
                                Term::BT(132),
                                order.as_ref(),
                            );
                        },
                    );
                }
                if let Some(price) = &line.price {
                    serializer.line_price(price);
                }
            },
        );
    }

    // Serializes the gross and net prices of a line (`BG-29`), every price with its own scale.
    // The gross price group is absent without the gross price and the discount.
    fn line_price(&mut self, price: &Price) {
        if price.gross.is_some() || price.discount.is_some() {
            self.field_group(
                Namespace::Ram,
                "GrossPriceProductTradePrice",
                "price",
                |serializer| {
                    if let Some(gross) = price.gross {
                        serializer.derived(Namespace::Ram, "ChargeAmount", &[], &plain(gross));
                    }
                    if let Some(base) = price.base_quantity {
                        serializer.derived(
                            Namespace::Ram,
                            "BasisQuantity",
                            &[("unitCode", base.unit.code())],
                            &plain(base.value),
                        );
                    }
                    if let Some(discount) = price.discount {
                        serializer.nested(
                            Namespace::Ram,
                            "AppliedTradeAllowanceCharge",
                            |serializer| {
                                serializer.indicator(false);
                                serializer.derived(
                                    Namespace::Ram,
                                    "ActualAmount",
                                    &[],
                                    &plain(discount),
                                );
                            },
                        );
                    }
                },
            );
        }
        self.field_group(
            Namespace::Ram,
            "NetPriceProductTradePrice",
            "price",
            |serializer| {
                if let Some(net) = price.net {
                    serializer.leaf(
                        Namespace::Ram,
                        "ChargeAmount",
                        "net",
                        Term::BT(146),
                        &plain(net),
                    );
                }
                if let Some(base) = price.base_quantity {
                    serializer.derived(
                        Namespace::Ram,
                        "BasisQuantity",
                        &[("unitCode", base.unit.code())],
                        &plain(base.value),
                    );
                }
            },
        );
    }

    // Serializes the line trade settlement: tax, period, adjustments, totals, object, account.
    fn line_settlement(&mut self, line: &InvoiceLine) {
        self.structural(
            Namespace::Ram,
            "SpecifiedLineTradeSettlement",
            |serializer| {
                if let Some(vat) = &line.vat {
                    serializer.line_tax(vat);
                }
                if let Some(period) = line.period {
                    serializer.billing_period(period, "period", Term::BG(26));
                }
                for (position, adjustment) in line.adjustments.iter().enumerate() {
                    let instance = index(position);
                    serializer.line_adjustment(adjustment, instance);
                }
                if let Some(net) = line.net_amount {
                    serializer.structural(
                        Namespace::Ram,
                        "SpecifiedTradeSettlementLineMonetarySummation",
                        |serializer| {
                            serializer.leaf(
                                Namespace::Ram,
                                "LineTotalAmount",
                                "net_amount",
                                Term::BT(131),
                                &money(net),
                            );
                        },
                    );
                }
                if let Some(object) = &line.object {
                    serializer.field_group(
                        Namespace::Ram,
                        "AdditionalReferencedDocument",
                        "object",
                        |serializer| {
                            if let Some(id) = &object.id {
                                serializer.derived(
                                    Namespace::Ram,
                                    "IssuerAssignedID",
                                    &[],
                                    id.as_ref(),
                                );
                            }
                            serializer.derived(Namespace::Ram, "TypeCode", &[], "130");
                            if let Some(scheme) = &object.scheme {
                                serializer.derived(
                                    Namespace::Ram,
                                    "ReferenceTypeCode",
                                    &[],
                                    &scheme.to_string(),
                                );
                            }
                        },
                    );
                }
                if let Some(reference) = &line.buyer_accounting_reference {
                    serializer.field_group(
                        Namespace::Ram,
                        "ReceivableSpecifiedTradeAccountingAccount",
                        "buyer_accounting_reference",
                        |serializer| {
                            serializer.derived(Namespace::Ram, "ID", &[], reference.as_ref());
                        },
                    );
                }
            },
        );
    }

    // Serializes the line VAT as a CII applicable trade tax, mapped to the line VAT.
    fn line_tax(&mut self, vat: &VatTreatment) {
        self.field_group(Namespace::Ram, "ApplicableTradeTax", "vat", |serializer| {
            serializer.derived(Namespace::Ram, "TypeCode", &[], "VAT");
            serializer.derived(
                Namespace::Ram,
                "CategoryCode",
                &[],
                &vat.category().to_string(),
            );
            serializer.derived(
                Namespace::Ram,
                "RateApplicablePercent",
                &[],
                &plain(vat.rate()),
            );
        });
    }

    // Serializes one line-level allowance or charge.
    fn line_adjustment(&mut self, adjustment: &LineAdjustment, instance: NonZeroUsize) {
        let charge = adjustment
            .reason
            .as_ref()
            .map(|reason| matches!(reason, AdjustmentReason::Charge { .. }));
        let term = if charge == Some(true) {
            Term::BG(28)
        } else {
            Term::BG(27)
        };
        self.repeatable(
            Namespace::Ram,
            "SpecifiedTradeAllowanceCharge",
            "adjustments",
            term,
            instance,
            |serializer| {
                if let Some(charge) = charge {
                    serializer.indicator(charge);
                }
                if let Some(amount) = &adjustment.amount {
                    serializer.adjustment_amount(amount);
                }
                if let Some(reason) = &adjustment.reason {
                    serializer.adjustment_reason(reason);
                }
            },
        );
    }

    // Serializes the header trade agreement (`BG-4`, `BG-7`, references, `BG-11`).
    fn header_trade_agreement(&mut self, invoice: &Invoice) {
        self.structural(
            Namespace::Ram,
            "ApplicableHeaderTradeAgreement",
            |serializer| {
                if let Some(reference) = &invoice.buyer_reference {
                    serializer.leaf(
                        Namespace::Ram,
                        "BuyerReference",
                        "buyer_reference",
                        Term::BT(10),
                        reference.as_ref(),
                    );
                }
                if let Some(seller) = &invoice.seller {
                    serializer.seller_party(seller);
                }
                if let Some(buyer) = &invoice.buyer {
                    serializer.buyer_party(buyer);
                }
                if let Some(representative) = &invoice.tax_representative {
                    serializer.tax_representative_party(representative);
                }
                if let Some(sales) = &invoice.sales_order_reference {
                    serializer.reference(
                        Namespace::Ram,
                        "SellerOrderReferencedDocument",
                        Term::BT(14),
                        "sales_order_reference",
                        sales.as_ref(),
                    );
                }
                if let Some(order) = &invoice.purchase_order_reference {
                    serializer.reference(
                        Namespace::Ram,
                        "BuyerOrderReferencedDocument",
                        Term::BT(13),
                        "purchase_order_reference",
                        order.as_ref(),
                    );
                }
                if let Some(contract) = &invoice.contract_reference {
                    serializer.reference(
                        Namespace::Ram,
                        "ContractReferencedDocument",
                        Term::BT(12),
                        "contract_reference",
                        contract.as_ref(),
                    );
                }
                serializer.additional_documents(invoice);
                if let Some(project) = &invoice.project_reference {
                    serializer.group(
                        Namespace::Ram,
                        "SpecifiedProcuringProject",
                        "project_reference",
                        Term::BT(11),
                        |serializer| {
                            serializer.derived(Namespace::Ram, "ID", &[], project.as_ref());
                            serializer.derived(Namespace::Ram, "Name", &[], "Project");
                        },
                    );
                }
            },
        );
    }

    // Serializes the invoiced object (`BT-18`), tender (`BT-17`), and supporting documents (`BG-24`).
    fn additional_documents(&mut self, invoice: &Invoice) {
        if let Some(object) = &invoice.object {
            self.group(
                Namespace::Ram,
                "AdditionalReferencedDocument",
                "object",
                Term::BT(18),
                |serializer| {
                    if let Some(id) = &object.id {
                        serializer.derived(Namespace::Ram, "IssuerAssignedID", &[], id.as_ref());
                    }
                    serializer.derived(Namespace::Ram, "TypeCode", &[], "130");
                    if let Some(scheme) = &object.scheme {
                        serializer.derived(
                            Namespace::Ram,
                            "ReferenceTypeCode",
                            &[],
                            &scheme.to_string(),
                        );
                    }
                },
            );
        }
        if let Some(tender) = &invoice.tender_or_lot_reference {
            self.group(
                Namespace::Ram,
                "AdditionalReferencedDocument",
                "tender_or_lot_reference",
                Term::BT(17),
                |serializer| {
                    serializer.derived(Namespace::Ram, "IssuerAssignedID", &[], tender.as_ref());
                    serializer.derived(Namespace::Ram, "TypeCode", &[], "50");
                },
            );
        }
        for (position, document) in invoice.supporting_documents.iter().enumerate() {
            let instance = index(position);
            self.repeatable(
                Namespace::Ram,
                "AdditionalReferencedDocument",
                "supporting_documents",
                Term::BG(24),
                instance,
                |serializer| {
                    if let Some(reference) = &document.reference {
                        serializer.derived(
                            Namespace::Ram,
                            "IssuerAssignedID",
                            &[],
                            reference.as_ref(),
                        );
                    }
                    if let Some(location) = &document.external_location {
                        if !serializer.forbids(Term::BT(124)) {
                            serializer.field_leaf(
                                Namespace::Ram,
                                "URIID",
                                "external_location",
                                location.as_str(),
                            );
                        }
                    }
                    serializer.derived(Namespace::Ram, "TypeCode", &[], "916");
                    if let Some(description) = &document.description {
                        serializer.field_leaf(
                            Namespace::Ram,
                            "Name",
                            "description",
                            description.as_ref(),
                        );
                    }
                },
            );
        }
    }

    // Serializes the seller party (`BG-4`).
    fn seller_party(&mut self, seller: &Seller) {
        self.group(
            Namespace::Ram,
            "SellerTradeParty",
            "seller",
            Term::BG(4),
            |serializer| {
                serializer.party_identifiers(&seller.identifiers, "identifiers");
                if let Some(name) = &seller.name {
                    serializer.field_leaf(Namespace::Ram, "Name", "name", name.as_ref());
                }
                if let Some(info) = &seller.additional_legal_information {
                    serializer.field_leaf(
                        Namespace::Ram,
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
                if let Some(address) = &seller.address {
                    serializer.postal_address(address);
                }
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
        self.group(
            Namespace::Ram,
            "BuyerTradeParty",
            "buyer",
            Term::BG(7),
            |serializer| {
                serializer.party_identifiers(&buyer.identifiers, "identifiers");
                if let Some(name) = &buyer.name {
                    serializer.field_leaf(Namespace::Ram, "Name", "name", name.as_ref());
                }
                serializer.legal_organization(
                    buyer.legal_entity.as_ref(),
                    buyer.trading_name.as_deref_ref(),
                );
                if let Some(contact) = &buyer.contact {
                    serializer.contact(contact);
                }
                if let Some(address) = &buyer.address {
                    serializer.postal_address(address);
                }
                if let Some(address) = &buyer.electronic_address {
                    serializer.electronic_address(address);
                }
                if let Some(vat) = &buyer.vat {
                    serializer.tax_registration(&vat.to_string(), "VA", "vat");
                }
            },
        );
    }

    // Serializes the seller tax representative (`BG-11`).
    fn tax_representative_party(&mut self, representative: &TaxRepresentative) {
        self.group(
            Namespace::Ram,
            "SellerTaxRepresentativeTradeParty",
            "tax_representative",
            Term::BG(11),
            |serializer| {
                if let Some(name) = &representative.name {
                    serializer.field_leaf(Namespace::Ram, "Name", "name", name.as_ref());
                }
                if let Some(address) = &representative.address {
                    serializer.postal_address(address);
                }
                if let Some(vat) = &representative.vat {
                    serializer.tax_registration(&vat.to_string(), "VA", "vat");
                }
            },
        );
    }

    // Serializes the party identifiers, with a scheme id when present.
    fn party_identifiers(&mut self, identifiers: &[OperationalEntity], field: &'static str) {
        for identifier in identifiers {
            let Some(id) = &identifier.id else {
                continue;
            };
            match &identifier.issuer {
                Some(issuer) => self.field_leaf_attr(
                    Namespace::Ram,
                    "GlobalID",
                    field,
                    &[("schemeID", &issuer.to_string())],
                    id.as_ref(),
                ),
                None => self.field_leaf(Namespace::Ram, "ID", field, id.as_ref()),
            }
        }
    }

    // Serializes the specified legal organization: the legal id and the trading name.
    fn legal_organization(&mut self, entity: Option<&LegalEntity>, trading_name: Option<&str>) {
        if entity.is_none() && trading_name.is_none() {
            return;
        }
        self.structural(Namespace::Ram, "SpecifiedLegalOrganization", |serializer| {
            if let Some(entity) = entity {
                serializer.legal_entity_id(entity);
            }
            if let Some(name) = trading_name {
                serializer.field_leaf(Namespace::Ram, "TradingBusinessName", "trading_name", name);
            }
        });
    }

    // Serializes the legal registration id (`BT-30`/`BT-47`/`BT-61`), with its scheme when present.
    fn legal_entity_id(&mut self, entity: &LegalEntity) {
        let Some(id) = &entity.id else {
            return;
        };
        match &entity.issuer {
            Some(issuer) => self.field_leaf_attr(
                Namespace::Ram,
                "ID",
                "legal_entity",
                &[("schemeID", &issuer.to_string())],
                id.as_ref(),
            ),
            None => self.field_leaf(Namespace::Ram, "ID", "legal_entity", id.as_ref()),
        }
    }

    // Serializes a defined trade contact (`BG-6`).
    fn contact(&mut self, contact: &Contact) {
        self.group(
            Namespace::Ram,
            "DefinedTradeContact",
            "contact",
            Term::BG(6),
            |serializer| {
                if let Some(name) = &contact.name {
                    serializer.field_leaf(Namespace::Ram, "PersonName", "name", name.as_ref());
                }
                if let Some(phone) = &contact.telephone {
                    serializer.structural(
                        Namespace::Ram,
                        "TelephoneUniversalCommunication",
                        |serializer| {
                            serializer.field_leaf(
                                Namespace::Ram,
                                "CompleteNumber",
                                "telephone",
                                phone.as_ref(),
                            );
                        },
                    );
                }
                if let Some(email) = &contact.email {
                    serializer.structural(
                        Namespace::Ram,
                        "EmailURIUniversalCommunication",
                        |serializer| {
                            serializer.field_leaf(Namespace::Ram, "URIID", "email", email.as_str());
                        },
                    );
                }
            },
        );
    }

    // Serializes a postal trade address.
    fn postal_address(&mut self, address: &PostalAddress) {
        self.field_group(
            Namespace::Ram,
            "PostalTradeAddress",
            "address",
            |serializer| {
                if let Some(zip) = &address.postal_code {
                    serializer.field_leaf(
                        Namespace::Ram,
                        "PostcodeCode",
                        "postal_code",
                        zip.as_ref(),
                    );
                }
                if let Some(line) = &address.line1 {
                    serializer.field_leaf(Namespace::Ram, "LineOne", "line1", line.as_ref());
                }
                if let Some(line) = &address.line2 {
                    serializer.field_leaf(Namespace::Ram, "LineTwo", "line2", line.as_ref());
                }
                if let Some(line) = &address.line3 {
                    serializer.field_leaf(Namespace::Ram, "LineThree", "line3", line.as_ref());
                }
                if let Some(city) = &address.city {
                    serializer.field_leaf(Namespace::Ram, "CityName", "city", city.as_ref());
                }
                if let Some(country) = &address.country {
                    serializer.field_leaf(Namespace::Ram, "CountryID", "country", country.alpha2());
                }
                if let Some(subdivision) = &address.country_subdivision {
                    serializer.field_leaf(
                        Namespace::Ram,
                        "CountrySubDivisionName",
                        "country_subdivision",
                        subdivision.as_ref(),
                    );
                }
            },
        );
    }

    // Serializes a party electronic address (`BT-34`/`BT-49`).
    fn electronic_address(&mut self, address: &ElectronicAddress) {
        let Some(id) = &address.id else {
            return;
        };
        self.structural(
            Namespace::Ram,
            "URIUniversalCommunication",
            |serializer| match &address.scheme {
                Some(scheme) => serializer.field_leaf_attr(
                    Namespace::Ram,
                    "URIID",
                    "electronic_address",
                    &[("schemeID", &scheme.to_string())],
                    id.as_ref(),
                ),
                None => serializer.field_leaf(
                    Namespace::Ram,
                    "URIID",
                    "electronic_address",
                    id.as_ref(),
                ),
            },
        );
    }

    // Serializes a specified tax registration under a scheme id.
    fn tax_registration(&mut self, id: &str, scheme: &str, field: &'static str) {
        self.field_group(
            Namespace::Ram,
            "SpecifiedTaxRegistration",
            field,
            |serializer| {
                serializer.field_leaf_attr(
                    Namespace::Ram,
                    "ID",
                    field,
                    &[("schemeID", scheme)],
                    id,
                );
            },
        );
    }

    // Serializes the header trade delivery (`BG-13`). The wrapper is mandatory in
    // the CII schema, so it is always written, even when empty.
    fn header_trade_delivery(&mut self, invoice: &Invoice) {
        self.structural(
            Namespace::Ram,
            "ApplicableHeaderTradeDelivery",
            |serializer| {
                if let Some(delivery) = &invoice.delivery {
                    serializer.delivery_party(delivery);
                    if let Some(date_value) = delivery.date {
                        serializer.structural(
                            Namespace::Ram,
                            "ActualDeliverySupplyChainEvent",
                            |serializer| {
                                serializer.datetime(
                                    "OccurrenceDateTime",
                                    "date",
                                    Term::BT(72),
                                    date_value,
                                );
                            },
                        );
                    }
                }
                if let Some(reference) = &invoice.despatch_advice_reference {
                    serializer.reference(
                        Namespace::Ram,
                        "DespatchAdviceReferencedDocument",
                        Term::BT(16),
                        "despatch_advice_reference",
                        reference.as_ref(),
                    );
                }
                if let Some(reference) = &invoice.receiving_advice_reference {
                    serializer.reference(
                        Namespace::Ram,
                        "ReceivingAdviceReferencedDocument",
                        Term::BT(15),
                        "receiving_advice_reference",
                        reference.as_ref(),
                    );
                }
            },
        );
    }

    // Serializes the delivery ship-to party (`BG-13` name, location, address).
    fn delivery_party(&mut self, delivery: &Delivery) {
        if delivery.name.is_none() && delivery.location.is_none() && delivery.address.is_none() {
            return;
        }
        self.group(
            Namespace::Ram,
            "ShipToTradeParty",
            "delivery",
            Term::BG(13),
            |serializer| {
                if let Some(id) = delivery
                    .location
                    .as_ref()
                    .and_then(|location| location.id.as_ref())
                {
                    let issuer = delivery
                        .location
                        .as_ref()
                        .and_then(|location| location.issuer);
                    match issuer {
                        Some(issuer) => serializer.field_leaf_attr(
                            Namespace::Ram,
                            "ID",
                            "location",
                            &[("schemeID", &issuer.to_string())],
                            id.as_ref(),
                        ),
                        None => {
                            serializer.field_leaf(Namespace::Ram, "ID", "location", id.as_ref())
                        }
                    }
                }
                if let Some(name) = &delivery.name {
                    serializer.field_leaf(Namespace::Ram, "Name", "name", name.as_ref());
                }
                if let Some(address) = &delivery.address {
                    serializer.postal_address(address);
                }
            },
        );
    }

    // Serializes the header trade settlement.
    fn header_trade_settlement(&mut self, invoice: &Invoice) {
        let currency = invoice.currency.as_ref().map(Currency::code);
        let attribute: Vec<(&str, &str)> = currency
            .map(|currency| vec![("currencyID", currency)])
            .unwrap_or_default();
        self.structural(
            Namespace::Ram,
            "ApplicableHeaderTradeSettlement",
            |serializer| {
                if let Some(creditor) = invoice.payment.as_ref().and_then(direct_debit_creditor) {
                    serializer.leaf(
                        Namespace::Ram,
                        "CreditorReferenceID",
                        "creditor_identifier",
                        Term::BT(90),
                        creditor,
                    );
                }
                if let Some(payment) = &invoice.payment {
                    if let Some(reference) = &payment.remittance_information {
                        serializer.leaf(
                            Namespace::Ram,
                            "PaymentReference",
                            "remittance_information",
                            Term::BT(83),
                            reference.as_ref(),
                        );
                    }
                }
                if let Some(accounting) = &invoice.vat_accounting_total {
                    serializer.leaf(
                        Namespace::Ram,
                        "TaxCurrencyCode",
                        "vat_accounting_total",
                        Term::BT(6),
                        accounting.currency.code(),
                    );
                }
                if let Some(currency) = currency {
                    serializer.leaf(
                        Namespace::Ram,
                        "InvoiceCurrencyCode",
                        "currency",
                        Term::BT(5),
                        currency,
                    );
                }
                if let Some(payee) = &invoice.payee {
                    serializer.payee_party(payee);
                }
                if let Some(payment) = &invoice.payment {
                    serializer.payment_means(payment);
                }
                serializer.tax_breakdown(invoice);
                if let Some(period) = invoice.invoicing_period {
                    serializer.billing_period(period, "invoicing_period", Term::BG(14));
                }
                serializer.adjustments(&invoice.adjustments);
                serializer.payment_terms(invoice);
                serializer.monetary_summation(invoice, &attribute);
                serializer.preceding_invoices(&invoice.preceding_invoices);
                if let Some(reference) = &invoice.buyer_accounting_reference {
                    serializer.field_group(
                        Namespace::Ram,
                        "ReceivableSpecifiedTradeAccountingAccount",
                        "buyer_accounting_reference",
                        |serializer| {
                            serializer.derived(Namespace::Ram, "ID", &[], reference.as_ref());
                        },
                    );
                }
            },
        );
    }

    // Serializes the payee party (`BG-10`).
    fn payee_party(&mut self, payee: &Payee) {
        self.group(
            Namespace::Ram,
            "PayeeTradeParty",
            "payee",
            Term::BG(10),
            |serializer| {
                serializer.party_identifiers(&payee.identifiers, "identifiers");
                if let Some(name) = &payee.name {
                    serializer.field_leaf(Namespace::Ram, "Name", "name", name.as_ref());
                }
                if let Some(entity) = &payee.legal_entity {
                    serializer.structural(
                        Namespace::Ram,
                        "SpecifiedLegalOrganization",
                        |serializer| serializer.legal_entity_id(entity),
                    );
                }
            },
        );
    }

    // Serializes the payment means (`BG-16`).
    fn payment_means(&mut self, payment: &PaymentInstructions) {
        self.group(
            Namespace::Ram,
            "SpecifiedTradeSettlementPaymentMeans",
            "payment",
            Term::BG(16),
            |serializer| {
                if let Some(means) = &payment.means {
                    serializer.derived(Namespace::Ram, "TypeCode", &[], &means.to_string());
                }
                if let Some(text) = &payment.means_text {
                    serializer.derived(Namespace::Ram, "Information", &[], text.as_ref());
                }
                match &payment.details {
                    Some(PaymentDetails::Card(card)) => {
                        serializer.nested(
                            Namespace::Ram,
                            "ApplicableTradeSettlementFinancialCard",
                            |serializer| {
                                if let Some(number) = &card.primary_account_number {
                                    serializer.derived(Namespace::Ram, "ID", &[], number.as_ref());
                                }
                                if let Some(holder) = &card.holder_name {
                                    serializer.derived(
                                        Namespace::Ram,
                                        "CardholderName",
                                        &[],
                                        holder.as_ref(),
                                    );
                                }
                            },
                        );
                    }
                    Some(PaymentDetails::DirectDebit(debit)) => {
                        if let Some(account) = &debit.debited_account {
                            serializer.nested(
                                Namespace::Ram,
                                "PayerPartyDebtorFinancialAccount",
                                |serializer| {
                                    serializer.derived(
                                        Namespace::Ram,
                                        "IBANID",
                                        &[],
                                        account.as_ref(),
                                    );
                                },
                            );
                        }
                    }
                    Some(PaymentDetails::CreditTransfers(transfers)) => {
                        for transfer in transfers {
                            serializer.nested(
                                Namespace::Ram,
                                "PayeePartyCreditorFinancialAccount",
                                |serializer| {
                                    if let Some(account) = &transfer.account {
                                        serializer.derived(
                                            Namespace::Ram,
                                            "IBANID",
                                            &[],
                                            account.as_ref(),
                                        );
                                    }
                                    if let Some(name) = &transfer.account_name {
                                        serializer.derived(
                                            Namespace::Ram,
                                            "AccountName",
                                            &[],
                                            name.as_ref(),
                                        );
                                    }
                                },
                            );
                            if let Some(provider) = &transfer.provider {
                                serializer.nested(
                                    Namespace::Ram,
                                    "PayeeSpecifiedCreditorFinancialInstitution",
                                    |serializer| {
                                        serializer.derived(
                                            Namespace::Ram,
                                            "BICID",
                                            &[],
                                            provider.as_ref(),
                                        );
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

    // Serializes the VAT breakdown (`BG-23`), each group an instance mapped to its own fields.
    fn tax_breakdown(&mut self, invoice: &Invoice) {
        let event = match invoice.vat_point {
            Some(VatPoint::Event(event)) => Some(u16::from(event).to_string()),
            _ => None,
        };
        for (position, group) in invoice.vat_breakdown.iter().enumerate() {
            let instance = index(position);
            self.repeatable(
                Namespace::Ram,
                "ApplicableTradeTax",
                "vat_breakdown",
                Term::BG(23),
                instance,
                |serializer| {
                    if let Some(tax) = group.tax {
                        serializer.leaf(
                            Namespace::Ram,
                            "CalculatedAmount",
                            "tax",
                            Term::BT(117),
                            &money(tax),
                        );
                    }
                    serializer.derived(Namespace::Ram, "TypeCode", &[], "VAT");
                    if let Some(VatTreatment::Exempt {
                        text: Some(text), ..
                    }) = &group.treatment
                    {
                        serializer.field_leaf(
                            Namespace::Ram,
                            "ExemptionReason",
                            "treatment",
                            text.as_ref(),
                        );
                    }
                    if let Some(taxable) = group.taxable {
                        serializer.leaf(
                            Namespace::Ram,
                            "BasisAmount",
                            "taxable",
                            Term::BT(116),
                            &money(taxable),
                        );
                    }
                    if let Some(treatment) = &group.treatment {
                        serializer.field_leaf(
                            Namespace::Ram,
                            "CategoryCode",
                            "treatment",
                            &treatment.category().to_string(),
                        );
                    }
                    if let Some(VatTreatment::Exempt {
                        code: Some(code), ..
                    }) = &group.treatment
                    {
                        serializer.field_leaf(
                            Namespace::Ram,
                            "ExemptionReasonCode",
                            "treatment",
                            &code.to_string(),
                        );
                    }
                    if let Some(event) = &event {
                        serializer.derived(Namespace::Ram, "DueDateTypeCode", &[], event);
                    }
                    if let Some(treatment) = &group.treatment {
                        serializer.field_leaf(
                            Namespace::Ram,
                            "RateApplicablePercent",
                            "treatment",
                            &plain(treatment.rate()),
                        );
                    }
                },
            );
        }
    }

    // Serializes a billing period (`BG-14`/`BG-26`).
    fn billing_period(&mut self, period: Period, field: &'static str, term: Term) {
        self.group(
            Namespace::Ram,
            "BillingSpecifiedPeriod",
            field,
            term,
            |serializer| {
                if let Some(start) = period.start() {
                    serializer.datetime("StartDateTime", field, term, start);
                }
                if let Some(end) = period.end() {
                    serializer.datetime("EndDateTime", field, term, end);
                }
            },
        );
    }

    // Serializes the document-level allowances and charges (`BG-20`/`BG-21`).
    fn adjustments(&mut self, adjustments: &[Adjustment]) {
        for (position, adjustment) in adjustments.iter().enumerate() {
            let instance = index(position);
            let charge = adjustment
                .reason
                .as_ref()
                .map(|reason| matches!(reason, AdjustmentReason::Charge { .. }));
            let term = if charge == Some(true) {
                Term::BG(21)
            } else {
                Term::BG(20)
            };
            self.repeatable(
                Namespace::Ram,
                "SpecifiedTradeAllowanceCharge",
                "adjustments",
                term,
                instance,
                |serializer| {
                    if let Some(charge) = charge {
                        serializer.indicator(charge);
                    }
                    if let Some(amount) = &adjustment.amount {
                        serializer.adjustment_amount(amount);
                    }
                    if let Some(reason) = &adjustment.reason {
                        serializer.adjustment_reason(reason);
                    }
                    if let Some(vat) = &adjustment.vat {
                        serializer.nested(Namespace::Ram, "CategoryTradeTax", |serializer| {
                            serializer.derived(Namespace::Ram, "TypeCode", &[], "VAT");
                            serializer.derived(
                                Namespace::Ram,
                                "CategoryCode",
                                &[],
                                &vat.category().to_string(),
                            );
                            serializer.derived(
                                Namespace::Ram,
                                "RateApplicablePercent",
                                &[],
                                &plain(vat.rate()),
                            );
                        });
                    }
                },
            );
        }
    }

    // Serializes the amount of an adjustment, mapped to the amount field.
    fn adjustment_amount(&mut self, amount: &AdjustmentAmount) {
        let actual = match amount {
            AdjustmentAmount::Relative { amount, rate, base } => {
                self.field_leaf(
                    Namespace::Ram,
                    "CalculationPercent",
                    "amount",
                    &plain(Decimal::from(*rate)),
                );
                self.field_leaf(Namespace::Ram, "BasisAmount", "amount", &money(*base));
                amount
            }
            AdjustmentAmount::Absolute(amount) => amount,
        };
        self.field_leaf(Namespace::Ram, "ActualAmount", "amount", &money(*actual));
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
            self.derived(Namespace::Ram, "ReasonCode", &[], &code);
        }
        if let Some(text) = text {
            self.derived(Namespace::Ram, "Reason", &[], text.as_ref());
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
        self.structural(Namespace::Ram, "SpecifiedTradePaymentTerms", |serializer| {
            if let Some(terms) = &invoice.payment_terms {
                serializer.field_leaf(
                    Namespace::Ram,
                    "Description",
                    "payment_terms",
                    terms.as_ref(),
                );
            }
            if let Some(due) = invoice.payment_due_date {
                serializer.datetime("DueDateDateTime", "payment_due_date", Term::BT(9), due);
            }
            if let Some(mandate) = mandate {
                serializer.field_leaf(
                    Namespace::Ram,
                    "DirectDebitMandateID",
                    "mandate_reference",
                    mandate,
                );
            }
        });
    }

    // Serializes the header monetary summation, each amount mapped to its own field.
    // The whole group is absent when the invoice states none of its amounts.
    fn monetary_summation(&mut self, invoice: &Invoice, currency: &[(&str, &str)]) {
        let totals: [(_, _, _, &[(&str, &str)], _); 9] = [
            (
                "LineTotalAmount",
                "line_net_total",
                Term::BT(106),
                &[],
                invoice.line_net_total,
            ),
            (
                "ChargeTotalAmount",
                "charges_total",
                Term::BT(108),
                &[],
                invoice.charges_total,
            ),
            (
                "AllowanceTotalAmount",
                "allowances_total",
                Term::BT(107),
                &[],
                invoice.allowances_total,
            ),
            (
                "TaxBasisTotalAmount",
                "net_total",
                Term::BT(109),
                &[],
                invoice.net_total,
            ),
            (
                "TaxTotalAmount",
                "vat_total",
                Term::BT(110),
                currency,
                invoice.vat_total,
            ),
            (
                "RoundingAmount",
                "rounding",
                Term::BT(114),
                &[],
                invoice.rounding,
            ),
            (
                "GrandTotalAmount",
                "gross_total",
                Term::BT(112),
                &[],
                invoice.gross_total,
            ),
            (
                "TotalPrepaidAmount",
                "paid",
                Term::BT(113),
                &[],
                invoice.paid,
            ),
            ("DuePayableAmount", "due", Term::BT(115), &[], invoice.due),
        ];
        if totals.iter().all(|(_, _, _, _, value)| value.is_none()) {
            return;
        }
        self.structural(
            Namespace::Ram,
            "SpecifiedTradeSettlementHeaderMonetarySummation",
            |serializer| {
                for (name, field, term, attributes, value) in totals {
                    if let Some(value) = value {
                        serializer.leaf_attr(
                            Namespace::Ram,
                            name,
                            field,
                            term,
                            attributes,
                            &money(value),
                        );
                    }
                }
            },
        );
    }

    // Serializes the preceding invoice references (`BG-3`).
    fn preceding_invoices(&mut self, preceding: &[PrecedingInvoice]) {
        for (position, invoice) in preceding.iter().enumerate() {
            let instance = index(position);
            self.repeatable(
                Namespace::Ram,
                "InvoiceReferencedDocument",
                "preceding_invoices",
                Term::BG(3),
                instance,
                |serializer| {
                    if let Some(number) = &invoice.number {
                        serializer.derived(
                            Namespace::Ram,
                            "IssuerAssignedID",
                            &[],
                            number.as_ref(),
                        );
                    }
                    if let Some(issued) = invoice.issue_date {
                        serializer.nested(Namespace::Ram, "FormattedIssueDateTime", |serializer| {
                            serializer.write_raw_datetime(Namespace::Qdt, issued);
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
            serializer.derived(Namespace::Ram, "IssuerAssignedID", &[], id);
        });
    }

    // ---- datatype carriers (not recorded) --------------------------------

    // Writes a date wrapper whose value carrier is a `udt:DateTimeString`.
    fn datetime(&mut self, element: &str, field: &'static str, term: Term, value: Date) {
        self.group(Namespace::Ram, element, field, term, |serializer| {
            serializer.write_raw_datetime(Namespace::Udt, value);
        });
    }

    // Writes a charge indicator whose value carrier is a `udt:Indicator`.
    fn indicator(&mut self, charge: bool) {
        self.nested(Namespace::Ram, "ChargeIndicator", |serializer| {
            serializer.write_element(
                Namespace::Udt,
                "Indicator",
                &[],
                if charge { "true" } else { "false" },
            );
        });
    }

    // Writes a `DateTimeString` value carrier in the given datatype namespace.
    fn write_raw_datetime(&mut self, namespace: Namespace, value: Date) {
        self.write_element(
            namespace,
            "DateTimeString",
            &[("format", "102")],
            &date(value),
        );
    }
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
