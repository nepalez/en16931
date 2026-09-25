//! The CII parsing walk, generic over the interfaces of the semantic model.

use super::{Namespace, Token};
use crate::Format;
use crate::prelude::{
    CountryCode, Currency, Date, Decimal, EmailAddress, Month, NonZeroUsize, Url,
};
use crate::{
    Adjustment, AdjustmentAmount, AdjustmentReason, Amount, Buyer, Cii, Contact, CreditTransfer,
    Delivery, DirectDebit, DocumentBuilder, ElectronicAddress, Error, Invoice, InvoiceKind,
    InvoiceReference, Item, ItemAttribute, ItemClassification, ItemReference, LegalEntity, Line,
    LineAdjustment, LocationReference, NonEmptyString, Note, ObjectReference, OperationalEntity,
    Parser, Payee, PaymentCard, PaymentDetails, PaymentInstructions, Percentage, Period,
    PostalAddress, Quantity, QuantityUnit, Seller, SupportingDocument, TaxRepresentative,
    VatBreakdown, VatCategory, VatExemptionReason, VatPoint, VatTreatment,
};

/// Reads the whole document from the parser into a builder, from the root element down,
/// filling the dictionary in the same pass.
///
/// An implementation of `Deserializable<Cii, N>` for a concrete invoice type calls this walk.
/// The walk builds every group from its default and fills it through the model interfaces,
/// so the invoice type and each of its groups must implement `Default`.
#[allow(clippy::type_complexity)]
pub fn deserialize<I, N>(parser: &mut Parser<Cii, N>) -> Result<DocumentBuilder<I>, Error>
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
    let kind = InvoiceKind::default();
    parser.enter_structural(Cii::root_namespace(kind), Cii::root_element(kind))?;

    let (profile, business_process) = parser.exchanged_document_context()?;
    let mut invoice = I::default();
    parser.exchanged_document(&mut invoice)?;

    let mut exemptions = ExemptionMap::new();
    if parser.is_open(Namespace::Rsm, "SupplyChainTradeTransaction") {
        parser.enter_structural(Namespace::Rsm, "SupplyChainTradeTransaction")?;
        while parser.is_open(Namespace::Ram, "IncludedSupplyChainTradeLineItem") {
            let instance = index(invoice.lines().len());
            let line = parser.parse_line(instance)?;
            invoice.lines().push(line);
        }
        parser.header_trade_agreement(&mut invoice)?;
        parser.header_trade_delivery(&mut invoice)?;
        exemptions = parser.header_trade_settlement(&mut invoice)?;
        parser.leave_structural()?;
    }

    parser.leave_structural()?;
    exemptions.apply(&mut invoice);

    Ok(DocumentBuilder {
        invoice,
        profile,
        business_process,
    })
}

// ---- parser --------------------------------------------------------------

impl<N: crate::Namespace + From<Namespace>> Parser<Cii, N> {
    // Parses the document context: the profile (`BT-24`) and business process (`BT-23`).
    fn exchanged_document_context(
        &mut self,
    ) -> Result<(crate::Profile, Option<crate::BusinessProcess>), Error> {
        self.enter_structural(Namespace::Rsm, "ExchangedDocumentContext")?;
        let business_process = if self.is_open(
            Namespace::Ram,
            "BusinessProcessSpecifiedDocumentContextParameter",
        ) {
            self.enter_structural(
                Namespace::Ram,
                "BusinessProcessSpecifiedDocumentContextParameter",
            )?;
            let value = self.rooted(Namespace::Ram, "ID")?.parse()?;
            self.leave_structural()?;
            Some(value)
        } else {
            None
        };
        self.enter_structural(Namespace::Ram, "GuidelineSpecifiedDocumentContextParameter")?;
        let profile = self.rooted(Namespace::Ram, "ID")?.parse()?;
        self.leave_structural()?;
        self.leave_structural()?;
        Ok((profile, business_process))
    }

    // Parses the exchanged document header into the invoice.
    fn exchanged_document<I: Invoice>(&mut self, invoice: &mut I) -> Result<(), Error> {
        self.enter_structural(Namespace::Rsm, "ExchangedDocument")?;
        *invoice.number() = self.optional_leaf(Namespace::Ram, "ID", "number")?;
        *invoice.type_code() = self
            .leaf(Namespace::Ram, "TypeCode", "type_code")?
            .parse()?;
        // CII states no kind of its own, so the type code lists of `BR-CL-01` decide it.
        *invoice.kind() = InvoiceKind::from(*invoice.type_code());
        *invoice.issue_date() = self.optional_datetime("IssueDateTime", "issue_date")?;
        while self.is_open(Namespace::Ram, "IncludedNote") {
            let instance = index(invoice.notes().len());
            self.enter_repeatable(Namespace::Ram, "IncludedNote", "notes", instance)?;
            let text = self
                .optional_derived(Namespace::Ram, "Content")?
                .map(|(_, text)| text.parse())
                .transpose()?;
            let subject_code = if self.is_open(Namespace::Ram, "SubjectCode") {
                Some(self.derived(Namespace::Ram, "SubjectCode")?.1.parse()?)
            } else {
                None
            };
            self.leave_repeatable()?;
            invoice.notes().push(Note { subject_code, text });
        }
        self.leave_structural()?;
        Ok(())
    }

    // Parses one invoice line.
    fn parse_line<L: Line + Default>(&mut self, instance: NonZeroUsize) -> Result<L, Error>
    where
        L::Item: Default,
    {
        self.enter_repeatable(
            Namespace::Ram,
            "IncludedSupplyChainTradeLineItem",
            "lines",
            instance,
        )?;

        let mut line = L::default();
        if self.is_open(Namespace::Ram, "AssociatedDocumentLineDocument") {
            self.enter_structural(Namespace::Ram, "AssociatedDocumentLineDocument")?;
            *line.id() = self.optional_leaf(Namespace::Ram, "LineID", "id")?;
            if self.is_open(Namespace::Ram, "IncludedNote") {
                self.enter_group(Namespace::Ram, "IncludedNote", "note")?;
                *line.note() = self
                    .optional_derived(Namespace::Ram, "Content")?
                    .map(|(_, text)| text.parse())
                    .transpose()?;
                self.leave_group()?;
            }
            self.leave_structural()?;
        }

        if self.is_open(Namespace::Ram, "SpecifiedTradeProduct") {
            *line.item() = Some(self.parse_product()?);
        }
        if self.is_open(Namespace::Ram, "SpecifiedLineTradeAgreement") {
            self.parse_agreement(&mut line)?;
        }

        if self.is_open(Namespace::Ram, "SpecifiedLineTradeDelivery") {
            self.enter_structural(Namespace::Ram, "SpecifiedLineTradeDelivery")?;
            *line.quantity() = self
                .optional_leaf_attr(Namespace::Ram, "BilledQuantity", "quantity")?
                .map(|(attributes, text)| parse_quantity(&attributes, &text))
                .transpose()?;
            self.leave_structural()?;
        }

        if self.is_open(Namespace::Ram, "SpecifiedLineTradeSettlement") {
            self.parse_line_settlement(&mut line)?;
        }

        self.leave_repeatable()?;
        Ok(line)
    }

