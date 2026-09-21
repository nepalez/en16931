use base64::Engine as _;

use crate::format::trace::Trace;
use crate::format::ubl;
use crate::prelude::*;
use crate::{
    Abbreviations, Adjustment, AdjustmentAmount, AdjustmentReason, BinaryObject, Buyer, Contact,
    Currency, Delivery, Dictionary, Document, DocumentBuilder, ElectronicAddress, Format, Invoice,
    InvoiceLine, Item, LegalEntity, LineAdjustment, Namespace, Note, ObjectReference,
    OperationalEntity, Payee, PaymentDetails, PaymentInstructions, Period, PostalAddress,
    PrecedingInvoice, Price, Seller, Serializable, SupportingDocument, TaxRepresentative, Term,
    Ubl, VatPoint, VatTreatment,
};

impl Serializable<Ubl> for Invoice {
    fn serialize(document: &mut Document<Self, Ubl>) {
        let mut serializer = Serializer::new(&document.builder);
        serializer.document(&document.builder);
        let (xml, dictionary, abbreviations) = serializer.finish();
        document.xml = xml;
        document.dictionary = dictionary;
        document.abbreviations = abbreviations;
    }
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

// Encodes a note as `#subject#text`, or bare text when the subject is dropped,
// or nothing when the note carries no text.
fn note_text(note: &Note, drop_subject: bool) -> Option<String> {
    let text = note.text.as_ref()?;
    Some(match &note.subject_code {
        Some(code) if !drop_subject => format!("#{}#{}", code.as_ref(), text.as_ref()),
        _ => text.as_ref().to_owned(),
    })
}

/// The stateful UBL writer: an XML sink plus the trace that builds the dictionary in lockstep.
struct Serializer {
    inner: Writer<Vec<u8>>,
    trace: Trace<ubl::Namespace>,
    abbreviations: Abbreviations<ubl::Namespace>,
    forbidden: &'static [Term],
    currency: Option<&'static str>,
}

impl Serializer {
    fn new(builder: &DocumentBuilder<Invoice>) -> Self {
        Self {
            inner: Writer::new(Vec::new()),
            trace: Trace::new(),
            abbreviations: <Ubl as Format>::Namespace::default_abbreviations(),
            forbidden: builder.profile.forbidden_terms(),
            currency: builder.invoice.currency.as_ref().map(Currency::code),
        }
    }

    // The `currencyID` attribute of an amount, absent without the invoice currency.
    fn currency_attribute(&self) -> Vec<(&'static str, &'static str)> {
        self.currency
            .map(|currency| vec![("currencyID", currency)])
            .unwrap_or_default()
    }

    fn finish(
        self,
    ) -> (
        String,
        Dictionary<ubl::Namespace>,
        Abbreviations<ubl::Namespace>,
    ) {
        let xml = String::from_utf8(self.inner.into_inner()).expect("quick-xml emits valid UTF-8");
        (xml, self.trace.into_dictionary(), self.abbreviations)
    }

    // Whether the profile forbids the term of a node about to be written.
    fn forbids(&self, term: Term) -> bool {
        self.forbidden.contains(&term)
    }

