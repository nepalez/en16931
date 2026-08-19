use base64::Engine as _;

use crate::format::trace::Trace;
use crate::format::ubl::{CAC, CBC, INV, prefix};
use crate::prelude::*;
use crate::{
    Abbreviations, Adjustment, AdjustmentAmount, AdjustmentReason, BinaryObject, Buyer, Contact,
    Delivery, Dictionary, DocumentBuilder, ElectronicAddress, Invoice, InvoiceLine, Item,
    LegalEntity, LineAdjustment, Namespace, Note, ObjectReference, OperationalEntity, Payee,
    PaymentDetails, PaymentInstructions, Period, PostalAddress, PrecedingInvoice, Seller,
    SupportingDocument, TaxRepresentative, Term, VatPoint, VatTreatment,
};

/// Serializes a `DocumentBuilder` to UBL XML,
/// returning the document, its dictionary, and its abbreviations.
pub(crate) fn serialize(builder: &DocumentBuilder) -> (String, Dictionary, Abbreviations) {
    let mut serializer = Serializer::new(builder);
    serializer.document(builder);
    serializer.finish()
}

// Renders a date as an ISO-8601 calendar date (`YYYY-MM-DD`), the UBL form.
fn date(value: Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
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

// Encodes a note as `#subject#text`, or bare text when the subject is dropped.
fn note_text(note: &Note, drop_subject: bool) -> String {
    match &note.subject_code {
        Some(code) if !drop_subject => format!("#{}#{}", code.as_ref(), note.text.as_ref()),
        _ => note.text.as_ref().to_owned(),
    }
}

/// The stateful UBL writer: an XML sink plus the trace that builds the dictionary in lockstep.
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

    // Whether the profile forbids the term of a node about to be written.
    fn forbids(&self, term: Term) -> bool {
        self.forbidden.contains(&term)
    }

    // Serializes the whole document under the UBL root element.
    fn document(&mut self, builder: &DocumentBuilder) {
        let invoice = &builder.invoice;
        let root = BytesStart::new("Invoice").with_attributes([
            ("xmlns", INV.uri()),
            ("xmlns:cac", CAC.uri()),
            ("xmlns:cbc", CBC.uri()),
        ]);
        self.write(Event::Start(root));
        for namespace in [INV, CAC, CBC] {
            self.abbreviations
                .declare(prefix(namespace), namespace)
                .expect("the writer binds each abbreviation to one namespace");
        }
        self.trace.enter(INV, "Invoice");
        self.trace.record_root();

        // Regulatory-flow fields: the specification identifier (BT-24) and business process (BT-23).
        self.rooted(CBC, "CustomizationID", &[], &builder.profile.to_string());
        if let Some(process) = &builder.business_process {
            self.rooted(CBC, "ProfileID", &[], process.as_ref());
        }

        self.leaf(CBC, "ID", "number", Term::BT(1), invoice.number.as_ref());
        self.leaf(
            CBC,
            "IssueDate",
            "issue_date",
            Term::BT(2),
            &date(invoice.issue_date),
        );
        if let Some(due) = invoice.payment_due_date {
            self.leaf(CBC, "DueDate", "payment_due_date", Term::BT(9), &date(due));
        }
        self.leaf(
            CBC,
            "InvoiceTypeCode",
            "type_code",
            Term::BT(3),
            &invoice.type_code.to_string(),
        );
        self.notes(&invoice.notes);
        if let Some(VatPointDate(point)) = self.tax_point_date(invoice) {
            self.leaf(CBC, "TaxPointDate", "vat_point", Term::BT(7), &date(point));
        }
        self.leaf(
            CBC,
            "DocumentCurrencyCode",
            "currency",
            Term::BT(5),
            invoice.currency.code(),
        );
        if let Some(accounting) = &invoice.vat_accounting_total {
            self.leaf(
                CBC,
                "TaxCurrencyCode",
                "vat_accounting_total",
                Term::BT(6),
                accounting.currency.code(),
            );
        }
        if let Some(reference) = &invoice.buyer_accounting_reference {
            self.leaf(
                CBC,
                "AccountingCost",
                "buyer_accounting_reference",
                Term::BT(19),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.buyer_reference {
            self.leaf(
                CBC,
                "BuyerReference",
                "buyer_reference",
                Term::BT(10),
                reference.as_ref(),
            );
        }

        self.invoice_period(invoice);
        self.order_reference(invoice);
        self.billing_references(&invoice.preceding_invoices);
        if let Some(reference) = &invoice.despatch_advice_reference {
            self.reference_group(
                CAC,
                "DespatchDocumentReference",
                Term::BT(16),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.receiving_advice_reference {
            self.reference_group(
                CAC,
                "ReceiptDocumentReference",
                Term::BT(15),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.tender_or_lot_reference {
            self.reference_group(
                CAC,
                "OriginatorDocumentReference",
                Term::BT(17),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.contract_reference {
            self.reference_group(
                CAC,
                "ContractDocumentReference",
                Term::BT(12),
                reference.as_ref(),
            );
        }
        self.additional_documents(invoice);
        if let Some(reference) = &invoice.project_reference {
            self.reference_group(CAC, "ProjectReference", Term::BT(11), reference.as_ref());
        }

        self.supplier_party(&invoice.seller);
        self.customer_party(&invoice.buyer);
        if let Some(payee) = &invoice.payee {
            self.payee_party(payee);
        }
        if let Some(representative) = &invoice.tax_representative {
            self.tax_representative_party(representative);
        }
        if let Some(delivery) = &invoice.delivery {
            self.delivery(delivery);
        }
        if let Some(payment) = &invoice.payment {
            self.payment_means(payment);
        }
        if let Some(terms) = &invoice.payment_terms {
            self.group(
                CAC,
                "PaymentTerms",
                "payment_terms",
                Term::BT(20),
                |serializer| {
                    serializer.field_leaf(CBC, "Note", "payment_terms", terms.as_ref());
                },
            );
        }
        self.adjustments(&invoice.adjustments);
        self.tax_total(invoice);
        self.legal_monetary_total(invoice);
        self.lines(&invoice.lines);

        self.trace.leave();
        self.write(Event::End(BytesEnd::new("Invoice")));
    }

    // Serializes the notes (`BG-1`), each a repeatable single-value element.
    fn notes(&mut self, notes: &[Note]) {
        let drop_subject = self.forbids(Term::BT(21));
        for (position, note) in notes.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive note index");
            let text = note_text(note, drop_subject);
            self.repeatable_leaf(CBC, "Note", "notes", Term::BG(1), instance, &text);
        }
    }

    // Serializes the invoice period (`BG-14`) and the VAT point date code (`BT-8`).
    fn invoice_period(&mut self, invoice: &Invoice) {
        let event = self.vat_point_event(invoice);
        if invoice.invoicing_period.is_none() && event.is_none() {
            return;
        }
        self.group(
            CAC,
            "InvoicePeriod",
            "invoicing_period",
            Term::BG(14),
            |serializer| {
                if let Some(period) = invoice.invoicing_period {
                    if let Some(start) = period.start() {
                        serializer.field_leaf(CBC, "StartDate", "invoicing_period", &date(start));
                    }
                    if let Some(end) = period.end() {
                        serializer.field_leaf(CBC, "EndDate", "invoicing_period", &date(end));
                    }
                }
                if let Some(code) = event {
                    serializer.field_leaf(CBC, "DescriptionCode", "vat_point", &code);
                }
            },
        );
    }

    // Serializes the order and sales order references (`BT-13`/`BT-14`).
    fn order_reference(&mut self, invoice: &Invoice) {
        if invoice.purchase_order_reference.is_none() && invoice.sales_order_reference.is_none() {
            return;
        }
        self.structural(CAC, "OrderReference", |serializer| {
            if let Some(order) = &invoice.purchase_order_reference {
                serializer.leaf(
                    CBC,
                    "ID",
                    "purchase_order_reference",
                    Term::BT(13),
                    order.as_ref(),
                );
            }
            if let Some(sales) = &invoice.sales_order_reference {
                serializer.leaf(
                    CBC,
                    "SalesOrderID",
                    "sales_order_reference",
                    Term::BT(14),
                    sales.as_ref(),
                );
            }
        });
    }

    // Serializes the preceding invoice references (`BG-3`).
    fn billing_references(&mut self, preceding: &[PrecedingInvoice]) {
        for (position, invoice) in preceding.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive reference index");
            self.repeatable(
                CAC,
                "BillingReference",
                "preceding_invoices",
                Term::BG(3),
                instance,
                |serializer| {
                    serializer.structural(CAC, "InvoiceDocumentReference", |serializer| {
                        serializer.field_leaf(CBC, "ID", "number", invoice.number.as_ref());
                        if let Some(issued) = invoice.issue_date {
                            serializer.field_leaf(CBC, "IssueDate", "issue_date", &date(issued));
                        }
                    });
                },
            );
        }
    }

    // Serializes a document reference carrying a single identifier.
    fn reference_group(&mut self, namespace: Namespace, element: &str, term: Term, id: &str) {
        self.group(namespace, element, field_of(term), term, |serializer| {
            serializer.field_leaf(CBC, "ID", field_of(term), id);
        });
    }

    // Serializes the invoiced object (`BT-18`) and the supporting documents (`BG-24`).
    fn additional_documents(&mut self, invoice: &Invoice) {
        if let Some(object) = &invoice.object {
            self.additional_object(object);
        }
        for (position, document) in invoice.supporting_documents.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive reference index");
            self.additional_supporting(document, instance);
        }
    }

    // Serializes the invoiced object identifier as an additional document reference.
    fn additional_object(&mut self, object: &ObjectReference) {
        self.group(
            CAC,
            "AdditionalDocumentReference",
            "object",
            Term::BT(18),
            |serializer| {
                match &object.scheme {
                    Some(scheme) => serializer.derived(
                        CBC,
                        "ID",
                        &[("schemeID", &scheme.to_string())],
                        object.id.as_ref(),
                    ),
                    None => serializer.derived(CBC, "ID", &[], object.id.as_ref()),
                }
                serializer.derived(CBC, "DocumentTypeCode", &[], "130");
            },
        );
    }

    // Serializes a supporting document (`BG-24`) as an additional document reference.
    fn additional_supporting(&mut self, document: &SupportingDocument, instance: NonZeroUsize) {
        self.repeatable(
            CAC,
            "AdditionalDocumentReference",
            "supporting_documents",
            Term::BG(24),
            instance,
            |serializer| {
                serializer.field_leaf(CBC, "ID", "reference", document.reference.as_ref());
                if let Some(description) = &document.description {
                    serializer.leaf(
                        CBC,
                        "DocumentDescription",
                        "description",
                        Term::BT(123),
                        description.as_ref(),
                    );
                }
                if let Some(location) = &document.external_location {
                    if !serializer.forbids(Term::BT(124)) {
                        serializer.structural(CAC, "Attachment", |serializer| {
                            serializer.structural(CAC, "ExternalReference", |serializer| {
                                serializer.field_leaf(
                                    CBC,
                                    "URI",
                                    "external_location",
                                    location.as_str(),
                                );
                            });
                        });
                    }
                }
                if let Some(binary) = &document.attachment {
                    serializer.structural(CAC, "Attachment", |serializer| {
                        serializer.embedded_binary(binary);
                    });
                }
            },
        );
    }

    // Serializes an embedded binary attachment (`BT-125`).
    fn embedded_binary(&mut self, binary: &BinaryObject) {
        let encoded = base64::engine::general_purpose::STANDARD.encode(binary.content());
        self.field_leaf_attr(
            CBC,
            "EmbeddedDocumentBinaryObject",
            "attachment",
            &[
                ("mimeCode", &binary.mime_code().to_string()),
                ("filename", binary.filename()),
            ],
            &encoded,
        );
    }

    // Serializes the seller (`BG-4`) under the supplier party wrapper.
    fn supplier_party(&mut self, seller: &Seller) {
        self.structural(CAC, "AccountingSupplierParty", |serializer| {
            serializer.group(CAC, "Party", "seller", Term::BG(4), |serializer| {
                if let Some(address) = &seller.electronic_address {
                    serializer.endpoint(address, Term::BT(34));
                }
                serializer.party_identifiers(&seller.identifiers, Term::BT(29));
                if let Some(trading) = &seller.trading_name {
                    serializer.party_name(trading.as_ref(), Term::BT(28));
                }
                serializer.postal_address("PostalAddress", Term::BG(5), &seller.address);
                if let Some(vat) = &seller.vat {
                    serializer.party_tax_scheme(&vat.to_string(), "VAT", Term::BT(31));
                }
                if let Some(registration) = &seller.tax_registration {
                    serializer.party_tax_scheme(registration.as_ref(), "FC", Term::BT(32));
                }
                serializer.party_legal_entity(
                    seller.name.as_ref(),
                    Term::BT(27),
                    seller.legal_entity.as_ref(),
                    Term::BT(30),
                    seller
                        .additional_legal_information
                        .as_ref()
                        .map(|value| value.as_ref()),
                );
                if let Some(contact) = &seller.contact {
                    serializer.contact(
                        contact,
                        Term::BG(6),
                        Term::BT(41),
                        Term::BT(42),
                        Term::BT(43),
                    );
                }
            });
        });
    }

    // Serializes the buyer (`BG-7`) under the customer party wrapper.
    fn customer_party(&mut self, buyer: &Buyer) {
        self.structural(CAC, "AccountingCustomerParty", |serializer| {
            serializer.group(CAC, "Party", "buyer", Term::BG(7), |serializer| {
                if let Some(address) = &buyer.electronic_address {
                    serializer.endpoint(address, Term::BT(49));
                }
                serializer.party_identifiers(&buyer.identifiers, Term::BT(46));
                if let Some(trading) = &buyer.trading_name {
                    serializer.party_name(trading.as_ref(), Term::BT(45));
                }
                serializer.postal_address("PostalAddress", Term::BG(8), &buyer.address);
                if let Some(vat) = &buyer.vat {
                    serializer.party_tax_scheme(&vat.to_string(), "VAT", Term::BT(48));
                }
                serializer.party_legal_entity(
                    buyer.name.as_ref(),
                    Term::BT(44),
                    buyer.legal_entity.as_ref(),
                    Term::BT(47),
                    None,
                );
                if let Some(contact) = &buyer.contact {
                    serializer.contact(
                        contact,
                        Term::BG(9),
                        Term::BT(56),
                        Term::BT(57),
                        Term::BT(58),
                    );
                }
            });
        });
    }

    // Serializes the payee (`BG-10`).
    fn payee_party(&mut self, payee: &Payee) {
        self.group(CAC, "PayeeParty", "payee", Term::BG(10), |serializer| {
            serializer.party_identifiers(&payee.identifiers, Term::BT(60));
            serializer.party_name(payee.name.as_ref(), Term::BT(59));
            if let Some(entity) = &payee.legal_entity {
                serializer.structural(CAC, "PartyLegalEntity", |serializer| {
                    serializer.legal_entity_id(entity, Term::BT(61));
                });
            }
        });
    }

    // Serializes the seller tax representative (`BG-11`).
    fn tax_representative_party(&mut self, representative: &TaxRepresentative) {
        self.group(
            CAC,
            "TaxRepresentativeParty",
            "tax_representative",
            Term::BG(11),
            |serializer| {
                serializer.party_name(representative.name.as_ref(), Term::BT(62));
                serializer.postal_address("PostalAddress", Term::BG(12), &representative.address);
                serializer.party_tax_scheme(&representative.vat.to_string(), "VAT", Term::BT(63));
            },
        );
    }

    // Serializes a party endpoint electronic address (`BT-34`/`BT-49`).
    fn endpoint(&mut self, address: &ElectronicAddress, term: Term) {
        self.leaf_attr(
            CBC,
            "EndpointID",
            field_of(term),
            term,
            &[("schemeID", &address.scheme.to_string())],
            address.id.as_ref(),
        );
    }

    // Serializes the party identifiers (`BT-29`/`BT-46`/`BT-60`).
    fn party_identifiers(&mut self, identifiers: &[OperationalEntity], term: Term) {
        for identifier in identifiers {
            self.group(
                CAC,
                "PartyIdentification",
                field_of(term),
                term,
                |serializer| match &identifier.issuer {
                    Some(issuer) => serializer.field_leaf_attr(
                        CBC,
                        "ID",
                        field_of(term),
                        &[("schemeID", &issuer.to_string())],
                        identifier.id.as_ref(),
                    ),
                    None => {
                        serializer.field_leaf(CBC, "ID", field_of(term), identifier.id.as_ref())
                    }
                },
            );
        }
    }

    // Serializes a party trading name (`BT-28`/`BT-45`).
    fn party_name(&mut self, name: &str, term: Term) {
        self.group(CAC, "PartyName", field_of(term), term, |serializer| {
            serializer.field_leaf(CBC, "Name", field_of(term), name);
        });
    }

    // Serializes a party tax scheme carrying a company id under a scheme code.
    fn party_tax_scheme(&mut self, company: &str, scheme: &str, term: Term) {
        self.group(CAC, "PartyTaxScheme", field_of(term), term, |serializer| {
            serializer.field_leaf(CBC, "CompanyID", field_of(term), company);
            serializer.structural(CAC, "TaxScheme", |serializer| {
                serializer.field_leaf(CBC, "ID", field_of(term), scheme);
            });
        });
    }

    // Serializes the party legal entity: the registration name, the legal id, and the legal form.
    fn party_legal_entity(
        &mut self,
        name: &str,
        name_term: Term,
        entity: Option<&LegalEntity>,
        entity_term: Term,
        legal_form: Option<&str>,
    ) {
        self.structural(CAC, "PartyLegalEntity", |serializer| {
            serializer.leaf(
                CBC,
                "RegistrationName",
                field_of(name_term),
                name_term,
                name,
            );
            if let Some(entity) = entity {
                serializer.legal_entity_id(entity, entity_term);
            }
            if let Some(form) = legal_form {
                serializer.leaf(
                    CBC,
                    "CompanyLegalForm",
                    "additional_legal_information",
                    Term::BT(33),
                    form,
                );
            }
        });
    }

    // Serializes the legal registration id (`BT-30`/`BT-47`/`BT-61`).
    fn legal_entity_id(&mut self, entity: &LegalEntity, term: Term) {
        match &entity.issuer {
            Some(issuer) => self.leaf_attr(
                CBC,
                "CompanyID",
                field_of(term),
                term,
                &[("schemeID", &issuer.to_string())],
                entity.id.as_ref(),
            ),
            None => self.leaf(CBC, "CompanyID", field_of(term), term, entity.id.as_ref()),
        }
    }

    // Serializes a contact group (`BG-6`/`BG-9`).
    fn contact(&mut self, contact: &Contact, group: Term, name: Term, phone: Term, mail: Term) {
        self.group(CAC, "Contact", field_of(group), group, |serializer| {
            if let Some(value) = &contact.name {
                serializer.leaf(CBC, "Name", field_of(name), name, value.as_ref());
            }
            if let Some(value) = &contact.telephone {
                serializer.leaf(CBC, "Telephone", field_of(phone), phone, value.as_ref());
            }
            if let Some(value) = &contact.email {
                serializer.leaf(CBC, "ElectronicMail", field_of(mail), mail, value.as_str());
            }
        });
    }

    // Serializes a postal address (`BG-5`/`BG-8`/`BG-12`/`BG-15`).
    fn postal_address(&mut self, element: &str, group: Term, address: &PostalAddress) {
        self.group(CAC, element, "address", group, |serializer| {
            if let Some(line) = &address.line1 {
                serializer.field_leaf(CBC, "StreetName", "line1", line.as_ref());
            }
            if let Some(line) = &address.line2 {
                serializer.field_leaf(CBC, "AdditionalStreetName", "line2", line.as_ref());
            }
            if let Some(city) = &address.city {
                serializer.field_leaf(CBC, "CityName", "city", city.as_ref());
            }
            if let Some(zip) = &address.postal_code {
                serializer.field_leaf(CBC, "PostalZone", "postal_code", zip.as_ref());
            }
            if let Some(subdivision) = &address.country_subdivision {
                serializer.field_leaf(
                    CBC,
                    "CountrySubentity",
                    "country_subdivision",
                    subdivision.as_ref(),
                );
            }
            if let Some(line) = &address.line3 {
                serializer.structural(CAC, "AddressLine", |serializer| {
                    serializer.field_leaf(CBC, "Line", "line3", line.as_ref());
                });
            }
            serializer.structural(CAC, "Country", |serializer| {
                serializer.field_leaf(
                    CBC,
                    "IdentificationCode",
                    "country",
                    address.country.alpha2(),
                );
            });
        });
    }

    // Serializes the delivery information (`BG-13`).
    fn delivery(&mut self, delivery: &Delivery) {
        self.group(CAC, "Delivery", "delivery", Term::BG(13), |serializer| {
            if let Some(date_value) = delivery.date {
                serializer.field_leaf(CBC, "ActualDeliveryDate", "date", &date(date_value));
            }
            if delivery.location.is_some() || delivery.address.is_some() {
                serializer.structural(CAC, "DeliveryLocation", |serializer| {
                    if let Some(location) = &delivery.location {
                        match &location.issuer {
                            Some(issuer) => serializer.field_leaf_attr(
                                CBC,
                                "ID",
                                "location",
                                &[("schemeID", &issuer.to_string())],
                                location.id.as_ref(),
                            ),
                            None => {
                                serializer.field_leaf(CBC, "ID", "location", location.id.as_ref())
                            }
                        }
                    }
                    if let Some(address) = &delivery.address {
                        serializer.postal_address("Address", Term::BG(15), address);
                    }
                });
            }
            if let Some(name) = &delivery.name {
                serializer.structural(CAC, "DeliveryParty", |serializer| {
                    serializer.party_name(name.as_ref(), Term::BT(70));
                });
            }
        });
    }

    // Serializes the payment instructions (`BG-16`).
    fn payment_means(&mut self, payment: &PaymentInstructions) {
        self.group(CAC, "PaymentMeans", "payment", Term::BG(16), |serializer| {
            let attributes: Vec<(String, String)> = match &payment.means_text {
                Some(text) => vec![("name".to_owned(), text.as_ref().to_owned())],
                None => Vec::new(),
            };
            let borrowed: Vec<(&str, &str)> = attributes
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect();
            serializer.field_leaf_attr(
                CBC,
                "PaymentMeansCode",
                "means",
                &borrowed,
                &payment.means.to_string(),
            );
            if let Some(reference) = &payment.remittance_information {
                serializer.field_leaf(
                    CBC,
                    "PaymentID",
                    "remittance_information",
                    reference.as_ref(),
                );
            }
            match &payment.details {
                Some(PaymentDetails::CreditTransfers(transfers)) => {
                    for transfer in transfers {
                        serializer.structural(CAC, "PayeeFinancialAccount", |serializer| {
                            serializer.field_leaf(CBC, "ID", "account", transfer.account.as_ref());
                            if let Some(name) = &transfer.account_name {
                                serializer.field_leaf(CBC, "Name", "account_name", name.as_ref());
                            }
                            if let Some(provider) = &transfer.provider {
                                serializer.structural(
                                    CAC,
                                    "FinancialInstitutionBranch",
                                    |serializer| {
                                        serializer.field_leaf(
                                            CBC,
                                            "ID",
                                            "provider",
                                            provider.as_ref(),
                                        );
                                    },
                                );
                            }
                        });
                    }
                }
                Some(PaymentDetails::Card(card)) => {
                    serializer.structural(CAC, "CardAccount", |serializer| {
                        serializer.field_leaf(
                            CBC,
                            "PrimaryAccountNumberID",
                            "primary_account_number",
                            card.primary_account_number.as_ref(),
                        );
                        if let Some(holder) = &card.holder_name {
                            serializer.field_leaf(
                                CBC,
                                "HolderName",
                                "holder_name",
                                holder.as_ref(),
                            );
                        }
                    });
                }
                Some(PaymentDetails::DirectDebit(debit)) => {
                    serializer.structural(CAC, "PaymentMandate", |serializer| {
                        if let Some(reference) = &debit.mandate_reference {
                            serializer.field_leaf(
                                CBC,
                                "ID",
                                "mandate_reference",
                                reference.as_ref(),
                            );
                        }
                        if let Some(creditor) = &debit.creditor_identifier {
                            serializer.field_leaf(
                                CBC,
                                "PayerPartyID",
                                "creditor_identifier",
                                creditor.as_ref(),
                            );
                        }
                        if let Some(account) = &debit.debited_account {
                            serializer.structural(CAC, "PayerFinancialAccount", |serializer| {
                                serializer.field_leaf(
                                    CBC,
                                    "ID",
                                    "debited_account",
                                    account.as_ref(),
                                );
                            });
                        }
                    });
                }
                None => {}
            }
        });
    }

    // Serializes the document-level allowances and charges (`BG-20`/`BG-21`).
    fn adjustments(&mut self, adjustments: &[Adjustment]) {
        for (position, adjustment) in adjustments.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive adjustment index");
            let charge = matches!(adjustment.reason, AdjustmentReason::Charge { .. });
            let term = if charge { Term::BG(21) } else { Term::BG(20) };
            self.repeatable(
                CAC,
                "AllowanceCharge",
                "adjustments",
                term,
                instance,
                |serializer| {
                    serializer.derived(
                        CBC,
                        "ChargeIndicator",
                        &[],
                        if charge { "true" } else { "false" },
                    );
                    serializer.adjustment_reason(&adjustment.reason);
                    serializer.adjustment_amount(&adjustment.amount);
                    serializer.tax_category(&adjustment.vat);
                },
            );
        }
    }

    // Serializes the reason code and text of an adjustment, mapped to the adjustment.
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
            self.derived(CBC, "AllowanceChargeReasonCode", &[], &code);
        }
        if let Some(text) = text {
            self.derived(CBC, "AllowanceChargeReason", &[], text.as_ref());
        }
    }

    // Serializes the amount of an adjustment, absolute or relative, mapped to the adjustment.
    fn adjustment_amount(&mut self, amount: &AdjustmentAmount) {
        let currency = self.currency;
        if let AdjustmentAmount::Relative { rate, base } = amount {
            self.derived(
                CBC,
                "MultiplierFactorNumeric",
                &[],
                &plain(Decimal::from(*rate)),
            );
            self.derived(
                CBC,
                "Amount",
                &[("currencyID", currency)],
                &money(amount.value()),
            );
            self.derived(
                CBC,
                "BaseAmount",
                &[("currencyID", currency)],
                &money(*base),
            );
        } else {
            self.derived(
                CBC,
                "Amount",
                &[("currencyID", currency)],
                &money(amount.value()),
            );
        }
    }

    // Serializes the tax total (`BG-23`) and the accounting-currency tax total (`BT-111`).
    fn tax_total(&mut self, invoice: &Invoice) {
        let currency = self.currency;
        self.structural(CAC, "TaxTotal", |serializer| {
            serializer.derived(
                CBC,
                "TaxAmount",
                &[("currencyID", currency)],
                &money(invoice.vat_total()),
            );
            for group in invoice.vat_breakdown() {
                serializer.structural(CAC, "TaxSubtotal", |serializer| {
                    serializer.derived(
                        CBC,
                        "TaxableAmount",
                        &[("currencyID", currency)],
                        &money(group.taxable),
                    );
                    serializer.derived(
                        CBC,
                        "TaxAmount",
                        &[("currencyID", currency)],
                        &money(group.tax),
                    );
                    serializer.structural(CAC, "TaxCategory", |serializer| {
                        serializer.derived(CBC, "ID", &[], &group.treatment.category().to_string());
                        serializer.derived(CBC, "Percent", &[], &plain(group.treatment.rate()));
                        if let VatTreatment::Exempt { code, text } = &group.treatment {
                            if let Some(code) = code {
                                serializer.derived(
                                    CBC,
                                    "TaxExemptionReasonCode",
                                    &[],
                                    &code.to_string(),
                                );
                            }
                            if let Some(text) = text {
                                serializer.derived(CBC, "TaxExemptionReason", &[], text.as_ref());
                            }
                        }
                        serializer.structural(CAC, "TaxScheme", |serializer| {
                            serializer.derived(CBC, "ID", &[], "VAT");
                        });
                    });
                });
            }
        });
        if let Some(accounting) = &invoice.vat_accounting_total {
            let accounting_currency = accounting.currency.code();
            let value = money(accounting.value);
            self.structural(CAC, "TaxTotal", |serializer| {
                serializer.derived(
                    CBC,
                    "TaxAmount",
                    &[("currencyID", accounting_currency)],
                    &value,
                );
            });
        }
    }

    // Serializes the legal monetary total, all derived amounts mapped to the root.
    fn legal_monetary_total(&mut self, invoice: &Invoice) {
        let currency = self.currency;
        self.structural(CAC, "LegalMonetaryTotal", |serializer| {
            let attr: [(&str, &str); 1] = [("currencyID", currency)];
            serializer.derived(
                CBC,
                "LineExtensionAmount",
                &attr,
                &money(invoice.line_net_total()),
            );
            serializer.derived(
                CBC,
                "TaxExclusiveAmount",
                &attr,
                &money(invoice.net_total()),
            );
            serializer.derived(
                CBC,
                "TaxInclusiveAmount",
                &attr,
                &money(invoice.gross_total()),
            );
            if !invoice.allowances_total().is_zero() {
                serializer.derived(
                    CBC,
                    "AllowanceTotalAmount",
                    &attr,
                    &money(invoice.allowances_total()),
                );
            }
            if !invoice.charges_total().is_zero() {
                serializer.derived(
                    CBC,
                    "ChargeTotalAmount",
                    &attr,
                    &money(invoice.charges_total()),
                );
            }
            if let Some(paid) = invoice.paid {
                serializer.derived(CBC, "PrepaidAmount", &attr, &money(paid));
            }
            if let Some(rounding) = invoice.rounding {
                serializer.derived(CBC, "PayableRoundingAmount", &attr, &money(rounding));
            }
            serializer.derived(CBC, "PayableAmount", &attr, &money(invoice.due()));
        });
    }

    // Serializes the invoice lines (`BG-25`), each a repeatable-group instance.
    fn lines(&mut self, lines: &[InvoiceLine]) {
        for (position, line) in lines.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive line index");
            self.repeatable(
                CAC,
                "InvoiceLine",
                "lines",
                Term::BG(25),
                instance,
                |serializer| {
                    serializer.invoice_line(line);
                },
            );
        }
    }

    // Serializes one invoice line body.
    fn invoice_line(&mut self, line: &InvoiceLine) {
        let currency = self.currency;
        self.leaf(CBC, "ID", "id", Term::BT(126), line.id.as_ref());
        if let Some(note) = &line.note {
            self.leaf(CBC, "Note", "note", Term::BT(127), note.as_ref());
        }
        self.leaf_attr(
            CBC,
            "InvoicedQuantity",
            "quantity",
            Term::BT(129),
            &[("unitCode", line.quantity.unit.code())],
            &plain(line.quantity.value),
        );
        self.derived(
            CBC,
            "LineExtensionAmount",
            &[("currencyID", currency)],
            &money(line.net_amount()),
        );
        if let Some(reference) = &line.buyer_accounting_reference {
            self.leaf(
                CBC,
                "AccountingCost",
                "buyer_accounting_reference",
                Term::BT(133),
                reference.as_ref(),
            );
        }
        if let Some(period) = line.period {
            self.line_period(period);
        }
        if let Some(order) = &line.order_line_reference {
            self.structural(CAC, "OrderLineReference", |serializer| {
                serializer.leaf(
                    CBC,
                    "LineID",
                    "order_line_reference",
                    Term::BT(132),
                    order.as_ref(),
                );
            });
        }
        if let Some(object) = &line.object {
            self.structural(CAC, "DocumentReference", |serializer| {
                match &object.scheme {
                    Some(scheme) => serializer.leaf_attr(
                        CBC,
                        "ID",
                        "object",
                        Term::BT(128),
                        &[("schemeID", &scheme.to_string())],
                        object.id.as_ref(),
                    ),
                    None => serializer.leaf(CBC, "ID", "object", Term::BT(128), object.id.as_ref()),
                }
            });
        }
        self.line_adjustments(&line.adjustments);
        self.item(&line.item, &line.vat);
        self.line_price(line);
    }

    // Serializes the line invoice period (`BG-26`).
    fn line_period(&mut self, period: Period) {
        self.group(CAC, "InvoicePeriod", "period", Term::BG(26), |serializer| {
            if let Some(start) = period.start() {
                serializer.field_leaf(CBC, "StartDate", "period", &date(start));
            }
            if let Some(end) = period.end() {
                serializer.field_leaf(CBC, "EndDate", "period", &date(end));
            }
        });
    }

    // Serializes the line-level allowances and charges (`BG-27`/`BG-28`).
    fn line_adjustments(&mut self, adjustments: &[LineAdjustment]) {
        for (position, adjustment) in adjustments.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive adjustment index");
            let charge = matches!(adjustment.reason, AdjustmentReason::Charge { .. });
            let term = if charge { Term::BG(28) } else { Term::BG(27) };
            self.repeatable(
                CAC,
                "AllowanceCharge",
                "adjustments",
                term,
                instance,
                |serializer| {
                    serializer.derived(
                        CBC,
                        "ChargeIndicator",
                        &[],
                        if charge { "true" } else { "false" },
                    );
                    serializer.adjustment_reason(&adjustment.reason);
                    serializer.adjustment_amount(&adjustment.amount);
                },
            );
        }
    }

    // Serializes the item (`BG-31`), rebasing the classified tax category onto the line VAT.
    fn item(&mut self, item: &Item, vat: &VatTreatment) {
        self.trace.enter(CAC, "Item");
        self.trace.push_field("item");
        self.trace.record_context();
        self.write_start(CAC, "Item");

        if let Some(description) = &item.description {
            self.field_leaf(CBC, "Description", "description", description.as_ref());
        }
        self.field_leaf(CBC, "Name", "name", item.name.as_ref());
        if let Some(id) = &item.buyer_id {
            self.structural(CAC, "BuyersItemIdentification", |serializer| {
                serializer.field_leaf(CBC, "ID", "buyer_id", id.as_ref());
            });
        }
        if let Some(id) = &item.seller_id {
            self.structural(CAC, "SellersItemIdentification", |serializer| {
                serializer.field_leaf(CBC, "ID", "seller_id", id.as_ref());
            });
        }
        if let Some(standard) = &item.standard_id {
            self.structural(CAC, "StandardItemIdentification", |serializer| {
                serializer.field_leaf_attr(
                    CBC,
                    "ID",
                    "standard_id",
                    &[("schemeID", &standard.issuer.to_string())],
                    standard.id.as_ref(),
                );
            });
        }
        if let Some(country) = item.country_of_origin {
            self.structural(CAC, "OriginCountry", |serializer| {
                serializer.field_leaf(
                    CBC,
                    "IdentificationCode",
                    "country_of_origin",
                    country.alpha2(),
                );
            });
        }
        for classification in &item.classifications {
            self.structural(CAC, "CommodityClassification", |serializer| {
                let mut attributes = vec![("listID".to_owned(), classification.scheme.to_string())];
                if let Some(version) = &classification.version {
                    attributes.push(("listVersionID".to_owned(), version.as_ref().to_owned()));
                }
                let borrowed: Vec<(&str, &str)> = attributes
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str()))
                    .collect();
                serializer.field_leaf_attr(
                    CBC,
                    "ItemClassificationCode",
                    "classifications",
                    &borrowed,
                    classification.id.as_ref(),
                );
            });
        }

        // The classified tax category is a sibling of the item in the model.
        self.trace.pop_context();
        self.classified_tax_category(vat);

        for attribute in &item.attributes {
            self.structural(CAC, "AdditionalItemProperty", |serializer| {
                serializer.field_leaf(CBC, "Name", "attributes", attribute.name.as_ref());
                serializer.field_leaf(CBC, "Value", "attributes", attribute.value.as_ref());
            });
        }

        self.write_end(CAC, "Item");
        self.trace.leave();
    }

    // Serializes the line VAT as a UBL classified tax category, mapped to the VAT field.
    fn classified_tax_category(&mut self, vat: &VatTreatment) {
        self.group(
            CAC,
            "ClassifiedTaxCategory",
            "vat",
            Term::BG(30),
            |serializer| {
                serializer.field_leaf(CBC, "ID", "vat", &vat.category().to_string());
                serializer.field_leaf(CBC, "Percent", "vat", &plain(vat.rate()));
                serializer.structural(CAC, "TaxScheme", |serializer| {
                    serializer.field_leaf(CBC, "ID", "vat", "VAT");
                });
            },
        );
    }

    // Serializes the line price (`BG-29`), its derived net amount mapped to the price group.
    fn line_price(&mut self, line: &InvoiceLine) {
        let currency = self.currency;
        self.group(CAC, "Price", "price", Term::BG(29), |serializer| {
            serializer.derived(
                CBC,
                "PriceAmount",
                &[("currencyID", currency)],
                &money(line.price.net()),
            );
            if let Some(base) = line.price.base_quantity {
                serializer.field_leaf_attr(
                    CBC,
                    "BaseQuantity",
                    "price",
                    &[("unitCode", base.unit.code())],
                    &plain(base.value),
                );
            }
            if let Some(discount) = line.price.discount {
                serializer.structural(CAC, "AllowanceCharge", |serializer| {
                    serializer.field_leaf(CBC, "ChargeIndicator", "price", "false");
                    serializer.field_leaf_attr(
                        CBC,
                        "Amount",
                        "price",
                        &[("currencyID", currency)],
                        &money(discount),
                    );
                    serializer.field_leaf_attr(
                        CBC,
                        "BaseAmount",
                        "price",
                        &[("currencyID", currency)],
                        &money(line.price.gross),
                    );
                });
            }
        });
    }

    // The VAT point date (`BT-7`), when the point is a date.
    fn tax_point_date(&self, invoice: &Invoice) -> Option<VatPointDate> {
        match invoice.vat_point {
            Some(VatPoint::Date(date)) => Some(VatPointDate(date)),
            _ => None,
        }
    }

    // The VAT point event code (`BT-8`), when the point is an event.
    fn vat_point_event(&self, invoice: &Invoice) -> Option<String> {
        match invoice.vat_point {
            Some(VatPoint::Event(event)) => Some(u16::from(event).to_string()),
            _ => None,
        }
    }

    // ---- element writers -------------------------------------------------

    // Writes a term-bearing leaf, skipped when the profile forbids the term.
    fn leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        value: &str,
    ) {
        if self.forbids(term) {
            return;
        }
        self.write_field_leaf(namespace, name, field, &[], value);
    }

    // Writes a term-bearing leaf with attributes, skipped when the profile forbids the term.
    fn leaf_attr(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        if self.forbids(term) {
            return;
        }
        self.write_field_leaf(namespace, name, field, attributes, value);
    }

    // Writes a leaf mapped to a model field, never filtered (its term is never forbidden).
    fn field_leaf(&mut self, namespace: Namespace, name: &str, field: &'static str, value: &str) {
        self.write_field_leaf(namespace, name, field, &[], value);
    }

    // Writes a leaf with attributes mapped to a model field, never filtered.
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

    // Writes a repeatable single-value element mapped to a group instance.
    fn repeatable_leaf(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        instance: NonZeroUsize,
        value: &str,
    ) {
        if self.forbids(term) {
            return;
        }
        self.trace.enter(namespace, name);
        self.trace.push_instance(field, instance);
        self.trace.record_context();
        self.write_element(namespace, name, &[], value);
        self.trace.pop_context();
        self.trace.leave();
    }

    // Writes a regulatory leaf recorded at the root context (`BT-23`/`BT-24`).
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

    // Writes a derived leaf with no model field, mapped to the enclosing context.
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

    // Writes a term-bearing group around a nested body, skipped when the term is forbidden.
    fn group(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        body: impl FnOnce(&mut Self),
    ) {
        if self.forbids(term) {
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

    // Writes one instance of a term-bearing repeatable group, carrying its index.
    fn repeatable(
        &mut self,
        namespace: Namespace,
        name: &str,
        field: &'static str,
        term: Term,
        instance: NonZeroUsize,
        body: impl FnOnce(&mut Self),
    ) {
        if self.forbids(term) {
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

    // Writes a term-less structural wrapper mapped to the root context.
    fn structural(&mut self, namespace: Namespace, name: &str, body: impl FnOnce(&mut Self)) {
        self.trace.enter(namespace, name);
        self.trace.record_root();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    // Writes a wrapper mapped to the enclosing context, without a new segment.
    fn nested(&mut self, namespace: Namespace, name: &str, body: impl FnOnce(&mut Self)) {
        self.trace.enter(namespace, name);
        self.trace.record_context();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    // Serializes an allowance or charge VAT category, mapped to the enclosing adjustment.
    fn tax_category(&mut self, vat: &VatTreatment) {
        self.nested(CAC, "TaxCategory", |serializer| {
            serializer.derived(CBC, "ID", &[], &vat.category().to_string());
            serializer.derived(CBC, "Percent", &[], &plain(vat.rate()));
            serializer.nested(CAC, "TaxScheme", |serializer| {
                serializer.derived(CBC, "ID", &[], "VAT");
            });
        });
    }

    // ---- raw XML ---------------------------------------------------------

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
    let prefix = prefix(namespace);
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}:{name}")
    }
}

// The context field a single-identifier reference term maps to.
fn field_of(term: Term) -> &'static str {
    match term {
        Term::BT(11) => "project_reference",
        Term::BT(12) => "contract_reference",
        Term::BT(15) => "receiving_advice_reference",
        Term::BT(16) => "despatch_advice_reference",
        Term::BT(17) => "tender_or_lot_reference",
        Term::BT(27) | Term::BT(44) => "name",
        Term::BT(28) | Term::BT(45) => "trading_name",
        Term::BT(29) | Term::BT(46) | Term::BT(60) => "identifiers",
        Term::BT(30) | Term::BT(47) | Term::BT(61) => "legal_entity",
        Term::BT(31) | Term::BT(48) | Term::BT(63) => "vat",
        Term::BT(32) => "tax_registration",
        Term::BT(34) | Term::BT(49) => "electronic_address",
        Term::BT(41) | Term::BT(56) => "name",
        Term::BT(42) | Term::BT(57) => "telephone",
        Term::BT(43) | Term::BT(58) => "email",
        Term::BT(59) | Term::BT(62) => "name",
        Term::BT(70) => "name",
        Term::BG(6) | Term::BG(9) => "contact",
        _ => "",
    }
}

// A newtype marking a resolved VAT point date, to keep the option handling clear.
struct VatPointDate(Date);