    // Parses the line product into an item.
    fn parse_product<T: Item + Default>(&mut self) -> Result<T, Error> {
        self.enter_group(Namespace::Ram, "SpecifiedTradeProduct", "item")?;

        let mut item = T::default();
        *item.standard_id() = self
            .optional_leaf_attr(Namespace::Ram, "GlobalID", "standard_id")?
            .map(|(attributes, id)| {
                Ok::<_, Error>(ItemReference {
                    id: Some(id.parse()?),
                    issuer: attr(&attributes, "schemeID")
                        .map(|value| value.parse())
                        .transpose()?,
                })
            })
            .transpose()?;
        *item.seller_id() = self.optional_leaf(Namespace::Ram, "SellerAssignedID", "seller_id")?;
        *item.buyer_id() = self.optional_leaf(Namespace::Ram, "BuyerAssignedID", "buyer_id")?;
        *item.name() = self.optional_leaf(Namespace::Ram, "Name", "name")?;
        *item.description() = self.optional_leaf(Namespace::Ram, "Description", "description")?;
        while self.is_open(Namespace::Ram, "ApplicableProductCharacteristic") {
            self.enter_structural(Namespace::Ram, "ApplicableProductCharacteristic")?;
            let name = self.optional_leaf(Namespace::Ram, "Description", "attributes")?;
            let value = self.optional_leaf(Namespace::Ram, "Value", "attributes")?;
            self.leave_structural()?;
            item.attributes().push(ItemAttribute { name, value });
        }
        while self.is_open(Namespace::Ram, "DesignatedProductClassification") {
            self.enter_structural(Namespace::Ram, "DesignatedProductClassification")?;
            let mut classification = ItemClassification::default();
            if let Some((class_attrs, id)) =
                self.optional_leaf_attr(Namespace::Ram, "ClassCode", "classifications")?
            {
                classification.id = Some(id.parse()?);
                classification.scheme = attr(&class_attrs, "listID")
                    .map(|value| value.parse())
                    .transpose()?;
                classification.version = match attr(&class_attrs, "listVersionID") {
                    Some(value) => Some(value.parse()?),
                    None => None,
                };
            }
            self.leave_structural()?;
            item.classifications().push(classification);
        }
        if self.is_open(Namespace::Ram, "OriginTradeCountry") {
            self.enter_structural(Namespace::Ram, "OriginTradeCountry")?;
            let code = self.optional_text(Namespace::Ram, "ID", "country_of_origin")?;
            self.leave_structural()?;
            *item.country_of_origin() = code.map(|code| parse_country(&code)).transpose()?;
        }

        self.leave_group()?;
        Ok(item)
    }

    // Parses the line trade agreement into the line: the order line reference and the price.
    // The base quantity of the net price group applies only without the gross price group.
    fn parse_agreement<L: Line>(&mut self, line: &mut L) -> Result<(), Error> {
        self.enter_structural(Namespace::Ram, "SpecifiedLineTradeAgreement")?;
        if self.is_open(Namespace::Ram, "BuyerOrderReferencedDocument") {
            self.enter_structural(Namespace::Ram, "BuyerOrderReferencedDocument")?;
            *line.order_line_reference() =
                self.optional_leaf(Namespace::Ram, "LineID", "order_line_reference")?;
            self.leave_structural()?;
        }

        let mut has_gross = false;
        if self.is_open(Namespace::Ram, "GrossPriceProductTradePrice") {
            has_gross = true;
            self.enter_group(Namespace::Ram, "GrossPriceProductTradePrice", "price")?;
            let price = line.price().get_or_insert_with(Default::default);
            price.gross = self
                .optional_derived(Namespace::Ram, "ChargeAmount")?
                .map(|(_, text)| parse_decimal(&text))
                .transpose()?;
            price.base_quantity = self
                .optional_derived(Namespace::Ram, "BasisQuantity")?
                .map(|(attributes, text)| parse_quantity(&attributes, &text))
                .transpose()?;
            if self.is_open(Namespace::Ram, "AppliedTradeAllowanceCharge") {
                self.enter_nested(Namespace::Ram, "AppliedTradeAllowanceCharge")?;
                self.optional_indicator()?;
                price.discount = self
                    .optional_derived(Namespace::Ram, "ActualAmount")?
                    .map(|(_, text)| parse_decimal(&text))
                    .transpose()?;
                self.leave_nested()?;
            }
            self.leave_group()?;
        }

        if self.is_open(Namespace::Ram, "NetPriceProductTradePrice") {
            self.enter_group(Namespace::Ram, "NetPriceProductTradePrice", "price")?;
            let net = self
                .optional_text(Namespace::Ram, "ChargeAmount", "net")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            let base_quantity = self
                .optional_derived(Namespace::Ram, "BasisQuantity")?
                .map(|(attributes, text)| parse_quantity(&attributes, &text))
                .transpose()?;
            self.leave_group()?;
            let price = line.price().get_or_insert_with(Default::default);
            price.net = net;
            if !has_gross {
                price.base_quantity = base_quantity;
            }
        }

        self.leave_structural()?;
        Ok(())
    }

