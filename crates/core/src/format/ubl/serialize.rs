//! The UBL writing walk, generic over the interfaces of the semantic model.

use super::Namespace;
use crate::prelude::*;
use crate::{
    Adjustment, AdjustmentAmount, AdjustmentReason, BinaryObject, Buyer, Contact, Currency,
    Delivery, DocumentBuilder, ElectronicAddress, Invoice, InvoiceReference, Item, LegalEntity,
    Line, LineAdjustment, Note, ObjectReference, OperationalEntity, Payee, PaymentDetails,
    PaymentInstructions, Period, PostalAddress, Price, Seller, Serializer, SupportingDocument,
    TaxRepresentative, Term, Ubl, VatPoint, VatTreatment,
};

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
    let text = note.text.as_ref()?.as_ref().to_owned();
    Some(match &note.subject_code {
        Some(code) if !drop_subject => format!("#{}#{}", code.as_ref(), text),
        _ => text,
    })
}

// The `currencyID` attribute of an amount, absent without the invoice currency.
fn currency_attribute<I: Invoice>(invoice: &mut I) -> Vec<(&'static str, &'static str)> {
    invoice
        .currency()
        .as_ref()
        .map(|currency| vec![("currencyID", Currency::code(currency))])
        .unwrap_or_default()
}

// The VAT point date (`BT-7`), when the point is a date.
fn tax_point_date<I: Invoice>(invoice: &mut I) -> Option<VatPointDate> {
    match *invoice.vat_point() {
        Some(VatPoint::Date(date)) => Some(VatPointDate(date)),
        _ => None,
    }
}

// The VAT point event code (`BT-8`), when the point is an event.
fn vat_point_event<I: Invoice>(invoice: &mut I) -> Option<String> {
    match *invoice.vat_point() {
        Some(VatPoint::Event(event)) => Some(u16::from(event).to_string()),
        _ => None,
    }
}

/// Writes the whole document of the builder under the UBL root element,
/// filling the dictionary and the abbreviation table in the same pass.
///
/// An implementation of `Serializable<Ubl, N>` for a concrete invoice type calls this walk.
pub fn serialize<I: Invoice, N: crate::Namespace + From<Namespace>>(
    serializer: &mut Serializer<Ubl, N>,
    builder: &mut DocumentBuilder<I>,
) {
    let DocumentBuilder {
        invoice,
        profile,
        business_process,
    } = builder;
    let currency = currency_attribute(invoice);
    serializer.root(|serializer| {
        // Regulatory-flow fields: the specification identifier (BT-24) and business process (BT-23).
        serializer.rooted(Namespace::Cbc, "CustomizationID", &[], &profile.to_string());
        if let Some(process) = business_process {
            serializer.rooted(Namespace::Cbc, "ProfileID", &[], process.as_ref());
        }

        if let Some(number) = invoice.number() {
            serializer.leaf(Namespace::Cbc, "ID", "number", Term::BT(1), number.as_ref());
        }
        if let Some(issued) = *invoice.issue_date() {
            serializer.leaf(
                Namespace::Cbc,
                "IssueDate",
                "issue_date",
                Term::BT(2),
                &date(issued),
            );
        }
        if let Some(due) = *invoice.payment_due_date() {
            serializer.leaf(
                Namespace::Cbc,
                "DueDate",
                "payment_due_date",
                Term::BT(9),
                &date(due),
            );
        }
        serializer.leaf(
            Namespace::Cbc,
            "InvoiceTypeCode",
            "type_code",
            Term::BT(3),
            &invoice.type_code().to_string(),
        );
        serializer.notes(invoice.notes());
        if let Some(VatPointDate(point)) = tax_point_date(invoice) {
            serializer.leaf(
                Namespace::Cbc,
                "TaxPointDate",
                "vat_point",
                Term::BT(7),
                &date(point),
            );
        }
        if let Some(currency) = invoice.currency() {
            serializer.leaf(
                Namespace::Cbc,
                "DocumentCurrencyCode",
                "currency",
                Term::BT(5),
                currency.code(),
            );
        }
        if let Some(accounting) = invoice.vat_accounting_total() {
            serializer.leaf(
                Namespace::Cbc,
                "TaxCurrencyCode",
                "vat_accounting_total",
                Term::BT(6),
                accounting.currency.code(),
            );
        }
        if let Some(reference) = invoice.buyer_accounting_reference() {
            serializer.leaf(
                Namespace::Cbc,
                "AccountingCost",
                "buyer_accounting_reference",
                Term::BT(19),
                reference.as_ref(),
            );
        }
        if let Some(reference) = invoice.buyer_reference() {
            serializer.leaf(
                Namespace::Cbc,
                "BuyerReference",
                "buyer_reference",
                Term::BT(10),
                reference.as_ref(),
            );
        }

        serializer.invoice_period(invoice);
        serializer.order_reference(invoice);
        serializer.billing_references(invoice.preceding_invoices());
        if let Some(reference) = invoice.despatch_advice_reference() {
            serializer.reference_group(
                Namespace::Cac,
                "DespatchDocumentReference",
                Term::BT(16),
                reference.as_ref(),
            );
        }
        if let Some(reference) = invoice.receiving_advice_reference() {
            serializer.reference_group(
                Namespace::Cac,
                "ReceiptDocumentReference",
                Term::BT(15),
                reference.as_ref(),
            );
        }
        if let Some(reference) = invoice.tender_or_lot_reference() {
            serializer.reference_group(
                Namespace::Cac,
                "OriginatorDocumentReference",
                Term::BT(17),
                reference.as_ref(),
            );
        }
        if let Some(reference) = invoice.contract_reference() {
            serializer.reference_group(
                Namespace::Cac,
                "ContractDocumentReference",
                Term::BT(12),
                reference.as_ref(),
            );
        }
        serializer.additional_documents(invoice);
        if let Some(reference) = invoice.project_reference() {
            serializer.reference_group(
                Namespace::Cac,
                "ProjectReference",
                Term::BT(11),
                reference.as_ref(),
            );
        }

        if let Some(seller) = invoice.seller() {
            serializer.supplier_party(seller);
        }
        if let Some(buyer) = invoice.buyer() {
            serializer.customer_party(buyer);
        }
        if let Some(payee) = invoice.payee() {
            serializer.payee_party(payee);
        }
        if let Some(representative) = invoice.tax_representative() {
            serializer.tax_representative_party(representative);
        }
        if let Some(delivery) = invoice.delivery() {
            serializer.delivery(delivery);
        }
        if let Some(payment) = invoice.payment() {
            serializer.payment_means(payment);
        }
        if let Some(terms) = invoice.payment_terms() {
            serializer.group(
                Namespace::Cac,
                "PaymentTerms",
                "payment_terms",
                Term::BT(20),
                |serializer| {
                    serializer.field_leaf(Namespace::Cbc, "Note", "payment_terms", terms.as_ref());
                },
            );
        }
        serializer.adjustments(invoice.adjustments(), &currency);
        serializer.tax_total(invoice);
        serializer.legal_monetary_total(invoice);
        serializer.lines(invoice.lines(), &currency);
    });
}

