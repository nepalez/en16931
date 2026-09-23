//! Re-exports of the core the crate builds on.

pub use en16931_core::{
    Adjustment, Amount, Buyer, Cii, Contact, CountryCode, Currency, Date, Decimal, Delivery,
    Deserializable, DocumentBuilder, ElectronicAddress, EmailAddress, Error, Invoice, InvoiceKind,
    InvoiceReference, InvoiceType, Item, ItemAttribute, ItemClassification, ItemReference,
    LegalEntity, Line, LineAdjustment, LocationReference, Namespace, NonEmptyString, Note,
    ObjectReference, OperationalEntity, Parser, Payee, PaymentInstructions, Period, PostalAddress,
    Price, Quantity, Seller, Serializable, Serializer, SupportingDocument, TaxRepresentative, Ubl,
    VatBreakdown, VatIdentifier, VatPoint, VatTreatment, cii, ubl,
};
