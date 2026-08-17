mod deserialize;
mod serialize;

use crate::Namespace;
pub(crate) use deserialize::deserialize;
pub(crate) use serialize::serialize;

// The CII record-form namespaces, shared by both halves.
const RSM: Namespace = Namespace::CrossIndustryInvoice;
const RAM: Namespace = Namespace::ReusableAggregateBusinessInformationEntity;

// The datatype-carrier namespaces, present in the XML but absent from the record form.
const UDT_PREFIX: &str = "udt";
const QDT_PREFIX: &str = "qdt";
const UDT_URI: &str = "urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100";
const QDT_URI: &str = "urn:un:unece:uncefact:data:standard:QualifiedDataType:100";

// The XML prefix a CII document binds to a record-form namespace.
fn prefix(namespace: Namespace) -> &'static str {
    match namespace {
        RSM => "rsm",
        RAM => "ram",
        _other => unreachable!("a CII document never carries the {_other} namespace"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Binding;
    use crate::format::test_helpers::{builder, pretty, variant_builder};

    #[test]
    fn detects_its_own_output_as_cii() {
        let (xml, _) = serialize(&builder(Binding::Cii));

        assert_eq!(Binding::detect(&xml).expect("a CII document"), Binding::Cii);
    }

    #[test]
    fn rebuilds_the_same_dictionary_on_parse() {
        let (xml, written) = serialize(&builder(Binding::Cii));

        let (_, read) = deserialize(&xml).expect("a valid CII document");

        assert_eq!(read, written);
    }

    #[test]
    fn emit_fixtures() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/src/format/cii/fixtures");
        let (rich, _) = serialize(&builder(Binding::Cii));
        let (variant, _) = serialize(&variant_builder(Binding::Cii));
        std::fs::write(format!("{base}/1.xml"), pretty(&rich)).expect("write");
        std::fs::write(format!("{base}/2.xml"), pretty(&variant)).expect("write");
    }

    #[test]
    fn serializes_the_document_to_cii() {
        let (xml, _) = serialize(&builder(Binding::Cii));

        assert_eq!(pretty(&xml), include_str!("cii/fixtures/1.xml"));
    }

    #[test]
    fn deserializes_the_document_from_cii() {
        let (parsed, _) =
            deserialize(include_str!("cii/fixtures/1.xml")).expect("a valid CII document");

        assert_eq!(parsed, builder(Binding::Cii));
    }

    #[test]
    fn serializes_the_variant_document_to_cii() {
        let (xml, _) = serialize(&variant_builder(Binding::Cii));

        assert_eq!(pretty(&xml), include_str!("cii/fixtures/2.xml"));
    }

    #[test]
    fn deserializes_the_variant_document_from_cii() {
        let (parsed, _) =
            deserialize(include_str!("cii/fixtures/2.xml")).expect("a valid CII document");

        assert_eq!(parsed, variant_builder(Binding::Cii));
    }
}