    // Parses the line trade settlement into the line.
    fn parse_line_settlement<L: Line>(&mut self, line: &mut L) -> Result<(), Error> {
        self.enter_structural(Namespace::Ram, "SpecifiedLineTradeSettlement")?;
        if self.is_open(Namespace::Ram, "ApplicableTradeTax") {
            *line.vat() = Some(self.parse_line_tax()?);
        }
        if self.is_open(Namespace::Ram, "BillingSpecifiedPeriod") {
            *line.period() = Some(self.billing_period("period")?);
        }
        while self.is_open(Namespace::Ram, "SpecifiedTradeAllowanceCharge") {
            let instance = index(line.adjustments().len());
            let adjustment = self.parse_line_adjustment(instance)?;
            line.adjustments().push(adjustment);
        }
        if self.is_open(
            Namespace::Ram,
            "SpecifiedTradeSettlementLineMonetarySummation",
        ) {
            self.enter_structural(
                Namespace::Ram,
                "SpecifiedTradeSettlementLineMonetarySummation",
            )?;
            *line.net_amount() = self
                .optional_text(Namespace::Ram, "LineTotalAmount", "net_amount")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            self.leave_structural()?;
        }
        if self.is_open(Namespace::Ram, "AdditionalReferencedDocument") {
            self.enter_group(Namespace::Ram, "AdditionalReferencedDocument", "object")?;
            let id = self
                .optional_derived(Namespace::Ram, "IssuerAssignedID")?
                .map(|(_, text)| text.parse())
                .transpose()?;
            self.optional_derived(Namespace::Ram, "TypeCode")?;
            let scheme = self
                .optional_derived(Namespace::Ram, "ReferenceTypeCode")?
                .map(|(_, text)| text.parse())
                .transpose()?;
            self.leave_group()?;
            *line.object() = Some(ObjectReference { id, scheme });
        }
        *line.buyer_accounting_reference() = self.optional_accounting_account()?;
        self.leave_structural()?;
        Ok(())
    }

    // Parses the receivable accounting account (`BT-19`/`BT-133`), when present.
    fn optional_accounting_account(&mut self) -> Result<Option<NonEmptyString>, Error> {
        if !self.is_open(Namespace::Ram, "ReceivableSpecifiedTradeAccountingAccount") {
            return Ok(None);
        }
        self.enter_group(
            Namespace::Ram,
            "ReceivableSpecifiedTradeAccountingAccount",
            "buyer_accounting_reference",
        )?;
        let reference = self
            .optional_derived(Namespace::Ram, "ID")?
            .map(|(_, text)| text.parse())
            .transpose()?;
        self.leave_group()?;
        Ok(reference)
    }

    fn parse_line_tax(&mut self) -> Result<VatTreatment, Error> {
        self.enter_group(Namespace::Ram, "ApplicableTradeTax", "vat")?;
        self.optional_derived(Namespace::Ram, "TypeCode")?;
        let category: VatCategory = self.derived(Namespace::Ram, "CategoryCode")?.1.parse()?;
        let rate = self
            .derived(Namespace::Ram, "RateApplicablePercent")?
            .1
            .parse()?;
        self.leave_group()?;
        Ok(VatTreatment::from_category(category, rate))
    }

    fn parse_line_adjustment(&mut self, instance: NonZeroUsize) -> Result<LineAdjustment, Error> {
        self.enter_repeatable(
            Namespace::Ram,
            "SpecifiedTradeAllowanceCharge",
            "adjustments",
            instance,
        )?;
        let charge = self.optional_indicator()?;
        let amount = self.parse_adjustment_amount()?;
        let reason = self.parse_reason(charge)?;
        self.leave_repeatable()?;
        Ok(LineAdjustment { amount, reason })
    }

    // Parses the header trade agreement into the invoice, when present.
    fn header_trade_agreement<I: Invoice>(&mut self, invoice: &mut I) -> Result<(), Error>
    where
        I::Seller: Default,
        <I::Seller as Seller>::Contact: Default,
        I::Buyer: Default,
        <I::Buyer as Buyer>::Contact: Default,
        I::TaxRepresentative: Default,
    {
        if !self.is_open(Namespace::Ram, "ApplicableHeaderTradeAgreement") {
            return Ok(());
        }
        self.enter_structural(Namespace::Ram, "ApplicableHeaderTradeAgreement")?;
        *invoice.buyer_reference() =
            self.optional_leaf(Namespace::Ram, "BuyerReference", "buyer_reference")?;
        if self.is_open(Namespace::Ram, "SellerTradeParty") {
            *invoice.seller() = Some(self.parse_seller()?);
        }
        if self.is_open(Namespace::Ram, "BuyerTradeParty") {
            *invoice.buyer() = Some(self.parse_buyer()?);
        }
        if self.is_open(Namespace::Ram, "SellerTaxRepresentativeTradeParty") {
            *invoice.tax_representative() = Some(self.parse_tax_representative()?);
        }
        *invoice.sales_order_reference() = self.optional_reference(
            Namespace::Ram,
            "SellerOrderReferencedDocument",
            "sales_order_reference",
        )?;
        *invoice.purchase_order_reference() = self.optional_reference(
            Namespace::Ram,
            "BuyerOrderReferencedDocument",
            "purchase_order_reference",
        )?;
        *invoice.contract_reference() = self.optional_reference(
            Namespace::Ram,
            "ContractReferencedDocument",
            "contract_reference",
        )?;
        while self.is_open(Namespace::Ram, "AdditionalReferencedDocument") {
            self.parse_additional_document(invoice)?;
        }
        if self.is_open(Namespace::Ram, "SpecifiedProcuringProject") {
            self.enter_group(
                Namespace::Ram,
                "SpecifiedProcuringProject",
                "project_reference",
            )?;
            *invoice.project_reference() = self
                .optional_derived(Namespace::Ram, "ID")?
                .map(|(_, text)| text.parse())
                .transpose()?;
            self.optional_derived(Namespace::Ram, "Name")?;
            self.leave_group()?;
        }
        self.leave_structural()?;
        Ok(())
    }