    // Serializes the whole document under the UBL root element.
    fn document(&mut self, builder: &DocumentBuilder<Invoice>) {
        let invoice = &builder.invoice;
        let declarations: Vec<(String, &'static str)> = ubl::Namespace::VARIANTS
            .iter()
            .map(|namespace| {
                let prefix = namespace.prefix();
                let key = if prefix.is_empty() {
                    "xmlns".to_owned()
                } else {
                    format!("xmlns:{prefix}")
                };
                (key, namespace.uri())
            })
            .collect();
        let root = BytesStart::new(qname(Ubl::root_namespace(), Ubl::ROOT_ELEMENT))
            .with_attributes(declarations.iter().map(|(key, uri)| (key.as_str(), *uri)));
        self.write(Event::Start(root));
        for namespace in ubl::Namespace::VARIANTS {
            self.abbreviations
                .declare(namespace.prefix(), *namespace)
                .expect("the writer binds each abbreviation to one namespace");
        }
        self.trace.enter(Ubl::root_namespace(), Ubl::ROOT_ELEMENT);
        self.trace.record_root();

        // Regulatory-flow fields: the specification identifier (BT-24) and business process (BT-23).
        self.rooted(
            ubl::Namespace::Cbc,
            "CustomizationID",
            &[],
            &builder.profile.to_string(),
        );
        if let Some(process) = &builder.business_process {
            self.rooted(ubl::Namespace::Cbc, "ProfileID", &[], process.as_ref());
        }

        if let Some(number) = &invoice.number {
            self.leaf(
                ubl::Namespace::Cbc,
                "ID",
                "number",
                Term::BT(1),
                number.as_ref(),
            );
        }
        if let Some(issued) = invoice.issue_date {
            self.leaf(
                ubl::Namespace::Cbc,
                "IssueDate",
                "issue_date",
                Term::BT(2),
                &date(issued),
            );
        }
        if let Some(due) = invoice.payment_due_date {
            self.leaf(
                ubl::Namespace::Cbc,
                "DueDate",
                "payment_due_date",
                Term::BT(9),
                &date(due),
            );
        }
        self.leaf(
            ubl::Namespace::Cbc,
            "InvoiceTypeCode",
            "type_code",
            Term::BT(3),
            &invoice.type_code.to_string(),
        );
        self.notes(&invoice.notes);
        if let Some(VatPointDate(point)) = self.tax_point_date(invoice) {
            self.leaf(
                ubl::Namespace::Cbc,
                "TaxPointDate",
                "vat_point",
                Term::BT(7),
                &date(point),
            );
        }
        if let Some(currency) = &invoice.currency {
            self.leaf(
                ubl::Namespace::Cbc,
                "DocumentCurrencyCode",
                "currency",
                Term::BT(5),
                currency.code(),
            );
        }
        if let Some(accounting) = &invoice.vat_accounting_total {
            self.leaf(
                ubl::Namespace::Cbc,
                "TaxCurrencyCode",
                "vat_accounting_total",
                Term::BT(6),
                accounting.currency.code(),
            );
        }
        if let Some(reference) = &invoice.buyer_accounting_reference {
            self.leaf(
                ubl::Namespace::Cbc,
                "AccountingCost",
                "buyer_accounting_reference",
                Term::BT(19),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.buyer_reference {
            self.leaf(
                ubl::Namespace::Cbc,
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
                ubl::Namespace::Cac,
                "DespatchDocumentReference",
                Term::BT(16),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.receiving_advice_reference {
            self.reference_group(
                ubl::Namespace::Cac,
                "ReceiptDocumentReference",
                Term::BT(15),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.tender_or_lot_reference {
            self.reference_group(
                ubl::Namespace::Cac,
                "OriginatorDocumentReference",
                Term::BT(17),
                reference.as_ref(),
            );
        }
        if let Some(reference) = &invoice.contract_reference {
            self.reference_group(
                ubl::Namespace::Cac,
                "ContractDocumentReference",
                Term::BT(12),
                reference.as_ref(),
            );
        }
        self.additional_documents(invoice);
        if let Some(reference) = &invoice.project_reference {
            self.reference_group(
                ubl::Namespace::Cac,
                "ProjectReference",
                Term::BT(11),
                reference.as_ref(),
            );
        }

        if let Some(seller) = &invoice.seller {
            self.supplier_party(seller);
        }
        if let Some(buyer) = &invoice.buyer {
            self.customer_party(buyer);
        }
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
                ubl::Namespace::Cac,
                "PaymentTerms",
                "payment_terms",
                Term::BT(20),
                |serializer| {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "Note",
                        "payment_terms",
                        terms.as_ref(),
                    );
                },
            );
        }
        self.adjustments(&invoice.adjustments);
        self.tax_total(invoice);
        self.legal_monetary_total(invoice);
        self.lines(&invoice.lines);

        self.trace.leave();
        self.write(Event::End(BytesEnd::new(qname(
            Ubl::root_namespace(),
            Ubl::ROOT_ELEMENT,
        ))));
    }

    // Serializes the notes (`BG-1`), each a repeatable single-value element.
    fn notes(&mut self, notes: &[Note]) {
        let drop_subject = self.forbids(Term::BT(21));
        for (position, note) in notes.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive note index");
            let Some(text) = note_text(note, drop_subject) else {
                continue;
            };
            self.repeatable_leaf(
                ubl::Namespace::Cbc,
                "Note",
                "notes",
                Term::BG(1),
                instance,
                &text,
            );
        }
    }

    // Serializes the invoice period (`BG-14`) and the VAT point date code (`BT-8`).
    fn invoice_period(&mut self, invoice: &Invoice) {
        let event = self.vat_point_event(invoice);
        if invoice.invoicing_period.is_none() && event.is_none() {
            return;
        }
        self.group(
            ubl::Namespace::Cac,
            "InvoicePeriod",
            "invoicing_period",
            Term::BG(14),
            |serializer| {
                if let Some(period) = invoice.invoicing_period {
                    if let Some(start) = period.start() {
                        serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "StartDate",
                            "invoicing_period",
                            &date(start),
                        );
                    }
                    if let Some(end) = period.end() {
                        serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "EndDate",
                            "invoicing_period",
                            &date(end),
                        );
                    }
                }
                if let Some(code) = event {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "DescriptionCode",
                        "vat_point",
                        &code,
                    );
                }
            },
        );
    }