impl<N: crate::Namespace + From<Namespace>> Serializer<Ubl, N> {
    // Serializes the notes (`BG-1`), each a repeatable single-value element.
    fn notes(&mut self, notes: &[Note]) {
        let drop_subject = self.forbids(Term::BT(21));
        for (position, note) in notes.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive note index");
            let Some(text) = note_text(note, drop_subject) else {
                continue;
            };
            self.repeatable_leaf(
                Namespace::Cbc,
                "Note",
                "notes",
                Term::BG(1),
                instance,
                &text,
            );
        }
    }

    // Serializes the invoice period (`BG-14`) and the VAT point date code (`BT-8`).
    fn invoice_period<I: Invoice>(&mut self, invoice: &mut I) {
        let event = vat_point_event(invoice);
        if invoice.invoicing_period().is_none() && event.is_none() {
            return;
        }
        self.group(
            Namespace::Cac,
            "InvoicePeriod",
            "invoicing_period",
            Term::BG(14),
            |serializer| {
                if let Some(period) = *invoice.invoicing_period() {
                    if let Some(start) = period.start() {
                        serializer.field_leaf(
                            Namespace::Cbc,
                            "StartDate",
                            "invoicing_period",
                            &date(start),
                        );
                    }
                    if let Some(end) = period.end() {
                        serializer.field_leaf(
                            Namespace::Cbc,
                            "EndDate",
                            "invoicing_period",
                            &date(end),
                        );
                    }
                }
                if let Some(code) = event {
                    serializer.field_leaf(Namespace::Cbc, "DescriptionCode", "vat_point", &code);
                }
            },
        );
    }

    // Serializes the order and sales order references (`BT-13`/`BT-14`).
    fn order_reference<I: Invoice>(&mut self, invoice: &mut I) {
        if invoice.purchase_order_reference().is_none() && invoice.sales_order_reference().is_none()
        {
            return;
        }
        self.structural(Namespace::Cac, "OrderReference", |serializer| {
            if let Some(order) = invoice.purchase_order_reference() {
                serializer.leaf(
                    Namespace::Cbc,
                    "ID",
                    "purchase_order_reference",
                    Term::BT(13),
                    order.as_ref(),
                );
            }
            if let Some(sales) = invoice.sales_order_reference() {
                serializer.leaf(
                    Namespace::Cbc,
                    "SalesOrderID",
                    "sales_order_reference",
                    Term::BT(14),
                    sales.as_ref(),
                );
            }
        });
    }

    // Serializes the preceding invoice references (`BG-3`).
    fn billing_references(&mut self, preceding: &[InvoiceReference]) {
        for (position, invoice) in preceding.iter().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive reference index");
            self.repeatable(
                Namespace::Cac,
                "BillingReference",
                "preceding_invoices",
                Term::BG(3),
                instance,
                |serializer| {
                    serializer.structural(
                        Namespace::Cac,
                        "InvoiceDocumentReference",
                        |serializer| {
                            if let Some(number) = &invoice.number {
                                serializer.field_leaf(
                                    Namespace::Cbc,
                                    "ID",
                                    "number",
                                    number.as_ref(),
                                );
                            }
                            if let Some(issued) = invoice.issue_date {
                                serializer.field_leaf(
                                    Namespace::Cbc,
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
    fn reference_group(&mut self, namespace: Namespace, element: &str, term: Term, id: &str) {
        self.group(namespace, element, field_of(term), term, |serializer| {
            serializer.field_leaf(Namespace::Cbc, "ID", field_of(term), id);
        });
    }

    // Serializes the invoiced object (`BT-18`) and the supporting documents (`BG-24`).
    fn additional_documents<I: Invoice>(&mut self, invoice: &mut I) {
        if let Some(object) = invoice.object() {
            self.additional_object(object);
        }
        for (position, document) in invoice.supporting_documents().iter_mut().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive reference index");
            self.additional_supporting(document, instance);
        }
    }

    // Serializes the invoiced object identifier as an additional document reference.
    fn additional_object(&mut self, object: &ObjectReference) {
        self.group(
            Namespace::Cac,
            "AdditionalDocumentReference",
            "object",
            Term::BT(18),
            |serializer| {
                if let Some(id) = &object.id {
                    match &object.scheme {
                        Some(scheme) => serializer.derived(
                            Namespace::Cbc,
                            "ID",
                            &[("schemeID", &scheme.to_string())],
                            id.as_ref(),
                        ),
                        None => serializer.derived(Namespace::Cbc, "ID", &[], id.as_ref()),
                    }
                }
                serializer.derived(Namespace::Cbc, "DocumentTypeCode", &[], "130");
            },
        );
    }

    // Serializes a supporting document (`BG-24`) as an additional document reference.
    fn additional_supporting(&mut self, document: &SupportingDocument, instance: NonZeroUsize) {
        self.repeatable(
            Namespace::Cac,
            "AdditionalDocumentReference",
            "supporting_documents",
            Term::BG(24),
            instance,
            |serializer| {
                if let Some(reference) = &document.reference {
                    serializer.field_leaf(Namespace::Cbc, "ID", "reference", reference.as_ref());
                }
                if let Some(description) = &document.description {
                    serializer.leaf(
                        Namespace::Cbc,
                        "DocumentDescription",
                        "description",
                        Term::BT(123),
                        description.as_ref(),
                    );
                }
                if let Some(location) = &document.external_location {
                    if !serializer.forbids(Term::BT(124)) {
                        serializer.structural(Namespace::Cac, "Attachment", |serializer| {
                            serializer.structural(
                                Namespace::Cac,
                                "ExternalReference",
                                |serializer| {
                                    serializer.field_leaf(
                                        Namespace::Cbc,
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
                    serializer.structural(Namespace::Cac, "Attachment", |serializer| {
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
            Namespace::Cbc,
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
    fn supplier_party<S: Seller>(&mut self, seller: &mut S) {
        self.structural(Namespace::Cac, "AccountingSupplierParty", |serializer| {
            serializer.group(
                Namespace::Cac,
                "Party",
                "seller",
                Term::BG(4),
                |serializer| {
                    if let Some(address) = seller.electronic_address() {
                        serializer.endpoint(address, Term::BT(34));
                    }
                    serializer.party_identifiers(seller.identifiers(), Term::BT(29));
                    if let Some(trading) = seller.trading_name() {
                        serializer.party_name(trading.as_ref(), Term::BT(28));
                    }
                    if let Some(address) = seller.address() {
                        serializer.postal_address("PostalAddress", Term::BG(5), address);
                    }
                    if let Some(vat) = seller.vat() {
                        serializer.party_tax_scheme(&vat.to_string(), "VAT", Term::BT(31));
                    }
                    if let Some(registration) = seller.tax_registration() {
                        serializer.party_tax_scheme(registration.as_ref(), "FC", Term::BT(32));
                    }
                    serializer.structural(Namespace::Cac, "PartyLegalEntity", |serializer| {
                        if let Some(name) = seller.name() {
                            serializer.leaf(
                                Namespace::Cbc,
                                "RegistrationName",
                                "name",
                                Term::BT(27),
                                name.as_ref(),
                            );
                        }
                        if let Some(entity) = seller.legal_entity() {
                            serializer.legal_entity_id(entity, Term::BT(30));
                        }
                        if let Some(form) = seller.additional_legal_information() {
                            serializer.leaf(
                                Namespace::Cbc,
                                "CompanyLegalForm",
                                "additional_legal_information",
                                Term::BT(33),
                                form.as_ref(),
                            );
                        }
                    });
                    if let Some(contact) = seller.contact() {
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
        });
    }

    // Serializes the buyer (`BG-7`) under the customer party wrapper.
    fn customer_party<B: Buyer>(&mut self, buyer: &mut B) {
        self.structural(Namespace::Cac, "AccountingCustomerParty", |serializer| {
            serializer.group(
                Namespace::Cac,
                "Party",
                "buyer",
                Term::BG(7),
                |serializer| {
                    if let Some(address) = buyer.electronic_address() {
                        serializer.endpoint(address, Term::BT(49));
                    }
                    serializer.party_identifiers(buyer.identifiers(), Term::BT(46));
                    if let Some(trading) = buyer.trading_name() {
                        serializer.party_name(trading.as_ref(), Term::BT(45));
                    }
                    if let Some(address) = buyer.address() {
                        serializer.postal_address("PostalAddress", Term::BG(8), address);
                    }
                    if let Some(vat) = buyer.vat() {
                        serializer.party_tax_scheme(&vat.to_string(), "VAT", Term::BT(48));
                    }
                    serializer.structural(Namespace::Cac, "PartyLegalEntity", |serializer| {
                        if let Some(name) = buyer.name() {
                            serializer.leaf(
                                Namespace::Cbc,
                                "RegistrationName",
                                "name",
                                Term::BT(44),
                                name.as_ref(),
                            );
                        }
                        if let Some(entity) = buyer.legal_entity() {
                            serializer.legal_entity_id(entity, Term::BT(47));
                        }
                    });
                    if let Some(contact) = buyer.contact() {
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
        });
    }

    // Serializes the payee (`BG-10`).
    fn payee_party<P: Payee>(&mut self, payee: &mut P) {
        self.group(
            Namespace::Cac,
            "PayeeParty",
            "payee",
            Term::BG(10),
            |serializer| {
                serializer.party_identifiers(payee.identifiers(), Term::BT(60));
                if let Some(name) = payee.name() {
                    serializer.party_name(name.as_ref(), Term::BT(59));
                }
                if let Some(entity) = payee.legal_entity() {
                    serializer.structural(Namespace::Cac, "PartyLegalEntity", |serializer| {
                        serializer.legal_entity_id(entity, Term::BT(61));
                    });
                }
            },
        );
    }

    // Serializes the seller tax representative (`BG-11`).
    fn tax_representative_party<T: TaxRepresentative>(&mut self, representative: &mut T) {
        self.group(
            Namespace::Cac,
            "TaxRepresentativeParty",
            "tax_representative",
            Term::BG(11),
            |serializer| {
                if let Some(name) = representative.name() {
                    serializer.party_name(name.as_ref(), Term::BT(62));
                }
                if let Some(address) = representative.address() {
                    serializer.postal_address("PostalAddress", Term::BG(12), address);
                }
                if let Some(vat) = representative.vat() {
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
                Namespace::Cbc,
                "EndpointID",
                field_of(term),
                term,
                &[("schemeID", &scheme.to_string())],
                id.as_ref(),
            ),
            None => self.leaf(
                Namespace::Cbc,
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
                Namespace::Cac,
                "PartyIdentification",
                field_of(term),
                term,
                |serializer| {
                    let Some(id) = &identifier.id else {
                        return;
                    };
                    match &identifier.issuer {
                        Some(issuer) => serializer.field_leaf_attr(
                            Namespace::Cbc,
                            "ID",
                            field_of(term),
                            &[("schemeID", &issuer.to_string())],
                            id.as_ref(),
                        ),
                        None => {
                            serializer.field_leaf(Namespace::Cbc, "ID", field_of(term), id.as_ref())
                        }
                    }
                },
            );
        }
    }

    // Serializes a party trading name (`BT-28`/`BT-45`).
    fn party_name(&mut self, name: &str, term: Term) {
        self.group(
            Namespace::Cac,
            "PartyName",
            field_of(term),
            term,
            |serializer| {
                serializer.field_leaf(Namespace::Cbc, "Name", field_of(term), name);
            },
        );
    }

    // Serializes a party tax scheme carrying a company id under a scheme code.
    fn party_tax_scheme(&mut self, company: &str, scheme: &str, term: Term) {
        self.group(
            Namespace::Cac,
            "PartyTaxScheme",
            field_of(term),
            term,
            |serializer| {
                serializer.field_leaf(Namespace::Cbc, "CompanyID", field_of(term), company);
                serializer.structural(Namespace::Cac, "TaxScheme", |serializer| {
                    serializer.field_leaf(Namespace::Cbc, "ID", field_of(term), scheme);
                });
            },
        );
    }

    // Serializes the legal registration id (`BT-30`/`BT-47`/`BT-61`).
    fn legal_entity_id(&mut self, entity: &LegalEntity, term: Term) {
        let Some(id) = &entity.id else {
            return;
        };
        match &entity.issuer {
            Some(issuer) => self.leaf_attr(
                Namespace::Cbc,
                "CompanyID",
                field_of(term),
                term,
                &[("schemeID", &issuer.to_string())],
                id.as_ref(),
            ),
            None => self.leaf(
                Namespace::Cbc,
                "CompanyID",
                field_of(term),
                term,
                id.as_ref(),
            ),
        }
    }

    // Serializes a contact group (`BG-6`/`BG-9`).
    fn contact<C: Contact>(
        &mut self,
        contact: &mut C,
        group: Term,
        name: Term,
        phone: Term,
        mail: Term,
    ) {
        self.group(
            Namespace::Cac,
            "Contact",
            field_of(group),
            group,
            |serializer| {
                if let Some(value) = contact.name() {
                    serializer.leaf(Namespace::Cbc, "Name", field_of(name), name, value.as_ref());
                }
                if let Some(value) = contact.telephone() {
                    serializer.leaf(
                        Namespace::Cbc,
                        "Telephone",
                        field_of(phone),
                        phone,
                        value.as_ref(),
                    );
                }
                if let Some(value) = contact.email() {
                    serializer.leaf(
                        Namespace::Cbc,
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
        self.group(Namespace::Cac, element, "address", group, |serializer| {
            if let Some(line) = &address.line1 {
                serializer.field_leaf(Namespace::Cbc, "StreetName", "line1", line.as_ref());
            }
            if let Some(line) = &address.line2 {
                serializer.field_leaf(
                    Namespace::Cbc,
                    "AdditionalStreetName",
                    "line2",
                    line.as_ref(),
                );
            }
            if let Some(city) = &address.city {
                serializer.field_leaf(Namespace::Cbc, "CityName", "city", city.as_ref());
            }
            if let Some(zip) = &address.postal_code {
                serializer.field_leaf(Namespace::Cbc, "PostalZone", "postal_code", zip.as_ref());
            }
            if let Some(subdivision) = &address.country_subdivision {
                serializer.field_leaf(
                    Namespace::Cbc,
                    "CountrySubentity",
                    "country_subdivision",
                    subdivision.as_ref(),
                );
            }
            if let Some(line) = &address.line3 {
                serializer.structural(Namespace::Cac, "AddressLine", |serializer| {
                    serializer.field_leaf(Namespace::Cbc, "Line", "line3", line.as_ref());
                });
            }
            if let Some(country) = &address.country {
                serializer.structural(Namespace::Cac, "Country", |serializer| {
                    serializer.field_leaf(
                        Namespace::Cbc,
                        "IdentificationCode",
                        "country",
                        country.alpha2(),
                    );
                });
            }
        });
    }

    // Serializes the delivery information (`BG-13`).
    fn delivery<D: Delivery>(&mut self, delivery: &mut D) {
        self.group(
            Namespace::Cac,
            "Delivery",
            "delivery",
            Term::BG(13),
            |serializer| {
                if let Some(date_value) = *delivery.date() {
                    serializer.field_leaf(
                        Namespace::Cbc,
                        "ActualDeliveryDate",
                        "date",
                        &date(date_value),
                    );
                }
                if delivery.location().is_some() || delivery.address().is_some() {
                    serializer.structural(Namespace::Cac, "DeliveryLocation", |serializer| {
                        if let Some(location) = delivery.location() {
                            if let Some(id) = &location.id {
                                match &location.issuer {
                                    Some(issuer) => serializer.field_leaf_attr(
                                        Namespace::Cbc,
                                        "ID",
                                        "location",
                                        &[("schemeID", &issuer.to_string())],
                                        id.as_ref(),
                                    ),
                                    None => serializer.field_leaf(
                                        Namespace::Cbc,
                                        "ID",
                                        "location",
                                        id.as_ref(),
                                    ),
                                }
                            }
                        }
                        if let Some(address) = delivery.address() {
                            serializer.postal_address("Address", Term::BG(15), address);
                        }
                    });
                }
                if let Some(name) = delivery.name() {
                    serializer.structural(Namespace::Cac, "DeliveryParty", |serializer| {
                        serializer.party_name(name.as_ref(), Term::BT(70));
                    });
                }
            },
        );
    }

    // Serializes the payment instructions (`BG-16`).
    fn payment_means(&mut self, payment: &PaymentInstructions) {
        self.group(
            Namespace::Cac,
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
                        Namespace::Cbc,
                        "PaymentMeansCode",
                        "means",
                        &borrowed,
                        &means.to_string(),
                    );
                }
                if let Some(reference) = &payment.remittance_information {
                    serializer.field_leaf(
                        Namespace::Cbc,
                        "PaymentID",
                        "remittance_information",
                        reference.as_ref(),
                    );
                }
                match &payment.details {
                    Some(PaymentDetails::CreditTransfers(transfers)) => {
                        for transfer in transfers.iter() {
                            serializer.structural(
                                Namespace::Cac,
                                "PayeeFinancialAccount",
                                |serializer| {
                                    if let Some(account) = &transfer.account {
                                        serializer.field_leaf(
                                            Namespace::Cbc,
                                            "ID",
                                            "account",
                                            account.as_ref(),
                                        );
                                    }
                                    if let Some(name) = &transfer.account_name {
                                        serializer.field_leaf(
                                            Namespace::Cbc,
                                            "Name",
                                            "account_name",
                                            name.as_ref(),
                                        );
                                    }
                                    if let Some(provider) = &transfer.provider {
                                        serializer.structural(
                                            Namespace::Cac,
                                            "FinancialInstitutionBranch",
                                            |serializer| {
                                                serializer.field_leaf(
                                                    Namespace::Cbc,
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
                        serializer.structural(Namespace::Cac, "CardAccount", |serializer| {
                            if let Some(number) = &card.primary_account_number {
                                serializer.field_leaf(
                                    Namespace::Cbc,
                                    "PrimaryAccountNumberID",
                                    "primary_account_number",
                                    number.as_ref(),
                                );
                            }
                            if let Some(holder) = &card.holder_name {
                                serializer.field_leaf(
                                    Namespace::Cbc,
                                    "HolderName",
                                    "holder_name",
                                    holder.as_ref(),
                                );
                            }
                        });
                    }
                    Some(PaymentDetails::DirectDebit(debit)) => {
                        serializer.structural(Namespace::Cac, "PaymentMandate", |serializer| {
                            if let Some(reference) = &debit.mandate_reference {
                                serializer.field_leaf(
                                    Namespace::Cbc,
                                    "ID",
                                    "mandate_reference",
                                    reference.as_ref(),
                                );
                            }
                            if let Some(creditor) = &debit.creditor_identifier {
                                serializer.field_leaf(
                                    Namespace::Cbc,
                                    "PayerPartyID",
                                    "creditor_identifier",
                                    creditor.as_ref(),
                                );
                            }
                            if let Some(account) = &debit.debited_account {
                                serializer.structural(
                                    Namespace::Cac,
                                    "PayerFinancialAccount",
                                    |serializer| {
                                        serializer.field_leaf(
                                            Namespace::Cbc,
                                            "ID",
                                            "debited_account",
                                            account.as_ref(),
                                        );
                                    },
                                );
                            }
                        });
                    }
                    None => {}
                }
            },
        );
    }

    // Serializes the document-level allowances and charges (`BG-20`/`BG-21`).
    fn adjustments(&mut self, adjustments: &[Adjustment], currency: &[(&str, &str)]) {
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
                Namespace::Cac,
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
                        serializer.adjustment_amount(amount, currency);
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
                Namespace::Cbc,
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
            self.derived(Namespace::Cbc, "AllowanceChargeReasonCode", &[], &code);
        }
        if let Some(text) = text {
            self.derived(Namespace::Cbc, "AllowanceChargeReason", &[], text.as_ref());
        }
    }

    // Serializes the amount of an adjustment, absolute or relative, mapped to the amount field.
    fn adjustment_amount(&mut self, amount: &AdjustmentAmount, currency: &[(&str, &str)]) {
        match amount {
            AdjustmentAmount::Relative { amount, rate, base } => {
                self.field_leaf(
                    Namespace::Cbc,
                    "MultiplierFactorNumeric",
                    "amount",
                    &plain(Decimal::from(*rate)),
                );
                self.field_leaf_attr(
                    Namespace::Cbc,
                    "Amount",
                    "amount",
                    currency,
                    &money(*amount),
                );
                self.field_leaf_attr(
                    Namespace::Cbc,
                    "BaseAmount",
                    "amount",
                    currency,
                    &money(*base),
                );
            }
            AdjustmentAmount::Absolute(amount) => {
                self.field_leaf_attr(
                    Namespace::Cbc,
                    "Amount",
                    "amount",
                    currency,
                    &money(*amount),
                );
            }
        }
    }

    // Serializes the tax total (`BT-110`, `BG-23`) and the accounting-currency tax total
    // (`BT-111`). The first total is absent without the VAT total and the breakdown.
    fn tax_total<I: Invoice>(&mut self, invoice: &mut I) {
        let currency = currency_attribute(invoice);
        if invoice.vat_total().is_some() || !invoice.vat_breakdown().is_empty() {
            self.structural(Namespace::Cac, "TaxTotal", |serializer| {
                if let Some(total) = *invoice.vat_total() {
                    serializer.leaf_attr(
                        Namespace::Cbc,
                        "TaxAmount",
                        "vat_total",
                        Term::BT(110),
                        &currency,
                        &money(total),
                    );
                }
                for (position, group) in invoice.vat_breakdown().iter_mut().enumerate() {
                    let instance =
                        NonZeroUsize::new(position + 1).expect("a positive breakdown index");
                    serializer.repeatable(
                        Namespace::Cac,
                        "TaxSubtotal",
                        "vat_breakdown",
                        Term::BG(23),
                        instance,
                        |serializer| {
                            if let Some(taxable) = group.taxable {
                                serializer.leaf_attr(
                                    Namespace::Cbc,
                                    "TaxableAmount",
                                    "taxable",
                                    Term::BT(116),
                                    &currency,
                                    &money(taxable),
                                );
                            }
                            if let Some(tax) = group.tax {
                                serializer.leaf_attr(
                                    Namespace::Cbc,
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
        if let Some(accounting) = invoice.vat_accounting_total() {
            let accounting_currency = accounting.currency.code();
            let value = money(accounting.value);
            self.structural(Namespace::Cac, "TaxTotal", |serializer| {
                serializer.leaf_attr(
                    Namespace::Cbc,
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
            Namespace::Cac,
            "TaxCategory",
            "treatment",
            Term::BT(118),
            |serializer| {
                serializer.derived(Namespace::Cbc, "ID", &[], &treatment.category().to_string());
                serializer.derived(Namespace::Cbc, "Percent", &[], &plain(treatment.rate()));
                if let VatTreatment::Exempt { code, text } = treatment {
                    if let Some(code) = code {
                        serializer.derived(
                            Namespace::Cbc,
                            "TaxExemptionReasonCode",
                            &[],
                            &code.to_string(),
                        );
                    }
                    if let Some(text) = text {
                        serializer.derived(
                            Namespace::Cbc,
                            "TaxExemptionReason",
                            &[],
                            text.as_ref(),
                        );
                    }
                }
                serializer.nested(Namespace::Cac, "TaxScheme", |serializer| {
                    serializer.derived(Namespace::Cbc, "ID", &[], "VAT");
                });
            },
        );
    }

    // Serializes the legal monetary total, each amount mapped to its own field.
    // The whole group is absent when the invoice states none of its amounts.
    fn legal_monetary_total<I: Invoice>(&mut self, invoice: &mut I) {
        let attr = currency_attribute(invoice);
        let totals = [
            (
                "LineExtensionAmount",
                "line_net_total",
                Term::BT(106),
                *invoice.line_net_total(),
            ),
            (
                "TaxExclusiveAmount",
                "net_total",
                Term::BT(109),
                *invoice.net_total(),
            ),
            (
                "TaxInclusiveAmount",
                "gross_total",
                Term::BT(112),
                *invoice.gross_total(),
            ),
            (
                "AllowanceTotalAmount",
                "allowances_total",
                Term::BT(107),
                *invoice.allowances_total(),
            ),
            (
                "ChargeTotalAmount",
                "charges_total",
                Term::BT(108),
                *invoice.charges_total(),
            ),
            ("PrepaidAmount", "paid", Term::BT(113), *invoice.paid()),
            (
                "PayableRoundingAmount",
                "rounding",
                Term::BT(114),
                *invoice.rounding(),
            ),
            ("PayableAmount", "due", Term::BT(115), *invoice.due()),
        ];
        if totals.iter().all(|(_, _, _, value)| value.is_none()) {
            return;
        }
        self.structural(Namespace::Cac, "LegalMonetaryTotal", |serializer| {
            for (name, field, term, value) in totals {
                if let Some(value) = value {
                    serializer.leaf_attr(Namespace::Cbc, name, field, term, &attr, &money(value));
                }
            }
        });
    }

    // Serializes the invoice lines (`BG-25`), each a repeatable-group instance.
    fn lines<L: Line>(&mut self, lines: &mut [L], currency: &[(&str, &str)]) {
        for (position, line) in lines.iter_mut().enumerate() {
            let instance = NonZeroUsize::new(position + 1).expect("a positive line index");
            self.repeatable(
                Namespace::Cac,
                "InvoiceLine",
                "lines",
                Term::BG(25),
                instance,
                |serializer| {
                    serializer.invoice_line(line, currency);
                },
            );
        }
    }

    // Serializes one invoice line body.
    fn invoice_line<L: Line>(&mut self, line: &mut L, currency: &[(&str, &str)]) {
        if let Some(id) = line.id() {
            self.leaf(Namespace::Cbc, "ID", "id", Term::BT(126), id.as_ref());
        }
        if let Some(note) = line.note() {
            self.leaf(Namespace::Cbc, "Note", "note", Term::BT(127), note.as_ref());
        }
        if let Some(quantity) = line.quantity() {
            self.leaf_attr(
                Namespace::Cbc,
                "InvoicedQuantity",
                "quantity",
                Term::BT(129),
                &[("unitCode", quantity.unit.code())],
                &plain(quantity.value),
            );
        }
        if let Some(net) = *line.net_amount() {
            self.leaf_attr(
                Namespace::Cbc,
                "LineExtensionAmount",
                "net_amount",
                Term::BT(131),
                currency,
                &money(net),
            );
        }
        if let Some(reference) = line.buyer_accounting_reference() {
            self.leaf(
                Namespace::Cbc,
                "AccountingCost",
                "buyer_accounting_reference",
                Term::BT(133),
                reference.as_ref(),
            );
        }
        if let Some(period) = *line.period() {
            self.line_period(period);
        }
        if let Some(order) = line.order_line_reference() {
            self.structural(Namespace::Cac, "OrderLineReference", |serializer| {
                serializer.leaf(
                    Namespace::Cbc,
                    "LineID",
                    "order_line_reference",
                    Term::BT(132),
                    order.as_ref(),
                );
            });
        }
        if let Some(object) = line.object() {
            self.structural(Namespace::Cac, "DocumentReference", |serializer| {
                let Some(id) = &object.id else {
                    return;
                };
                match &object.scheme {
                    Some(scheme) => serializer.leaf_attr(
                        Namespace::Cbc,
                        "ID",
                        "object",
                        Term::BT(128),
                        &[("schemeID", &scheme.to_string())],
                        id.as_ref(),
                    ),
                    None => {
                        serializer.leaf(Namespace::Cbc, "ID", "object", Term::BT(128), id.as_ref())
                    }
                }
            });
        }
        self.line_adjustments(line.adjustments(), currency);
        if line.item().is_some() || line.vat().is_some() {
            self.item(line);
        }
        if let Some(price) = line.price() {
            self.line_price(price, currency);
        }
    }

    // Serializes the line invoice period (`BG-26`).
    fn line_period(&mut self, period: Period) {
        self.group(
            Namespace::Cac,
            "InvoicePeriod",
            "period",
            Term::BG(26),
            |serializer| {
                if let Some(start) = period.start() {
                    serializer.field_leaf(Namespace::Cbc, "StartDate", "period", &date(start));
                }
                if let Some(end) = period.end() {
                    serializer.field_leaf(Namespace::Cbc, "EndDate", "period", &date(end));
                }
            },
        );
    }

    // Serializes the line-level allowances and charges (`BG-27`/`BG-28`).
    fn line_adjustments(&mut self, adjustments: &[LineAdjustment], currency: &[(&str, &str)]) {
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
                Namespace::Cac,
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
                        serializer.adjustment_amount(amount, currency);
                    }
                },
            );
        }
    }

    // Serializes the item (`BG-31`), rebasing the classified tax category onto the line VAT.
    // The item is borrowed twice, because the classified tax category sits between its head
    // and its attributes in the UBL schema.
    fn item<L: Line>(&mut self, line: &mut L) {
        self.trace.enter(Namespace::Cac.into(), "Item");
        self.trace.push_field("item");
        self.trace.record_context();
        self.write_start(Namespace::Cac, "Item");

        if let Some(item) = line.item() {
            self.item_head(item);
        }

        // The classified tax category is a sibling of the item in the model.
        self.trace.pop_context();
        if let Some(vat) = line.vat() {
            self.classified_tax_category(vat);
        }

        if let Some(item) = line.item() {
            self.item_attributes(item);
        }

        self.write_end(Namespace::Cac, "Item");
        self.trace.leave();
    }

    // Serializes the item fields that precede the classified tax category.
    fn item_head<T: Item>(&mut self, item: &mut T) {
        if let Some(description) = item.description() {
            self.field_leaf(
                Namespace::Cbc,
                "Description",
                "description",
                description.as_ref(),
            );
        }
        if let Some(name) = item.name() {
            self.field_leaf(Namespace::Cbc, "Name", "name", name.as_ref());
        }
        if let Some(id) = item.buyer_id() {
            self.structural(Namespace::Cac, "BuyersItemIdentification", |serializer| {
                serializer.field_leaf(Namespace::Cbc, "ID", "buyer_id", id.as_ref());
            });
        }
        if let Some(id) = item.seller_id() {
            self.structural(Namespace::Cac, "SellersItemIdentification", |serializer| {
                serializer.field_leaf(Namespace::Cbc, "ID", "seller_id", id.as_ref());
            });
        }
        if let Some(standard) = item.standard_id() {
            self.structural(Namespace::Cac, "StandardItemIdentification", |serializer| {
                let Some(id) = &standard.id else {
                    return;
                };
                match &standard.issuer {
                    Some(issuer) => serializer.field_leaf_attr(
                        Namespace::Cbc,
                        "ID",
                        "standard_id",
                        &[("schemeID", &issuer.to_string())],
                        id.as_ref(),
                    ),
                    None => serializer.field_leaf(Namespace::Cbc, "ID", "standard_id", id.as_ref()),
                }
            });
        }
        if let Some(country) = *item.country_of_origin() {
            self.structural(Namespace::Cac, "OriginCountry", |serializer| {
                serializer.field_leaf(
                    Namespace::Cbc,
                    "IdentificationCode",
                    "country_of_origin",
                    country.alpha2(),
                );
            });
        }
        for classification in item.classifications().iter() {
            self.structural(Namespace::Cac, "CommodityClassification", |serializer| {
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
                    Namespace::Cbc,
                    "ItemClassificationCode",
                    "classifications",
                    &borrowed,
                    id.as_ref(),
                );
            });
        }
    }

    // Serializes the item attributes (`BG-32`), which follow the classified tax category.
    fn item_attributes<T: Item>(&mut self, item: &mut T) {
        for attribute in item.attributes().iter() {
            self.structural(Namespace::Cac, "AdditionalItemProperty", |serializer| {
                if let Some(name) = &attribute.name {
                    serializer.field_leaf(Namespace::Cbc, "Name", "attributes", name.as_ref());
                }
                if let Some(value) = &attribute.value {
                    serializer.field_leaf(Namespace::Cbc, "Value", "attributes", value.as_ref());
                }
            });
        }
    }

    // Serializes the line VAT as a UBL classified tax category, mapped to the VAT field.
    fn classified_tax_category(&mut self, vat: &VatTreatment) {
        self.group(
            Namespace::Cac,
            "ClassifiedTaxCategory",
            "vat",
            Term::BG(30),
            |serializer| {
                serializer.field_leaf(Namespace::Cbc, "ID", "vat", &vat.category().to_string());
                serializer.field_leaf(Namespace::Cbc, "Percent", "vat", &plain(vat.rate()));
                serializer.structural(Namespace::Cac, "TaxScheme", |serializer| {
                    serializer.field_leaf(Namespace::Cbc, "ID", "vat", "VAT");
                });
            },
        );
    }

    // Serializes the line price (`BG-29`), every price with its own scale.
    fn line_price(&mut self, price: &Price, currency: &[(&str, &str)]) {
        self.group(
            Namespace::Cac,
            "Price",
            "price",
            Term::BG(29),
            |serializer| {
                if let Some(net) = price.net {
                    serializer.leaf_attr(
                        Namespace::Cbc,
                        "PriceAmount",
                        "net",
                        Term::BT(146),
                        currency,
                        &plain(net),
                    );
                }
                if let Some(base) = price.base_quantity {
                    serializer.field_leaf_attr(
                        Namespace::Cbc,
                        "BaseQuantity",
                        "price",
                        &[("unitCode", base.unit.code())],
                        &plain(base.value),
                    );
                }
                if let Some(discount) = price.discount {
                    serializer.structural(Namespace::Cac, "AllowanceCharge", |serializer| {
                        serializer.field_leaf(Namespace::Cbc, "ChargeIndicator", "price", "false");
                        serializer.field_leaf_attr(
                            Namespace::Cbc,
                            "Amount",
                            "price",
                            currency,
                            &plain(discount),
                        );
                        if let Some(gross) = price.gross {
                            serializer.field_leaf_attr(
                                Namespace::Cbc,
                                "BaseAmount",
                                "price",
                                currency,
                                &plain(gross),
                            );
                        }
                    });
                }
            },
        );
    }

    // Serializes an allowance or charge VAT category, mapped to the enclosing adjustment.
    fn tax_category(&mut self, vat: &VatTreatment) {
        self.nested(Namespace::Cac, "TaxCategory", |serializer| {
            serializer.derived(Namespace::Cbc, "ID", &[], &vat.category().to_string());
            serializer.derived(Namespace::Cbc, "Percent", &[], &plain(vat.rate()));
            serializer.nested(Namespace::Cac, "TaxScheme", |serializer| {
                serializer.derived(Namespace::Cbc, "ID", &[], "VAT");
            });
        });
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