    // Parses one additional referenced document into the invoice, by its fixed type code.
    fn parse_additional_document<I: Invoice>(&mut self, invoice: &mut I) -> Result<(), Error> {
        match self.additional_kind() {
            AdditionalKind::Object => {
                self.enter_group(Namespace::Ram, "AdditionalReferencedDocument", "object")?;
                let id = self.optional_issuer_assigned_id()?;
                self.optional_derived(Namespace::Ram, "TypeCode")?;
                let scheme = self
                    .optional_derived(Namespace::Ram, "ReferenceTypeCode")?
                    .map(|(_, text)| text.parse())
                    .transpose()?;
                self.leave_group()?;
                *invoice.object() = Some(ObjectReference { id, scheme });
            }
            AdditionalKind::Tender => {
                self.enter_group(
                    Namespace::Ram,
                    "AdditionalReferencedDocument",
                    "tender_or_lot_reference",
                )?;
                *invoice.tender_or_lot_reference() = self.optional_issuer_assigned_id()?;
                self.optional_derived(Namespace::Ram, "TypeCode")?;
                self.leave_group()?;
            }
            AdditionalKind::Supporting => {
                let instance = index(invoice.supporting_documents().len());
                self.enter_repeatable(
                    Namespace::Ram,
                    "AdditionalReferencedDocument",
                    "supporting_documents",
                    instance,
                )?;
                let reference = self.optional_issuer_assigned_id()?;
                let external_location = self
                    .optional_text(Namespace::Ram, "URIID", "external_location")?
                    .map(|uri| parse_url(&uri))
                    .transpose()?;
                self.optional_derived(Namespace::Ram, "TypeCode")?;
                let description = self.optional_leaf(Namespace::Ram, "Name", "description")?;
                self.leave_repeatable()?;
                invoice.supporting_documents().push(SupportingDocument {
                    reference,
                    description,
                    external_location,
                    attachment: None,
                });
            }
        }
        Ok(())
    }

    // Reads the issuer-assigned identifier of a referenced document, when present.
    fn optional_issuer_assigned_id(&mut self) -> Result<Option<NonEmptyString>, Error> {
        self.optional_derived(Namespace::Ram, "IssuerAssignedID")?
            .map(|(_, text)| text.parse())
            .transpose()
    }

    // Classifies the current additional referenced document by its fixed type code.
    fn additional_kind(&self) -> AdditionalKind {
        match self.peek_child_text("TypeCode").as_deref() {
            Some("130") => AdditionalKind::Object,
            Some("50") => AdditionalKind::Tender,
            _ => AdditionalKind::Supporting,
        }
    }

    fn parse_seller<S: Seller + Default>(&mut self) -> Result<S, Error>
    where
        S::Contact: Default,
    {
        self.enter_group(Namespace::Ram, "SellerTradeParty", "seller")?;
        let mut seller = S::default();
        *seller.identifiers() = self.parse_identifiers("identifiers")?;
        *seller.name() = self.optional_leaf(Namespace::Ram, "Name", "name")?;
        *seller.additional_legal_information() = self.optional_leaf(
            Namespace::Ram,
            "Description",
            "additional_legal_information",
        )?;
        let (legal_entity, trading_name) = self.parse_legal_organization()?;
        *seller.legal_entity() = legal_entity;
        *seller.trading_name() = trading_name;
        if self.is_open(Namespace::Ram, "DefinedTradeContact") {
            *seller.contact() = Some(self.parse_contact()?);
        }
        *seller.address() = self.optional_address()?;
        *seller.electronic_address() = self.optional_electronic_address()?;
        while self.is_open(Namespace::Ram, "SpecifiedTaxRegistration") {
            match self.parse_tax_registration()? {
                Some(TaxRegistration::Vat(value)) => *seller.vat() = Some(value),
                Some(TaxRegistration::Other(value)) => *seller.tax_registration() = Some(value),
                None => {}
            }
        }
        self.leave_group()?;
        Ok(seller)
    }

    fn parse_buyer<B: Buyer + Default>(&mut self) -> Result<B, Error>
    where
        B::Contact: Default,
    {
        self.enter_group(Namespace::Ram, "BuyerTradeParty", "buyer")?;
        let mut buyer = B::default();
        *buyer.identifiers() = self.parse_identifiers("identifiers")?;
        *buyer.name() = self.optional_leaf(Namespace::Ram, "Name", "name")?;
        let (legal_entity, trading_name) = self.parse_legal_organization()?;
        *buyer.legal_entity() = legal_entity;
        *buyer.trading_name() = trading_name;
        if self.is_open(Namespace::Ram, "DefinedTradeContact") {
            *buyer.contact() = Some(self.parse_contact()?);
        }
        *buyer.address() = self.optional_address()?;
        *buyer.electronic_address() = self.optional_electronic_address()?;
        while self.is_open(Namespace::Ram, "SpecifiedTaxRegistration") {
            if let Some(TaxRegistration::Vat(value)) = self.parse_tax_registration()? {
                *buyer.vat() = Some(value);
            }
        }
        self.leave_group()?;
        Ok(buyer)
    }

    fn parse_tax_representative<T: TaxRepresentative + Default>(&mut self) -> Result<T, Error> {
        self.enter_group(
            Namespace::Ram,
            "SellerTaxRepresentativeTradeParty",
            "tax_representative",
        )?;
        let mut representative = T::default();
        *representative.name() = self.optional_leaf(Namespace::Ram, "Name", "name")?;
        *representative.address() = self.optional_address()?;
        if self.is_open(Namespace::Ram, "SpecifiedTaxRegistration") {
            *representative.vat() = match self.parse_tax_registration()? {
                Some(TaxRegistration::Vat(value)) => Some(value),
                Some(TaxRegistration::Other(_)) => {
                    return Err(bad("a tax representative without a VAT id"));
                }
                None => None,
            };
        }
        self.leave_group()?;
        Ok(representative)
    }