    // Serializes the order and sales order references (`BT-13`/`BT-14`).
    fn order_reference(&mut self, invoice: &Invoice) {
        if invoice.purchase_order_reference.is_none() && invoice.sales_order_reference.is_none() {
            return;
        }
        self.structural(ubl::Namespace::Cac, "OrderReference", |serializer| {
            if let Some(order) = &invoice.purchase_order_reference {
                serializer.leaf(
                    ubl::Namespace::Cbc,
                    "ID",
                    "purchase_order_reference",
                    Term::BT(13),
                    order.as_ref(),
                );
            }
            if let Some(sales) = &invoice.sales_order_reference {
                serializer.leaf(
                    ubl::Namespace::Cbc,
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
                ubl::Namespace::Cac,
                "BillingReference",
                "preceding_invoices",
                Term::BG(3),
                instance,
                |serializer| {
                    serializer.structural(
                        ubl::Namespace::Cac,
                        "InvoiceDocumentReference",
                        |serializer| {
                            if let Some(number) = &invoice.number {
                                serializer.field_leaf(
                                    ubl::Namespace::Cbc,
                                    "ID",
                                    "number",
                                    number.as_ref(),
                                );
                            }
                            if let Some(issued) = invoice.issue_date {
                                serializer.field_leaf(
                                    ubl::Namespace::Cbc,
                                    "IssueDate",
                                    "issue_date",
                                    &date(issued),
                                );
                            }
                        },
                    );
                },
            );
        }
    }

    // Serializes a document reference carrying a single identifier.
    fn reference_group(&mut self, namespace: ubl::Namespace, element: &str, term: Term, id: &str) {
        self.group(namespace, element, field_of(term), term, |serializer| {
            serializer.field_leaf(ubl::Namespace::Cbc, "ID", field_of(term), id);
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
            ubl::Namespace::Cac,
            "AdditionalDocumentReference",
            "object",
            Term::BT(18),
            |serializer| {
                if let Some(id) = &object.id {
                    match &object.scheme {
                        Some(scheme) => serializer.derived(
                            ubl::Namespace::Cbc,
                            "ID",
                            &[("schemeID", &scheme.to_string())],
                            id.as_ref(),
                        ),
                        None => serializer.derived(ubl::Namespace::Cbc, "ID", &[], id.as_ref()),
                    }
                }
                serializer.derived(ubl::Namespace::Cbc, "DocumentTypeCode", &[], "130");
            },
        );
    }

    // Serializes a supporting document (`BG-24`) as an additional document reference.
    fn additional_supporting(&mut self, document: &SupportingDocument, instance: NonZeroUsize) {
        self.repeatable(
            ubl::Namespace::Cac,
            "AdditionalDocumentReference",
            "supporting_documents",
            Term::BG(24),
            instance,
            |serializer| {
                if let Some(reference) = &document.reference {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "ID",
                        "reference",
                        reference.as_ref(),
                    );
                }
                if let Some(description) = &document.description {
                    serializer.leaf(
                        ubl::Namespace::Cbc,
                        "DocumentDescription",
                        "description",
                        Term::BT(123),
                        description.as_ref(),
                    );
                }
                if let Some(location) = &document.external_location {
                    if !serializer.forbids(Term::BT(124)) {
                        serializer.structural(ubl::Namespace::Cac, "Attachment", |serializer| {
                            serializer.structural(
                                ubl::Namespace::Cac,
                                "ExternalReference",
                                |serializer| {
                                    serializer.field_leaf(
                                        ubl::Namespace::Cbc,
                                        "URI",
                                        "external_location",
                                        location.as_str(),
                                    );
                                },
                            );
                        });
                    }
                }
                if let Some(binary) = &document.attachment {
                    serializer.structural(ubl::Namespace::Cac, "Attachment", |serializer| {
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
            ubl::Namespace::Cbc,
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
        self.structural(
            ubl::Namespace::Cac,
            "AccountingSupplierParty",
            |serializer| {
                serializer.group(
                    ubl::Namespace::Cac,
                    "Party",
                    "seller",
                    Term::BG(4),
                    |serializer| {
                        if let Some(address) = &seller.electronic_address {
                            serializer.endpoint(address, Term::BT(34));
                        }
                        serializer.party_identifiers(&seller.identifiers, Term::BT(29));
                        if let Some(trading) = &seller.trading_name {
                            serializer.party_name(trading.as_ref(), Term::BT(28));
                        }
                        if let Some(address) = &seller.address {
                            serializer.postal_address("PostalAddress", Term::BG(5), address);
                        }
                        if let Some(vat) = &seller.vat {
                            serializer.party_tax_scheme(&vat.to_string(), "VAT", Term::BT(31));
                        }
                        if let Some(registration) = &seller.tax_registration {
                            serializer.party_tax_scheme(registration.as_ref(), "FC", Term::BT(32));
                        }
                        serializer.party_legal_entity(
                            seller.name.as_ref().map(|name| name.as_ref()),
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
                    },
                );
            },
        );
    }

    // Serializes the buyer (`BG-7`) under the customer party wrapper.
    fn customer_party(&mut self, buyer: &Buyer) {
        self.structural(
            ubl::Namespace::Cac,
            "AccountingCustomerParty",
            |serializer| {
                serializer.group(
                    ubl::Namespace::Cac,
                    "Party",
                    "buyer",
                    Term::BG(7),
                    |serializer| {
                        if let Some(address) = &buyer.electronic_address {
                            serializer.endpoint(address, Term::BT(49));
                        }
                        serializer.party_identifiers(&buyer.identifiers, Term::BT(46));
                        if let Some(trading) = &buyer.trading_name {
                            serializer.party_name(trading.as_ref(), Term::BT(45));
                        }
                        if let Some(address) = &buyer.address {
                            serializer.postal_address("PostalAddress", Term::BG(8), address);
                        }
                        if let Some(vat) = &buyer.vat {
                            serializer.party_tax_scheme(&vat.to_string(), "VAT", Term::BT(48));
                        }
                        serializer.party_legal_entity(
                            buyer.name.as_ref().map(|name| name.as_ref()),
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
                    },
                );
            },
        );
    }

    // Serializes the payee (`BG-10`).
    fn payee_party(&mut self, payee: &Payee) {
        self.group(
            ubl::Namespace::Cac,
            "PayeeParty",
            "payee",
            Term::BG(10),
            |serializer| {
                serializer.party_identifiers(&payee.identifiers, Term::BT(60));
                if let Some(name) = &payee.name {
                    serializer.party_name(name.as_ref(), Term::BT(59));
                }
                if let Some(entity) = &payee.legal_entity {
                    serializer.structural(ubl::Namespace::Cac, "PartyLegalEntity", |serializer| {
                        serializer.legal_entity_id(entity, Term::BT(61));
                    });
                }
            },
        );
    }

    // Serializes the seller tax representative (`BG-11`).
    fn tax_representative_party(&mut self, representative: &TaxRepresentative) {
        self.group(
            ubl::Namespace::Cac,
            "TaxRepresentativeParty",
            "tax_representative",
            Term::BG(11),
            |serializer| {
                if let Some(name) = &representative.name {
                    serializer.party_name(name.as_ref(), Term::BT(62));
                }
                if let Some(address) = &representative.address {
                    serializer.postal_address("PostalAddress", Term::BG(12), address);
                }
                if let Some(vat) = &representative.vat {
                    serializer.party_tax_scheme(&vat.to_string(), "VAT", Term::BT(63));
                }
            },
        );
    }

    // Serializes a party endpoint electronic address (`BT-34`/`BT-49`).
    fn endpoint(&mut self, address: &ElectronicAddress, term: Term) {
        let Some(id) = &address.id else {
            return;
        };
        match &address.scheme {
            Some(scheme) => self.leaf_attr(
                ubl::Namespace::Cbc,
                "EndpointID",
                field_of(term),
                term,
                &[("schemeID", &scheme.to_string())],
                id.as_ref(),
            ),
            None => self.leaf(
                ubl::Namespace::Cbc,
                "EndpointID",
                field_of(term),
                term,
                id.as_ref(),
            ),
        }
    }

    // Serializes the party identifiers (`BT-29`/`BT-46`/`BT-60`).
    fn party_identifiers(&mut self, identifiers: &[OperationalEntity], term: Term) {
        for identifier in identifiers {
            self.group(
                ubl::Namespace::Cac,
                "PartyIdentification",
                field_of(term),
                term,
                |serializer| {
                    let Some(id) = &identifier.id else {
                        return;
                    };
                    match &identifier.issuer {
                        Some(issuer) => serializer.field_leaf_attr(
                            ubl::Namespace::Cbc,
                            "ID",
                            field_of(term),
                            &[("schemeID", &issuer.to_string())],
                            id.as_ref(),
                        ),
                        None => serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "ID",
                            field_of(term),
                            id.as_ref(),
                        ),
                    }
                },
            );
        }
    }

    // Serializes a party trading name (`BT-28`/`BT-45`).
    fn party_name(&mut self, name: &str, term: Term) {
        self.group(
            ubl::Namespace::Cac,
            "PartyName",
            field_of(term),
            term,
            |serializer| {
                serializer.field_leaf(ubl::Namespace::Cbc, "Name", field_of(term), name);
            },
        );
    }

    // Serializes a party tax scheme carrying a company id under a scheme code.
    fn party_tax_scheme(&mut self, company: &str, scheme: &str, term: Term) {
        self.group(
            ubl::Namespace::Cac,
            "PartyTaxScheme",
            field_of(term),
            term,
            |serializer| {
                serializer.field_leaf(ubl::Namespace::Cbc, "CompanyID", field_of(term), company);
                serializer.structural(ubl::Namespace::Cac, "TaxScheme", |serializer| {
                    serializer.field_leaf(ubl::Namespace::Cbc, "ID", field_of(term), scheme);
                });
            },
        );
    }

    // Serializes the party legal entity: the registration name, the legal id, and the legal form.
    fn party_legal_entity(
        &mut self,
        name: Option<&str>,
        name_term: Term,
        entity: Option<&LegalEntity>,
        entity_term: Term,
        legal_form: Option<&str>,
    ) {
        self.structural(ubl::Namespace::Cac, "PartyLegalEntity", |serializer| {
            if let Some(name) = name {
                serializer.leaf(
                    ubl::Namespace::Cbc,
                    "RegistrationName",
                    field_of(name_term),
                    name_term,
                    name,
                );
            }
            if let Some(entity) = entity {
                serializer.legal_entity_id(entity, entity_term);
            }
            if let Some(form) = legal_form {
                serializer.leaf(
                    ubl::Namespace::Cbc,
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
        let Some(id) = &entity.id else {
            return;
        };
        match &entity.issuer {
            Some(issuer) => self.leaf_attr(
                ubl::Namespace::Cbc,
                "CompanyID",
                field_of(term),
                term,
                &[("schemeID", &issuer.to_string())],
                id.as_ref(),
            ),
            None => self.leaf(
                ubl::Namespace::Cbc,
                "CompanyID",
                field_of(term),
                term,
                id.as_ref(),
            ),
        }
    }

    // Serializes a contact group (`BG-6`/`BG-9`).
    fn contact(&mut self, contact: &Contact, group: Term, name: Term, phone: Term, mail: Term) {
        self.group(
            ubl::Namespace::Cac,
            "Contact",
            field_of(group),
            group,
            |serializer| {
                if let Some(value) = &contact.name {
                    serializer.leaf(
                        ubl::Namespace::Cbc,
                        "Name",
                        field_of(name),
                        name,
                        value.as_ref(),
                    );
                }
                if let Some(value) = &contact.telephone {
                    serializer.leaf(
                        ubl::Namespace::Cbc,
                        "Telephone",
                        field_of(phone),
                        phone,
                        value.as_ref(),
                    );
                }
                if let Some(value) = &contact.email {
                    serializer.leaf(
                        ubl::Namespace::Cbc,
                        "ElectronicMail",
                        field_of(mail),
                        mail,
                        value.as_str(),
                    );
                }
            },
        );
    }

    // Serializes a postal address (`BG-5`/`BG-8`/`BG-12`/`BG-15`).
    fn postal_address(&mut self, element: &str, group: Term, address: &PostalAddress) {
        self.group(
            ubl::Namespace::Cac,
            element,
            "address",
            group,
            |serializer| {
                if let Some(line) = &address.line1 {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "StreetName",
                        "line1",
                        line.as_ref(),
                    );
                }
                if let Some(line) = &address.line2 {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "AdditionalStreetName",
                        "line2",
                        line.as_ref(),
                    );
                }
                if let Some(city) = &address.city {
                    serializer.field_leaf(ubl::Namespace::Cbc, "CityName", "city", city.as_ref());
                }
                if let Some(zip) = &address.postal_code {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "PostalZone",
                        "postal_code",
                        zip.as_ref(),
                    );
                }
                if let Some(subdivision) = &address.country_subdivision {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "CountrySubentity",
                        "country_subdivision",
                        subdivision.as_ref(),
                    );
                }
                if let Some(line) = &address.line3 {
                    serializer.structural(ubl::Namespace::Cac, "AddressLine", |serializer| {
                        serializer.field_leaf(ubl::Namespace::Cbc, "Line", "line3", line.as_ref());
                    });
                }
                if let Some(country) = &address.country {
                    serializer.structural(ubl::Namespace::Cac, "Country", |serializer| {
                        serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "IdentificationCode",
                            "country",
                            country.alpha2(),
                        );
                    });
                }
            },
        );
    }

    // Serializes the delivery information (`BG-13`).
    fn delivery(&mut self, delivery: &Delivery) {
        self.group(
            ubl::Namespace::Cac,
            "Delivery",
            "delivery",
            Term::BG(13),
            |serializer| {
                if let Some(date_value) = delivery.date {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "ActualDeliveryDate",
                        "date",
                        &date(date_value),
                    );
                }
                if delivery.location.is_some() || delivery.address.is_some() {
                    serializer.structural(ubl::Namespace::Cac, "DeliveryLocation", |serializer| {
                        if let Some(location) = &delivery.location {
                            if let Some(id) = &location.id {
                                match &location.issuer {
                                    Some(issuer) => serializer.field_leaf_attr(
                                        ubl::Namespace::Cbc,
                                        "ID",
                                        "location",
                                        &[("schemeID", &issuer.to_string())],
                                        id.as_ref(),
                                    ),
                                    None => serializer.field_leaf(
                                        ubl::Namespace::Cbc,
                                        "ID",
                                        "location",
                                        id.as_ref(),
                                    ),
                                }
                            }
                        }
                        if let Some(address) = &delivery.address {
                            serializer.postal_address("Address", Term::BG(15), address);
                        }
                    });
                }
                if let Some(name) = &delivery.name {
                    serializer.structural(ubl::Namespace::Cac, "DeliveryParty", |serializer| {
                        serializer.party_name(name.as_ref(), Term::BT(70));
                    });
                }
            },
        );
    }

    // Serializes the payment instructions (`BG-16`).
    fn payment_means(&mut self, payment: &PaymentInstructions) {
        self.group(
            ubl::Namespace::Cac,
            "PaymentMeans",
            "payment",
            Term::BG(16),
            |serializer| {
                let attributes: Vec<(String, String)> = match &payment.means_text {
                    Some(text) => vec![("name".to_owned(), text.as_ref().to_owned())],
                    None => Vec::new(),
                };
                let borrowed: Vec<(&str, &str)> = attributes
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str()))
                    .collect();
                if let Some(means) = &payment.means {
                    serializer.field_leaf_attr(
                        ubl::Namespace::Cbc,
                        "PaymentMeansCode",
                        "means",
                        &borrowed,
                        &means.to_string(),
                    );
                }
                if let Some(reference) = &payment.remittance_information {
                    serializer.field_leaf(
                        ubl::Namespace::Cbc,
                        "PaymentID",
                        "remittance_information",
                        reference.as_ref(),
                    );
                }
                match &payment.details {
                    Some(PaymentDetails::CreditTransfers(transfers)) => {
                        for transfer in transfers {
                            serializer.structural(
                                ubl::Namespace::Cac,
                                "PayeeFinancialAccount",
                                |serializer| {
                                    if let Some(account) = &transfer.account {
                                        serializer.field_leaf(
                                            ubl::Namespace::Cbc,
                                            "ID",
                                            "account",
                                            account.as_ref(),
                                        );
                                    }
                                    if let Some(name) = &transfer.account_name {
                                        serializer.field_leaf(
                                            ubl::Namespace::Cbc,
                                            "Name",
                                            "account_name",
                                            name.as_ref(),
                                        );
                                    }
                                    if let Some(provider) = &transfer.provider {
                                        serializer.structural(
                                            ubl::Namespace::Cac,
                                            "FinancialInstitutionBranch",
                                            |serializer| {
                                                serializer.field_leaf(
                                                    ubl::Namespace::Cbc,
                                                    "ID",
                                                    "provider",
                                                    provider.as_ref(),
                                                );
                                            },
                                        );
                                    }
                                },
                            );
                        }
                    }
                    Some(PaymentDetails::Card(card)) => {
                        serializer.structural(ubl::Namespace::Cac, "CardAccount", |serializer| {
                            if let Some(number) = &card.primary_account_number {
                                serializer.field_leaf(
                                    ubl::Namespace::Cbc,
                                    "PrimaryAccountNumberID",
                                    "primary_account_number",
                                    number.as_ref(),
                                );
                            }
                            if let Some(holder) = &card.holder_name {
                                serializer.field_leaf(
                                    ubl::Namespace::Cbc,
                                    "HolderName",
                                    "holder_name",
                                    holder.as_ref(),
                                );
                            }
                        });
                    }
                    Some(PaymentDetails::DirectDebit(debit)) => {
                        serializer.structural(
                            ubl::Namespace::Cac,
                            "PaymentMandate",
                            |serializer| {
                                if let Some(reference) = &debit.mandate_reference {
                                    serializer.field_leaf(
                                        ubl::Namespace::Cbc,
                                        "ID",
                                        "mandate_reference",
                                        reference.as_ref(),
                                    );
                                }
                                if let Some(creditor) = &debit.creditor_identifier {
                                    serializer.field_leaf(
                                        ubl::Namespace::Cbc,
                                        "PayerPartyID",
                                        "creditor_identifier",
                                        creditor.as_ref(),
                                    );
                                }
                                if let Some(account) = &debit.debited_account {
                                    serializer.structural(
                                        ubl::Namespace::Cac,
                                        "PayerFinancialAccount",
                                        |serializer| {
                                            serializer.field_leaf(
                                                ubl::Namespace::Cbc,
                                                "ID",
                                                "debited_account",
                                                account.as_ref(),
                                            );
                                        },
                                    );
                                }
                            },
                        );
                    }
                    None => {}
                }
            },
        );
    }

    // Serializes the document-level allowances and charges (`BG-20`/`BG-21`).
    fn adjustments(&mut self, adjustments: &[Adjustment]) {
        for (position, adjustment) in adjustments.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive adjustment index");
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
                ubl::Namespace::Cac,
                "AllowanceCharge",
                "adjustments",
                term,
                instance,
                |serializer| {
                    serializer.charge_indicator(charge);
                    if let Some(reason) = &adjustment.reason {
                        serializer.adjustment_reason(reason);
                    }
                    if let Some(amount) = &adjustment.amount {
                        serializer.adjustment_amount(amount);
                    }
                    if let Some(vat) = &adjustment.vat {
                        serializer.tax_category(vat);
                    }
                },
            );
        }
    }

    // Serializes the charge indicator of an adjustment, absent without the direction.
    fn charge_indicator(&mut self, charge: Option<bool>) {
        if let Some(charge) = charge {
            self.derived(
                ubl::Namespace::Cbc,
                "ChargeIndicator",
                &[],
                if charge { "true" } else { "false" },
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
            self.derived(ubl::Namespace::Cbc, "AllowanceChargeReasonCode", &[], &code);
        }
        if let Some(text) = text {
            self.derived(
                ubl::Namespace::Cbc,
                "AllowanceChargeReason",
                &[],
                text.as_ref(),
            );
        }
    }

    // Serializes the amount of an adjustment, absolute or relative, mapped to the amount field.
    fn adjustment_amount(&mut self, amount: &AdjustmentAmount) {
        let currency = self.currency_attribute();
        match amount {
            AdjustmentAmount::Relative { amount, rate, base } => {
                self.field_leaf(
                    ubl::Namespace::Cbc,
                    "MultiplierFactorNumeric",
                    "amount",
                    &plain(Decimal::from(*rate)),
                );
                self.field_leaf_attr(
                    ubl::Namespace::Cbc,
                    "Amount",
                    "amount",
                    &currency,
                    &money(*amount),
                );
                self.field_leaf_attr(
                    ubl::Namespace::Cbc,
                    "BaseAmount",
                    "amount",
                    &currency,
                    &money(*base),
                );
            }
            AdjustmentAmount::Absolute(amount) => {
                self.field_leaf_attr(
                    ubl::Namespace::Cbc,
                    "Amount",
                    "amount",
                    &currency,
                    &money(*amount),
                );
            }
        }
    }

    // Serializes the tax total (`BT-110`, `BG-23`) and the accounting-currency tax total
    // (`BT-111`). The first total is absent without the VAT total and the breakdown.
    fn tax_total(&mut self, invoice: &Invoice) {
        let currency = self.currency_attribute();
        if invoice.vat_total.is_some() || !invoice.vat_breakdown.is_empty() {
            self.structural(ubl::Namespace::Cac, "TaxTotal", |serializer| {
                if let Some(total) = invoice.vat_total {
                    serializer.leaf_attr(
                        ubl::Namespace::Cbc,
                        "TaxAmount",
                        "vat_total",
                        Term::BT(110),
                        &currency,
                        &money(total),
                    );
                }
                for (position, group) in invoice.vat_breakdown.iter().enumerate() {
                    let instance =
                        NonZeroUsize::new(position + 1).expect("a positive breakdown index");
                    serializer.repeatable(
                        ubl::Namespace::Cac,
                        "TaxSubtotal",
                        "vat_breakdown",
                        Term::BG(23),
                        instance,
                        |serializer| {
                            if let Some(taxable) = group.taxable {
                                serializer.leaf_attr(
                                    ubl::Namespace::Cbc,
                                    "TaxableAmount",
                                    "taxable",
                                    Term::BT(116),
                                    &currency,
                                    &money(taxable),
                                );
                            }
                            if let Some(tax) = group.tax {
                                serializer.leaf_attr(
                                    ubl::Namespace::Cbc,
                                    "TaxAmount",
                                    "tax",
                                    Term::BT(117),
                                    &currency,
                                    &money(tax),
                                );
                            }
                            if let Some(treatment) = &group.treatment {
                                serializer.breakdown_category(treatment);
                            }
                        },
                    );
                }
            });
        }
        if let Some(accounting) = &invoice.vat_accounting_total {
            let accounting_currency = accounting.currency.code();
            let value = money(accounting.value);
            self.structural(ubl::Namespace::Cac, "TaxTotal", |serializer| {
                serializer.leaf_attr(
                    ubl::Namespace::Cbc,
                    "TaxAmount",
                    "vat_accounting_total",
                    Term::BT(111),
                    &[("currencyID", accounting_currency)],
                    &value,
                );
            });
        }
    }

    // Serializes the VAT category of a breakdown group, mapped to its treatment field.
    fn breakdown_category(&mut self, treatment: &VatTreatment) {
        self.group(
            ubl::Namespace::Cac,
            "TaxCategory",
            "treatment",
            Term::BT(118),
            |serializer| {
                serializer.derived(
                    ubl::Namespace::Cbc,
                    "ID",
                    &[],
                    &treatment.category().to_string(),
                );
                serializer.derived(
                    ubl::Namespace::Cbc,
                    "Percent",
                    &[],
                    &plain(treatment.rate()),
                );
                if let VatTreatment::Exempt { code, text } = treatment {
                    if let Some(code) = code {
                        serializer.derived(
                            ubl::Namespace::Cbc,
                            "TaxExemptionReasonCode",
                            &[],
                            &code.to_string(),
                        );
                    }
                    if let Some(text) = text {
                        serializer.derived(
                            ubl::Namespace::Cbc,
                            "TaxExemptionReason",
                            &[],
                            text.as_ref(),
                        );
                    }
                }
                serializer.nested(ubl::Namespace::Cac, "TaxScheme", |serializer| {
                    serializer.derived(ubl::Namespace::Cbc, "ID", &[], "VAT");
                });
            },
        );
    }

    // Serializes the legal monetary total, each amount mapped to its own field.
    // The whole group is absent when the invoice states none of its amounts.
    fn legal_monetary_total(&mut self, invoice: &Invoice) {
        let attr = self.currency_attribute();
        let totals = [
            (
                "LineExtensionAmount",
                "line_net_total",
                Term::BT(106),
                invoice.line_net_total,
            ),
            (
                "TaxExclusiveAmount",
                "net_total",
                Term::BT(109),
                invoice.net_total,
            ),
            (
                "TaxInclusiveAmount",
                "gross_total",
                Term::BT(112),
                invoice.gross_total,
            ),
            (
                "AllowanceTotalAmount",
                "allowances_total",
                Term::BT(107),
                invoice.allowances_total,
            ),
            (
                "ChargeTotalAmount",
                "charges_total",
                Term::BT(108),
                invoice.charges_total,
            ),
            ("PrepaidAmount", "paid", Term::BT(113), invoice.paid),
            (
                "PayableRoundingAmount",
                "rounding",
                Term::BT(114),
                invoice.rounding,
            ),
            ("PayableAmount", "due", Term::BT(115), invoice.due),
        ];
        if totals.iter().all(|(_, _, _, value)| value.is_none()) {
            return;
        }
        self.structural(ubl::Namespace::Cac, "LegalMonetaryTotal", |serializer| {
            for (name, field, term, value) in totals {
                if let Some(value) = value {
                    serializer.leaf_attr(
                        ubl::Namespace::Cbc,
                        name,
                        field,
                        term,
                        &attr,
                        &money(value),
                    );
                }
            }
        });
    }

    // Serializes the invoice lines (`BG-25`), each a repeatable-group instance.
    fn lines(&mut self, lines: &[InvoiceLine]) {
        for (position, line) in lines.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive line index");
            self.repeatable(
                ubl::Namespace::Cac,
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
        let currency = self.currency_attribute();
        if let Some(id) = &line.id {
            self.leaf(ubl::Namespace::Cbc, "ID", "id", Term::BT(126), id.as_ref());
        }
        if let Some(note) = &line.note {
            self.leaf(
                ubl::Namespace::Cbc,
                "Note",
                "note",
                Term::BT(127),
                note.as_ref(),
            );
        }
        if let Some(quantity) = &line.quantity {
            self.leaf_attr(
                ubl::Namespace::Cbc,
                "InvoicedQuantity",
                "quantity",
                Term::BT(129),
                &[("unitCode", quantity.unit.code())],
                &plain(quantity.value),
            );
        }
        if let Some(net) = line.net_amount {
            self.leaf_attr(
                ubl::Namespace::Cbc,
                "LineExtensionAmount",
                "net_amount",
                Term::BT(131),
                &currency,
                &money(net),
            );
        }
        if let Some(reference) = &line.buyer_accounting_reference {
            self.leaf(
                ubl::Namespace::Cbc,
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
            self.structural(ubl::Namespace::Cac, "OrderLineReference", |serializer| {
                serializer.leaf(
                    ubl::Namespace::Cbc,
                    "LineID",
                    "order_line_reference",
                    Term::BT(132),
                    order.as_ref(),
                );
            });
        }
        if let Some(object) = &line.object {
            self.structural(ubl::Namespace::Cac, "DocumentReference", |serializer| {
                let Some(id) = &object.id else {
                    return;
                };
                match &object.scheme {
                    Some(scheme) => serializer.leaf_attr(
                        ubl::Namespace::Cbc,
                        "ID",
                        "object",
                        Term::BT(128),
                        &[("schemeID", &scheme.to_string())],
                        id.as_ref(),
                    ),
                    None => serializer.leaf(
                        ubl::Namespace::Cbc,
                        "ID",
                        "object",
                        Term::BT(128),
                        id.as_ref(),
                    ),
                }
            });
        }
        self.line_adjustments(&line.adjustments);
        if line.item.is_some() || line.vat.is_some() {
            self.item(line.item.as_ref(), line.vat.as_ref());
        }
        if let Some(price) = &line.price {
            self.line_price(price);
        }
    }

    // Serializes the line invoice period (`BG-26`).
    fn line_period(&mut self, period: Period) {
        self.group(
            ubl::Namespace::Cac,
            "InvoicePeriod",
            "period",
            Term::BG(26),
            |serializer| {
                if let Some(start) = period.start() {
                    serializer.field_leaf(ubl::Namespace::Cbc, "StartDate", "period", &date(start));
                }
                if let Some(end) = period.end() {
                    serializer.field_leaf(ubl::Namespace::Cbc, "EndDate", "period", &date(end));
                }
            },
        );
    }

    // Serializes the line-level allowances and charges (`BG-27`/`BG-28`).
    fn line_adjustments(&mut self, adjustments: &[LineAdjustment]) {
        for (position, adjustment) in adjustments.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive adjustment index");
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
                ubl::Namespace::Cac,
                "AllowanceCharge",
                "adjustments",
                term,
                instance,
                |serializer| {
                    serializer.charge_indicator(charge);
                    if let Some(reason) = &adjustment.reason {
                        serializer.adjustment_reason(reason);
                    }
                    if let Some(amount) = &adjustment.amount {
                        serializer.adjustment_amount(amount);
                    }
                },
            );
        }
    }

    // Serializes the item (`BG-31`), rebasing the classified tax category onto the line VAT.
    fn item(&mut self, item: Option<&Item>, vat: Option<&VatTreatment>) {
        self.trace.enter(ubl::Namespace::Cac, "Item");
        self.trace.push_field("item");
        self.trace.record_context();
        self.write_start(ubl::Namespace::Cac, "Item");

        if let Some(item) = item {
            self.item_head(item);
        }

        // The classified tax category is a sibling of the item in the model.
        self.trace.pop_context();
        if let Some(vat) = vat {
            self.classified_tax_category(vat);
        }

        if let Some(item) = item {
            self.item_attributes(item);
        }

        self.write_end(ubl::Namespace::Cac, "Item");
        self.trace.leave();
    }

    // Serializes the item fields that precede the classified tax category.
    fn item_head(&mut self, item: &Item) {
        if let Some(description) = &item.description {
            self.field_leaf(
                ubl::Namespace::Cbc,
                "Description",
                "description",
                description.as_ref(),
            );
        }
        if let Some(name) = &item.name {
            self.field_leaf(ubl::Namespace::Cbc, "Name", "name", name.as_ref());
        }
        if let Some(id) = &item.buyer_id {
            self.structural(
                ubl::Namespace::Cac,
                "BuyersItemIdentification",
                |serializer| {
                    serializer.field_leaf(ubl::Namespace::Cbc, "ID", "buyer_id", id.as_ref());
                },
            );
        }
        if let Some(id) = &item.seller_id {
            self.structural(
                ubl::Namespace::Cac,
                "SellersItemIdentification",
                |serializer| {
                    serializer.field_leaf(ubl::Namespace::Cbc, "ID", "seller_id", id.as_ref());
                },
            );
        }
        if let Some(standard) = &item.standard_id {
            self.structural(
                ubl::Namespace::Cac,
                "StandardItemIdentification",
                |serializer| {
                    let Some(id) = &standard.id else {
                        return;
                    };
                    match &standard.issuer {
                        Some(issuer) => serializer.field_leaf_attr(
                            ubl::Namespace::Cbc,
                            "ID",
                            "standard_id",
                            &[("schemeID", &issuer.to_string())],
                            id.as_ref(),
                        ),
                        None => serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "ID",
                            "standard_id",
                            id.as_ref(),
                        ),
                    }
                },
            );
        }
        if let Some(country) = item.country_of_origin {
            self.structural(ubl::Namespace::Cac, "OriginCountry", |serializer| {
                serializer.field_leaf(
                    ubl::Namespace::Cbc,
                    "IdentificationCode",
                    "country_of_origin",
                    country.alpha2(),
                );
            });
        }
        for classification in &item.classifications {
            self.structural(
                ubl::Namespace::Cac,
                "CommodityClassification",
                |serializer| {
                    let Some(id) = &classification.id else {
                        return;
                    };
                    let mut attributes = Vec::new();
                    if let Some(scheme) = &classification.scheme {
                        attributes.push(("listID".to_owned(), scheme.to_string()));
                    }
                    if let Some(version) = &classification.version {
                        attributes.push(("listVersionID".to_owned(), version.as_ref().to_owned()));
                    }
                    let borrowed: Vec<(&str, &str)> = attributes
                        .iter()
                        .map(|(key, value)| (key.as_str(), value.as_str()))
                        .collect();
                    serializer.field_leaf_attr(
                        ubl::Namespace::Cbc,
                        "ItemClassificationCode",
                        "classifications",
                        &borrowed,
                        id.as_ref(),
                    );
                },
            );
        }
    }

    // Serializes the item attributes (`BG-32`), which follow the classified tax category.
    fn item_attributes(&mut self, item: &Item) {
        for attribute in &item.attributes {
            self.structural(
                ubl::Namespace::Cac,
                "AdditionalItemProperty",
                |serializer| {
                    if let Some(name) = &attribute.name {
                        serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "Name",
                            "attributes",
                            name.as_ref(),
                        );
                    }
                    if let Some(value) = &attribute.value {
                        serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "Value",
                            "attributes",
                            value.as_ref(),
                        );
                    }
                },
            );
        }
    }