    fn parse_identifiers(&mut self, field: &'static str) -> Result<Vec<OperationalEntity>, Error> {
        let mut identifiers = Vec::new();
        loop {
            if self.is_open(Namespace::Ram, "ID") {
                let id = self.leaf(Namespace::Ram, "ID", field)?.parse()?;
                identifiers.push(OperationalEntity {
                    id: Some(id),
                    issuer: None,
                });
            } else if self.is_open(Namespace::Ram, "GlobalID") {
                let (attributes, id) = self.leaf_attr(Namespace::Ram, "GlobalID", field)?;
                let issuer = attr(&attributes, "schemeID")
                    .map(|value| value.parse())
                    .transpose()?;
                identifiers.push(OperationalEntity {
                    id: Some(id.parse()?),
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
        if !self.is_open(Namespace::Ram, "SpecifiedLegalOrganization") {
            return Ok((None, None));
        }
        self.enter_structural(Namespace::Ram, "SpecifiedLegalOrganization")?;
        let legal_entity = self.optional_legal_entity()?;
        let trading_name =
            self.optional_leaf(Namespace::Ram, "TradingBusinessName", "trading_name")?;
        self.leave_structural()?;
        Ok((legal_entity, trading_name))
    }

    // Reads the legal registration identifier of a party, when present.
    fn optional_legal_entity(&mut self) -> Result<Option<LegalEntity>, Error> {
        self.optional_leaf_attr(Namespace::Ram, "ID", "legal_entity")?
            .map(|(attributes, id)| {
                Ok::<_, Error>(LegalEntity {
                    id: Some(id.parse()?),
                    issuer: attr(&attributes, "schemeID")
                        .map(|value| value.parse())
                        .transpose()?,
                })
            })
            .transpose()
    }

    fn parse_contact<C: Contact + Default>(&mut self) -> Result<C, Error> {
        self.enter_group(Namespace::Ram, "DefinedTradeContact", "contact")?;
        let mut contact = C::default();
        *contact.name() = self.optional_leaf(Namespace::Ram, "PersonName", "name")?;
        if self.is_open(Namespace::Ram, "TelephoneUniversalCommunication") {
            self.enter_structural(Namespace::Ram, "TelephoneUniversalCommunication")?;
            *contact.telephone() =
                self.optional_leaf(Namespace::Ram, "CompleteNumber", "telephone")?;
            self.leave_structural()?;
        }
        if self.is_open(Namespace::Ram, "EmailURIUniversalCommunication") {
            self.enter_structural(Namespace::Ram, "EmailURIUniversalCommunication")?;
            *contact.email() = self
                .optional_text(Namespace::Ram, "URIID", "email")?
                .map(|value| parse_email(&value))
                .transpose()?;
            self.leave_structural()?;
        }
        self.leave_group()?;
        Ok(contact)
    }

    fn optional_address(&mut self) -> Result<Option<PostalAddress>, Error> {
        if !self.is_open(Namespace::Ram, "PostalTradeAddress") {
            return Ok(None);
        }
        self.enter_group(Namespace::Ram, "PostalTradeAddress", "address")?;
        let postal_code = self.optional_leaf(Namespace::Ram, "PostcodeCode", "postal_code")?;
        let line1 = self.optional_leaf(Namespace::Ram, "LineOne", "line1")?;
        let line2 = self.optional_leaf(Namespace::Ram, "LineTwo", "line2")?;
        let line3 = self.optional_leaf(Namespace::Ram, "LineThree", "line3")?;
        let city = self.optional_leaf(Namespace::Ram, "CityName", "city")?;
        let country = self
            .optional_text(Namespace::Ram, "CountryID", "country")?
            .map(|code| parse_country(&code))
            .transpose()?;
        let country_subdivision = self.optional_leaf(
            Namespace::Ram,
            "CountrySubDivisionName",
            "country_subdivision",
        )?;
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

    fn optional_electronic_address(&mut self) -> Result<Option<ElectronicAddress>, Error> {
        if !self.is_open(Namespace::Ram, "URIUniversalCommunication") {
            return Ok(None);
        }
        self.enter_structural(Namespace::Ram, "URIUniversalCommunication")?;
        let mut address = ElectronicAddress::default();
        if let Some((attributes, id)) =
            self.optional_leaf_attr(Namespace::Ram, "URIID", "electronic_address")?
        {
            address.id = Some(id.parse()?);
            address.scheme = attr(&attributes, "schemeID")
                .map(|value| value.parse())
                .transpose()?;
        }
        self.leave_structural()?;
        Ok(Some(address))
    }

    fn parse_tax_registration(&mut self) -> Result<Option<TaxRegistration>, Error> {
        // The serializer chose the field from the scheme, so peek it first.
        let scheme = self.peek_scheme_id();
        let field: &'static str = if scheme == "FC" {
            "tax_registration"
        } else {
            "vat"
        };
        self.enter_group(Namespace::Ram, "SpecifiedTaxRegistration", field)?;
        let id = self
            .optional_leaf_attr(Namespace::Ram, "ID", field)?
            .map(|(_, id)| id);
        self.leave_group()?;
        let Some(id) = id else {
            return Ok(None);
        };
        if scheme == "FC" {
            Ok(Some(TaxRegistration::Other(id.parse()?)))
        } else {
            Ok(Some(TaxRegistration::Vat(id.parse()?)))
        }
    }

    // Parses the header trade delivery into the invoice, when present.
    fn header_trade_delivery<I: Invoice>(&mut self, invoice: &mut I) -> Result<(), Error>
    where
        I::Delivery: Default,
    {
        if !self.is_open(Namespace::Ram, "ApplicableHeaderTradeDelivery") {
            return Ok(());
        }
        self.enter_structural(Namespace::Ram, "ApplicableHeaderTradeDelivery")?;
        if self.is_open(Namespace::Ram, "ShipToTradeParty") {
            let delivery = invoice.delivery().get_or_insert_with(Default::default);
            self.parse_ship_to(delivery)?;
        }
        if self.is_open(Namespace::Ram, "ActualDeliverySupplyChainEvent") {
            self.enter_structural(Namespace::Ram, "ActualDeliverySupplyChainEvent")?;
            let date = self.optional_datetime("OccurrenceDateTime", "date")?;
            self.leave_structural()?;
            if let Some(date) = date {
                let delivery = invoice.delivery().get_or_insert_with(Default::default);
                *delivery.date() = Some(date);
            }
        }
        *invoice.despatch_advice_reference() = self.optional_reference(
            Namespace::Ram,
            "DespatchAdviceReferencedDocument",
            "despatch_advice_reference",
        )?;
        *invoice.receiving_advice_reference() = self.optional_reference(
            Namespace::Ram,
            "ReceivingAdviceReferencedDocument",
            "receiving_advice_reference",
        )?;
        self.leave_structural()?;
        Ok(())
    }

    fn parse_ship_to<D: Delivery>(&mut self, delivery: &mut D) -> Result<(), Error> {
        self.enter_group(Namespace::Ram, "ShipToTradeParty", "delivery")?;
        *delivery.location() = self
            .optional_leaf_attr(Namespace::Ram, "ID", "location")?
            .map(|(attributes, id)| {
                Ok::<_, Error>(LocationReference {
                    id: Some(id.parse()?),
                    issuer: attr(&attributes, "schemeID")
                        .map(|value| value.parse())
                        .transpose()?,
                })
            })
            .transpose()?;
        *delivery.name() = self.optional_leaf(Namespace::Ram, "Name", "name")?;
        *delivery.address() = self.optional_address()?;
        self.leave_group()?;
        Ok(())
    }

    // Parses the header trade settlement into the invoice, when present,
    // returning the exemption reasons collected from the VAT breakdown.
    fn header_trade_settlement<I: Invoice>(
        &mut self,
        invoice: &mut I,
    ) -> Result<ExemptionMap, Error>
    where
        I::Payee: Default,
    {
        if !self.is_open(Namespace::Ram, "ApplicableHeaderTradeSettlement") {
            return Ok(ExemptionMap::new());
        }
        self.enter_structural(Namespace::Ram, "ApplicableHeaderTradeSettlement")?;
        let creditor =
            self.optional_leaf(Namespace::Ram, "CreditorReferenceID", "creditor_identifier")?;
        let remittance =
            self.optional_leaf(Namespace::Ram, "PaymentReference", "remittance_information")?;
        let accounting_currency = if self.is_open(Namespace::Ram, "TaxCurrencyCode") {
            Some(parse_currency(&self.leaf(
                Namespace::Ram,
                "TaxCurrencyCode",
                "vat_accounting_total",
            )?)?)
        } else {
            None
        };
        *invoice.currency() = self
            .optional_text(Namespace::Ram, "InvoiceCurrencyCode", "currency")?
            .map(|code| parse_currency(&code))
            .transpose()?;
        if self.is_open(Namespace::Ram, "PayeeTradeParty") {
            *invoice.payee() = Some(self.parse_payee()?);
        }
        if self.is_open(Namespace::Ram, "SpecifiedTradeSettlementPaymentMeans") {
            let payment = invoice.payment().insert(Default::default());
            payment.remittance_information = remittance;
            self.parse_payment_means(payment)?;
        }

        let (exemptions, vat_point) = self.parse_tax_breakdown(invoice)?;
        *invoice.vat_point() = vat_point;
        if self.is_open(Namespace::Ram, "BillingSpecifiedPeriod") {
            *invoice.invoicing_period() = Some(self.billing_period("invoicing_period")?);
        }
        while self.is_open(Namespace::Ram, "SpecifiedTradeAllowanceCharge") {
            let instance = index(invoice.adjustments().len());
            let adjustment = self.parse_adjustment(instance)?;
            invoice.adjustments().push(adjustment);
        }
        let mut mandate = None;
        if self.is_open(Namespace::Ram, "SpecifiedTradePaymentTerms") {
            let (terms, due, reference) = self.parse_payment_terms()?;
            *invoice.payment_terms() = terms;
            *invoice.payment_due_date() = due;
            mandate = reference;
        }
        self.parse_monetary_summation(invoice)?;
        // The summation carries no accounting-currency VAT total (`BT-111`) the parser reads.
        let accounting_value: Option<Decimal> = None;
        while self.is_open(Namespace::Ram, "InvoiceReferencedDocument") {
            let instance = index(invoice.preceding_invoices().len());
            let reference = self.parse_preceding_invoice(instance)?;
            invoice.preceding_invoices().push(reference);
        }
        *invoice.buyer_accounting_reference() = self.optional_accounting_account()?;
        self.leave_structural()?;

        // Inject the direct-debit creditor and mandate collected across the settlement.
        if let Some(payment) = invoice.payment() {
            if let Some(PaymentDetails::DirectDebit(debit)) = &mut payment.details {
                debit.creditor_identifier = creditor;
                debit.mandate_reference = mandate;
            }
        }
        *invoice.vat_accounting_total() = match (accounting_currency, accounting_value) {
            (Some(currency), Some(value)) => Some(Amount { value, currency }),
            _ => None,
        };

        Ok(exemptions)
    }

    fn parse_payee<P: Payee + Default>(&mut self) -> Result<P, Error> {
        self.enter_group(Namespace::Ram, "PayeeTradeParty", "payee")?;
        let mut payee = P::default();
        *payee.identifiers() = self.parse_identifiers("identifiers")?;
        *payee.name() = self.optional_leaf(Namespace::Ram, "Name", "name")?;
        if self.is_open(Namespace::Ram, "SpecifiedLegalOrganization") {
            self.enter_structural(Namespace::Ram, "SpecifiedLegalOrganization")?;
            *payee.legal_entity() = self.optional_legal_entity()?;
            self.leave_structural()?;
        }
        self.leave_group()?;
        Ok(payee)
    }

    // Parses the payment means into the payment instructions.
    fn parse_payment_means(&mut self, payment: &mut PaymentInstructions) -> Result<(), Error> {
        self.enter_group(
            Namespace::Ram,
            "SpecifiedTradeSettlementPaymentMeans",
            "payment",
        )?;
        payment.means = self
            .optional_derived(Namespace::Ram, "TypeCode")?
            .map(|(_, text)| text.parse())
            .transpose()?;
        payment.means_text = if self.is_open(Namespace::Ram, "Information") {
            Some(self.derived(Namespace::Ram, "Information")?.1.parse()?)
        } else {
            None
        };
        payment.details = if self.is_open(Namespace::Ram, "ApplicableTradeSettlementFinancialCard")
        {
            self.enter_nested(Namespace::Ram, "ApplicableTradeSettlementFinancialCard")?;
            let primary_account_number = self
                .optional_derived(Namespace::Ram, "ID")?
                .map(|(_, text)| text.parse())
                .transpose()?;
            let holder_name = if self.is_open(Namespace::Ram, "CardholderName") {
                Some(self.derived(Namespace::Ram, "CardholderName")?.1.parse()?)
            } else {
                None
            };
            self.leave_nested()?;
            Some(PaymentDetails::Card(PaymentCard {
                primary_account_number,
                holder_name,
            }))
        } else if self.is_open(Namespace::Ram, "PayerPartyDebtorFinancialAccount") {
            self.enter_nested(Namespace::Ram, "PayerPartyDebtorFinancialAccount")?;
            let debited_account = self
                .optional_derived(Namespace::Ram, "IBANID")?
                .map(|(_, text)| text.parse())
                .transpose()?;
            self.leave_nested()?;
            Some(PaymentDetails::DirectDebit(DirectDebit {
                mandate_reference: None,
                creditor_identifier: None,
                debited_account,
            }))
        } else if self.is_open(Namespace::Ram, "PayeePartyCreditorFinancialAccount") {
            let mut transfers = Vec::new();
            while self.is_open(Namespace::Ram, "PayeePartyCreditorFinancialAccount") {
                self.enter_nested(Namespace::Ram, "PayeePartyCreditorFinancialAccount")?;
                let account = self
                    .optional_derived(Namespace::Ram, "IBANID")?
                    .map(|(_, text)| text.parse())
                    .transpose()?;
                let account_name = if self.is_open(Namespace::Ram, "AccountName") {
                    Some(self.derived(Namespace::Ram, "AccountName")?.1.parse()?)
                } else {
                    None
                };
                self.leave_nested()?;
                let provider =
                    if self.is_open(Namespace::Ram, "PayeeSpecifiedCreditorFinancialInstitution") {
                        self.enter_nested(
                            Namespace::Ram,
                            "PayeeSpecifiedCreditorFinancialInstitution",
                        )?;
                        let provider = self
                            .optional_derived(Namespace::Ram, "BICID")?
                            .map(|(_, text)| text.parse())
                            .transpose()?;
                        self.leave_nested()?;
                        provider
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
        Ok(())
    }

    // Parses the VAT breakdown (`BG-23`) into the invoice, collecting exemption reasons
    // and the VAT point event.
    fn parse_tax_breakdown<I: Invoice>(
        &mut self,
        invoice: &mut I,
    ) -> Result<(ExemptionMap, Option<VatPoint>), Error> {
        let mut exemptions = ExemptionMap::new();
        let mut vat_point = None;
        while self.is_open(Namespace::Ram, "ApplicableTradeTax") {
            let instance = index(invoice.vat_breakdown().len());
            self.enter_repeatable(
                Namespace::Ram,
                "ApplicableTradeTax",
                "vat_breakdown",
                instance,
            )?;
            let tax = self
                .optional_text(Namespace::Ram, "CalculatedAmount", "tax")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            self.optional_derived(Namespace::Ram, "TypeCode")?;
            let text: Option<NonEmptyString> = self
                .optional_text(Namespace::Ram, "ExemptionReason", "treatment")?
                .map(|text| text.parse())
                .transpose()?;
            let taxable = self
                .optional_text(Namespace::Ram, "BasisAmount", "taxable")?
                .map(|text| parse_decimal(&text))
                .transpose()?;
            let category: Option<VatCategory> = self
                .optional_text(Namespace::Ram, "CategoryCode", "treatment")?
                .map(|text| text.parse())
                .transpose()?;
            let code: Option<VatExemptionReason> = self
                .optional_text(Namespace::Ram, "ExemptionReasonCode", "treatment")?
                .map(|text| text.parse())
                .transpose()?;
            if self.is_open(Namespace::Ram, "DueDateTypeCode") {
                let event = self.derived(Namespace::Ram, "DueDateTypeCode")?.1;
                vat_point = Some(VatPoint::Event(event.parse()?));
            }
            let rate: Option<Percentage> = self
                .optional_text(Namespace::Ram, "RateApplicablePercent", "treatment")?
                .map(|text| text.parse())
                .transpose()?;
            self.leave_repeatable()?;
            let treatment = match category {
                Some(category) => {
                    let rate = match rate {
                        Some(rate) => rate,
                        None => Percentage::try_from(Decimal::ZERO)?,
                    };
                    Some(match VatTreatment::from_category(category, rate) {
                        VatTreatment::Exempt { .. } => VatTreatment::Exempt {
                            code,
                            text: text.clone(),
                        },
                        treatment => treatment,
                    })
                }
                None => None,
            };
            if category == Some(VatCategory::Exempt) {
                exemptions.set(code, text);
            }
            invoice.vat_breakdown().push(VatBreakdown {
                treatment,
                taxable,
                tax,
            });
        }
        Ok((exemptions, vat_point))
    }

    fn parse_adjustment(&mut self, instance: NonZeroUsize) -> Result<Adjustment, Error> {
        self.enter_repeatable(
            Namespace::Ram,
            "SpecifiedTradeAllowanceCharge",
            "adjustments",
            instance,
        )?;
        let charge = self.optional_indicator()?;
        let amount = self.parse_adjustment_amount()?;
        let reason = self.parse_reason(charge)?;
        let vat = if self.is_open(Namespace::Ram, "CategoryTradeTax") {
            self.enter_nested(Namespace::Ram, "CategoryTradeTax")?;
            self.optional_derived(Namespace::Ram, "TypeCode")?;
            let category: VatCategory = self.derived(Namespace::Ram, "CategoryCode")?.1.parse()?;
            let rate = self
                .derived(Namespace::Ram, "RateApplicablePercent")?
                .1
                .parse()?;
            self.leave_nested()?;
            Some(VatTreatment::from_category(category, rate))
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

    // Parses the amount of an adjustment, absolute or relative, absent without the amount.
    fn parse_adjustment_amount(&mut self) -> Result<Option<AdjustmentAmount>, Error> {
        let rate = self.optional_text(Namespace::Ram, "CalculationPercent", "amount")?;
        let base = self
            .optional_text(Namespace::Ram, "BasisAmount", "amount")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        let amount = self
            .optional_text(Namespace::Ram, "ActualAmount", "amount")?
            .map(|text| parse_decimal(&text))
            .transpose()?;
        Ok(match (rate, base, amount) {
            (Some(rate), Some(base), Some(amount)) => Some(AdjustmentAmount::Relative {
                amount,
                rate: rate.parse()?,
                base,
            }),
            (None, _, Some(amount)) => Some(AdjustmentAmount::Absolute(amount)),
            _ => None,
        })
    }

    // Parses the reason code and text of an adjustment, dropped without the direction.
    fn parse_reason(&mut self, charge: Option<bool>) -> Result<Option<AdjustmentReason>, Error> {
        let code = if self.is_open(Namespace::Ram, "ReasonCode") {
            Some(self.derived(Namespace::Ram, "ReasonCode")?.1)
        } else {
            None
        };
        let text = if self.is_open(Namespace::Ram, "Reason") {
            Some(self.derived(Namespace::Ram, "Reason")?.1.parse()?)
        } else {
            None
        };
        Ok(match charge {
            Some(true) => Some(AdjustmentReason::Charge {
                code: code.map(|code| code.parse()).transpose()?,
                text,
            }),
            Some(false) => Some(AdjustmentReason::Allowance {
                code: code.map(|code| code.parse()).transpose()?,
                text,
            }),
            None => None,
        })
    }

    #[allow(clippy::type_complexity)]
    fn parse_payment_terms(
        &mut self,
    ) -> Result<(Option<NonEmptyString>, Option<Date>, Option<NonEmptyString>), Error> {
        self.enter_structural(Namespace::Ram, "SpecifiedTradePaymentTerms")?;
        let terms = self.optional_leaf(Namespace::Ram, "Description", "payment_terms")?;
        let due = self.optional_datetime("DueDateDateTime", "payment_due_date")?;
        let mandate =
            self.optional_leaf(Namespace::Ram, "DirectDebitMandateID", "mandate_reference")?;
        self.leave_structural()?;
        Ok((terms, due, mandate))
    }

    // Parses the header monetary summation into the invoice, keeping every stated amount.
    fn parse_monetary_summation<I: Invoice>(&mut self, invoice: &mut I) -> Result<(), Error> {
        if !self.is_open(
            Namespace::Ram,
            "SpecifiedTradeSettlementHeaderMonetarySummation",
        ) {
            return Ok(());
        }
        self.enter_structural(
            Namespace::Ram,
            "SpecifiedTradeSettlementHeaderMonetarySummation",
        )?;
        while self.is_open_namespace(Namespace::Ram) {
            let (_, name) = self.head()?;
            let (field, slot): (&'static str, &mut Option<Decimal>) = match name.as_str() {
                "LineTotalAmount" => ("line_net_total", invoice.line_net_total()),
                "ChargeTotalAmount" => ("charges_total", invoice.charges_total()),
                "AllowanceTotalAmount" => ("allowances_total", invoice.allowances_total()),
                "TaxBasisTotalAmount" => ("net_total", invoice.net_total()),
                "TaxTotalAmount" => ("vat_total", invoice.vat_total()),
                "RoundingAmount" => ("rounding", invoice.rounding()),
                "GrandTotalAmount" => ("gross_total", invoice.gross_total()),
                "TotalPrepaidAmount" => ("paid", invoice.paid()),
                "DuePayableAmount" => ("due", invoice.due()),
                _ => {
                    self.derived(Namespace::Ram, &name)?;
                    continue;
                }
            };
            let text = self.leaf(Namespace::Ram, &name, field)?;
            *slot = Some(parse_decimal(&text)?);
        }
        self.leave_structural()?;
        Ok(())
    }

    fn parse_preceding_invoice(
        &mut self,
        instance: NonZeroUsize,
    ) -> Result<InvoiceReference, Error> {
        self.enter_repeatable(
            Namespace::Ram,
            "InvoiceReferencedDocument",
            "preceding_invoices",
            instance,
        )?;
        let number = self.optional_issuer_assigned_id()?;
        let issue_date = if self.is_open(Namespace::Ram, "FormattedIssueDateTime") {
            self.enter_nested(Namespace::Ram, "FormattedIssueDateTime")?;
            self.take_open(Namespace::Qdt, "DateTimeString")?;
            let date = self.take_text();
            self.take_close()?;
            self.leave_nested()?;
            Some(parse_date(&date)?)
        } else {
            None
        };
        self.leave_repeatable()?;
        Ok(InvoiceReference { number, issue_date })
    }

    // Parses a billing period (`BG-14`/`BG-26`).
    fn billing_period(&mut self, field: &'static str) -> Result<Period, Error> {
        self.enter_group(Namespace::Ram, "BillingSpecifiedPeriod", field)?;
        let start = self.optional_datetime("StartDateTime", field)?;
        let end = self.optional_datetime("EndDateTime", field)?;
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
        let id = self.optional_issuer_assigned_id()?;
        self.leave_group()?;
        Ok(id)
    }

    // ---- datatype carriers ----------------------------------------------

    // Reads an optional date wrapper whose value carrier is a `udt:DateTimeString`.
    fn optional_datetime(
        &mut self,
        element: &str,
        field: &'static str,
    ) -> Result<Option<Date>, Error> {
        if self.is_open(Namespace::Ram, element) {
            Ok(Some(self.datetime(element, field)?))
        } else {
            Ok(None)
        }
    }

    // Reads a date wrapper whose value carrier is a `udt:DateTimeString`.
    fn datetime(&mut self, element: &str, field: &'static str) -> Result<Date, Error> {
        self.enter_group(Namespace::Ram, element, field)?;
        self.take_open(Namespace::Udt, "DateTimeString")?;
        let text = self.take_text();
        self.take_close()?;
        self.leave_group()?;
        parse_date(&text)
    }

    // Reads an optional charge indicator whose value carrier is a `udt:Indicator`.
    fn optional_indicator(&mut self) -> Result<Option<bool>, Error> {
        if self.is_open(Namespace::Ram, "ChargeIndicator") {
            Ok(Some(self.indicator()?))
        } else {
            Ok(None)
        }
    }

    // Reads a charge indicator whose value carrier is a `udt:Indicator`.
    fn indicator(&mut self) -> Result<bool, Error> {
        self.enter_nested(Namespace::Ram, "ChargeIndicator")?;
        self.take_open(Namespace::Udt, "Indicator")?;
        let text = self.take_text();
        self.take_close()?;
        self.leave_nested()?;
        Ok(text.trim() == "true")
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
}

// ---- helpers -------------------------------------------------------------

enum AdditionalKind {
    Object,
    Tender,
    Supporting,
}

enum TaxRegistration {
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
    let unit = QuantityUnit::from_code(code)
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