    // Serializes the line VAT as a UBL classified tax category, mapped to the VAT field.
    fn classified_tax_category(&mut self, vat: &VatTreatment) {
        self.group(
            ubl::Namespace::Cac,
            "ClassifiedTaxCategory",
            "vat",
            Term::BG(30),
            |serializer| {
                serializer.field_leaf(
                    ubl::Namespace::Cbc,
                    "ID",
                    "vat",
                    &vat.category().to_string(),
                );
                serializer.field_leaf(ubl::Namespace::Cbc, "Percent", "vat", &plain(vat.rate()));
                serializer.structural(ubl::Namespace::Cac, "TaxScheme", |serializer| {
                    serializer.field_leaf(ubl::Namespace::Cbc, "ID", "vat", "VAT");
                });
            },
        );
    }

    // Serializes the line price (`BG-29`), every price with its own scale.
    fn line_price(&mut self, price: &Price) {
        let currency = self.currency_attribute();
        self.group(
            ubl::Namespace::Cac,
            "Price",
            "price",
            Term::BG(29),
            |serializer| {
                if let Some(net) = price.net {
                    serializer.leaf_attr(
                        ubl::Namespace::Cbc,
                        "PriceAmount",
                        "net",
                        Term::BT(146),
                        &currency,
                        &plain(net),
                    );
                }
                if let Some(base) = price.base_quantity {
                    serializer.field_leaf_attr(
                        ubl::Namespace::Cbc,
                        "BaseQuantity",
                        "price",
                        &[("unitCode", base.unit.code())],
                        &plain(base.value),
                    );
                }
                if let Some(discount) = price.discount {
                    serializer.structural(ubl::Namespace::Cac, "AllowanceCharge", |serializer| {
                        serializer.field_leaf(
                            ubl::Namespace::Cbc,
                            "ChargeIndicator",
                            "price",
                            "false",
                        );
                        serializer.field_leaf_attr(
                            ubl::Namespace::Cbc,
                            "Amount",
                            "price",
                            &currency,
                            &plain(discount),
                        );
                        if let Some(gross) = price.gross {
                            serializer.field_leaf_attr(
                                ubl::Namespace::Cbc,
                                "BaseAmount",
                                "price",
                                &currency,
                                &plain(gross),
                            );
                        }
                    });
                }
            },
        );
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
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
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
    fn field_leaf(
        &mut self,
        namespace: ubl::Namespace,
        name: &str,
        field: &'static str,
        value: &str,
    ) {
        self.write_field_leaf(namespace, name, field, &[], value);
    }

    // Writes a leaf with attributes mapped to a model field, never filtered.
    fn field_leaf_attr(
        &mut self,
        namespace: ubl::Namespace,
        name: &str,
        field: &'static str,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        self.write_field_leaf(namespace, name, field, attributes, value);
    }

    fn write_field_leaf(
        &mut self,
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
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
        namespace: ubl::Namespace,
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
    fn structural(&mut self, namespace: ubl::Namespace, name: &str, body: impl FnOnce(&mut Self)) {
        self.trace.enter(namespace, name);
        self.trace.record_root();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    // Writes a wrapper mapped to the enclosing context, without a new segment.
    fn nested(&mut self, namespace: ubl::Namespace, name: &str, body: impl FnOnce(&mut Self)) {
        self.trace.enter(namespace, name);
        self.trace.record_context();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    // Serializes an allowance or charge VAT category, mapped to the enclosing adjustment.
    fn tax_category(&mut self, vat: &VatTreatment) {
        self.nested(ubl::Namespace::Cac, "TaxCategory", |serializer| {
            serializer.derived(ubl::Namespace::Cbc, "ID", &[], &vat.category().to_string());
            serializer.derived(ubl::Namespace::Cbc, "Percent", &[], &plain(vat.rate()));
            serializer.nested(ubl::Namespace::Cac, "TaxScheme", |serializer| {
                serializer.derived(ubl::Namespace::Cbc, "ID", &[], "VAT");
            });
        });
    }

    // ---- raw XML ---------------------------------------------------------

    fn write_element(
        &mut self,
        namespace: ubl::Namespace,
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

    fn write_start(&mut self, namespace: ubl::Namespace, name: &str) {
        self.write(Event::Start(BytesStart::new(qname(namespace, name))));
    }

    fn write_end(&mut self, namespace: ubl::Namespace, name: &str) {
        self.write(Event::End(BytesEnd::new(qname(namespace, name))));
    }

    fn write(&mut self, event: Event<'_>) {
        self.inner
            .write_event(event)
            .expect("writing XML to an in-memory buffer never fails");
    }
}

// The record-form qualified name of an element, prefixed for its namespace.
fn qname(namespace: ubl::Namespace, name: &str) -> String {
    let prefix = namespace.prefix();
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
