use crate::Error;
use crate::prelude::*;

// The position of a step whose address omits it.
const FIRST: NonZeroUsize = NonZeroUsize::new(1).expect("a positive index");

/// Rewrite an address of the ISO-skeleton dialect into the normalized form.
///
/// The dialect names an element of a step in three ways.
///
/// 1. It writes a prefixed name, `cac:LegalMonetaryTotal[1]`,
/// 2. or it writes a wildcard and moves the namespace into a predicate,
///    `*:LegalMonetaryTotal[namespace-uri()='U']`,
/// 3. or it moves the name there as well,
///    `*[local-name()='LegalMonetaryTotal' and namespace-uri()='U'][1]`.
///
/// The position is optional, and a step that omits it stands first among its siblings.
///
/// The reader strips that syntax and nothing else.
/// It keeps the namespace in the form the address wrote it,
/// because only the table of the checked document binds an abbreviation.
/// A step no element of the model answers, an attribute among them, stays in place,
/// since the match against a document is the one that drops it.
///
/// An address outside the grammar fails the whole report.
#[derive(Debug)]
pub struct Iso;

impl Normalizer for Iso {
    type Error = Error;

    fn normalize(&self, location: &str) -> Result<Location, Self::Error> {
        let address = location.trim();
        let steps = split(address.strip_prefix('/').unwrap_or(address))
            .into_iter()
            .map(step)
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Error::Malformed(location.to_owned()))?;
        Ok(Location { steps })
    }
}

/// What a step of the address writes before its predicates.
enum Head {
    // A wildcard, with the name when the step still spells it.
    Wildcard(Option<String>),
    // A name, with the abbreviation of the namespace it belongs to.
    Element(String, String),
    // The name of an attribute.
    Attribute(String),
}

/// What a predicate of a step asserts about it.
enum Claim {
    // The local name of the element.
    Name(String),
    // The namespace URI of the element.
    Namespace(String),
    // The position among the same-named siblings.
    Position(NonZeroUsize),
}

fn split(address: &str) -> Vec<&str> {
    let mut steps = Vec::new();
    let mut quote = None;
    let mut depth = 0usize;
    let mut start = 0usize;
    for (at, symbol) in address.char_indices() {
        match symbol {
            _ if quote == Some(symbol) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(symbol),
            _ if quote.is_some() => {}
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            '/' if depth == 0 => {
                steps.push(&address[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    steps.push(&address[start..]);
    steps
}

fn step(source: &str) -> Option<LocationStep> {
    let cut = source.find('[').unwrap_or(source.len());
    let (front, tail) = source.split_at(cut);

    let mut name = None;
    let mut uri = None;
    let mut index = FIRST;
    for body in predicates(tail)? {
        for conjunct in conjuncts(body) {
            match claim(conjunct.trim())? {
                Claim::Name(value) => name = Some(value),
                Claim::Namespace(value) => uri = Some(value),
                Claim::Position(value) => index = value,
            }
        }
    }

    match head(front)? {
        Head::Wildcard(local) => Some(LocationStep {
            namespace: Some(RawNamespace::Uri(uri?)),
            name: single(local, name)?,
            index,
        }),
        Head::Element(abbreviation, local) => uri.is_none().then_some(LocationStep {
            namespace: Some(RawNamespace::Abbreviation(abbreviation)),
            name: single(Some(local), name)?,
            index,
        }),
        Head::Attribute(local) => uri.is_none().then_some(LocationStep {
            namespace: None,
            name: single(Some(local), name)?,
            index,
        }),
    }
}

fn head(source: &str) -> Option<Head> {
    if let Some(attribute) = source.strip_prefix('@') {
        let local = attribute
            .split_once(':')
            .map_or(attribute, |(_, name)| name);
        return Some(Head::Attribute(named(local)?));
    }
    if source == "*" {
        return Some(Head::Wildcard(None));
    }
    if let Some(local) = source.strip_prefix("*:") {
        return Some(Head::Wildcard(Some(named(local)?)));
    }
    let (abbreviation, local) = source.split_once(':').unwrap_or(("", source));
    let abbreviation = match abbreviation {
        "" => String::new(),
        _ => named(abbreviation)?,
    };
    Some(Head::Element(abbreviation, named(local)?))
}

fn predicates(tail: &str) -> Option<Vec<&str>> {
    let mut bodies = Vec::new();
    let mut rest = tail;
    while !rest.is_empty() {
        let body = rest.strip_prefix('[')?;
        let end = closing(body)?;
        bodies.push(&body[..end]);
        rest = &body[end + 1..];
    }
    Some(bodies)
}

fn closing(body: &str) -> Option<usize> {
    let mut quote = None;
    let mut depth = 0usize;
    for (at, symbol) in body.char_indices() {
        match symbol {
            _ if quote == Some(symbol) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(symbol),
            _ if quote.is_some() => {}
            '[' => depth += 1,
            ']' if depth == 0 => return Some(at),
            ']' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn conjuncts(body: &str) -> Vec<&str> {
    const AND: &str = " and ";

    let mut parts = Vec::new();
    let mut quote = None;
    let mut start = 0usize;
    for (at, symbol) in body.char_indices() {
        match symbol {
            _ if quote == Some(symbol) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(symbol),
            _ if quote.is_some() => {}
            _ if at >= start && body[at..].starts_with(AND) => {
                parts.push(&body[start..at]);
                start = at + AND.len();
            }
            _ => {}
        }
    }
    parts.push(&body[start..]);
    parts
}

fn claim(source: &str) -> Option<Claim> {
    if let Some(value) = argument(source, "local-name()") {
        return Some(Claim::Name(named(&value)?));
    }
    if let Some(value) = argument(source, "namespace-uri()") {
        return Some(Claim::Namespace(value));
    }
    source.parse().ok().map(Claim::Position)
}

fn argument(source: &str, function: &str) -> Option<String> {
    let value = source
        .strip_prefix(function)?
        .trim_start()
        .strip_prefix('=')?
        .trim_start();
    let quote = value.chars().next()?;
    match quote {
        '\'' | '"' => {
            let literal = value.strip_prefix(quote)?.strip_suffix(quote)?;
            Some(literal.replace(&format!("{quote}{quote}"), &quote.to_string()))
        }
        _ => None,
    }
}

fn single(head: Option<String>, predicate: Option<String>) -> Option<String> {
    match (head, predicate) {
        (Some(name), None) | (None, Some(name)) => Some(name),
        _ => None,
    }
}

fn named(source: &str) -> Option<String> {
    let mut symbols = source.chars();
    let head = symbols.next()?;
    let tail = symbols.all(|symbol| symbol.is_alphanumeric() || matches!(symbol, '_' | '-' | '.'));
    ((head.is_alphabetic() || head == '_') && tail).then(|| source.to_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    const INV: &str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";
    const CAC: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2";
    const CBC: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";
    const RSM: &str = "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100";
    const RAM: &str =
        "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100";

    fn abbreviated(abbreviation: &str, name: &str, index: usize) -> LocationStep {
        LocationStep {
            namespace: Some(RawNamespace::Abbreviation(abbreviation.to_owned())),
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    fn resolved(uri: &str, name: &str, index: usize) -> LocationStep {
        LocationStep {
            namespace: Some(RawNamespace::Uri(uri.to_owned())),
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    #[test]
    fn reads_an_address_of_prefixed_names() {
        let expected = Ok(Location {
            steps: vec![
                abbreviated("ubl", "Invoice", 1),
                abbreviated("cac", "InvoiceLine", 2),
                abbreviated("cbc", "ID", 1),
            ],
        });

        let actual = Iso.normalize("/ubl:Invoice/cac:InvoiceLine[2]/cbc:ID");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_an_address_of_named_predicates() {
        let expected = Ok(Location {
            steps: vec![
                resolved(INV, "Invoice", 1),
                resolved(CAC, "InvoiceLine", 2),
                resolved(CBC, "ID", 1),
            ],
        });

        let actual = Iso.normalize(&format!(
            "/*[local-name()='Invoice' and namespace-uri()='{INV}'][1]\
             /*[local-name()='InvoiceLine' and namespace-uri()='{CAC}'][2]\
             /*[local-name()='ID' and namespace-uri()='{CBC}'][1]"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_an_address_of_wildcard_names() {
        let expected = Ok(Location {
            steps: vec![resolved(INV, "Invoice", 1), resolved(CAC, "InvoiceLine", 2)],
        });

        let actual = Iso.normalize(&format!(
            "/*:Invoice[namespace-uri()='{INV}']/*:InvoiceLine[namespace-uri()='{CAC}'][2]"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_an_address_of_the_other_binding() {
        let expected = Ok(Location {
            steps: vec![
                resolved(RSM, "CrossIndustryInvoice", 1),
                resolved(RSM, "SupplyChainTradeTransaction", 1),
                resolved(RAM, "IncludedSupplyChainTradeLineItem", 2),
                resolved(RAM, "AssociatedDocumentLineDocument", 1),
                resolved(RAM, "LineID", 1),
            ],
        });

        let actual = Iso.normalize(&format!(
            "/*[local-name()='CrossIndustryInvoice' and namespace-uri()='{RSM}']\
             /*[local-name()='SupplyChainTradeTransaction' and namespace-uri()='{RSM}']\
             /*[local-name()='IncludedSupplyChainTradeLineItem' and namespace-uri()='{RAM}'][2]\
             /*[local-name()='AssociatedDocumentLineDocument' and namespace-uri()='{RAM}']\
             /*[local-name()='LineID' and namespace-uri()='{RAM}']"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_namespace_that_carries_separators() {
        let expected = Ok(Location {
            steps: vec![resolved("http://example.org/ns/one", "Invoice", 1)],
        });

        let actual = Iso.normalize(
            "/*[local-name()='Invoice' and namespace-uri()='http://example.org/ns/one']",
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_step_of_the_default_namespace() {
        let expected = Ok(Location {
            steps: vec![abbreviated("", "Invoice", 1), abbreviated("", "ID", 1)],
        });

        let actual = Iso.normalize("/Invoice/ID");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_step_of_an_attribute() {
        let expected = Ok(Location {
            steps: vec![
                abbreviated("ubl", "Invoice", 1),
                abbreviated("cbc", "ID", 1),
                LocationStep {
                    namespace: None,
                    name: "schemeID".to_owned(),
                    index: FIRST,
                },
            ],
        });

        let actual = Iso.normalize("/ubl:Invoice/cbc:ID/@schemeID");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_an_attribute_of_a_prefixed_name() {
        let expected = Ok(Location {
            steps: vec![
                abbreviated("ubl", "Invoice", 1),
                LocationStep {
                    namespace: None,
                    name: "ID".to_owned(),
                    index: FIRST,
                },
            ],
        });

        let actual = Iso.normalize("/ubl:Invoice/@cbc:ID");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_an_address_of_no_leading_separator() {
        let expected = Ok(Location {
            steps: vec![
                abbreviated("ubl", "Invoice", 1),
                abbreviated("cbc", "ID", 1),
            ],
        });

        let actual = Iso.normalize("ubl:Invoice/cbc:ID");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_the_last_position_a_step_writes() {
        let expected = Ok(Location {
            steps: vec![abbreviated("ubl", "Invoice", 2)],
        });

        let actual = Iso.normalize("/ubl:Invoice[1][2]");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_namespace_in_double_quotes() {
        let expected = Ok(Location {
            steps: vec![resolved(INV, "Invoice", 1)],
        });

        let actual = Iso.normalize(&format!(
            "/*[local-name()=\"Invoice\" and namespace-uri()=\"{INV}\"]"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_predicate_of_loose_spacing() {
        let expected = Ok(Location {
            steps: vec![resolved(INV, "Invoice", 1)],
        });

        let actual = Iso.normalize(&format!(
            "/*[ local-name() = 'Invoice'  and  namespace-uri() = '{INV}' ]"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_namespace_that_carries_a_conjunction() {
        let expected = Ok(Location {
            steps: vec![resolved("urn:example:a and b", "Invoice", 1)],
        });

        let actual =
            Iso.normalize("/*[local-name()='Invoice' and namespace-uri()='urn:example:a and b']");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_namespace_that_carries_a_quote() {
        let expected = Ok(Location {
            steps: vec![resolved("urn:example:a'b", "Invoice", 1)],
        });

        let actual =
            Iso.normalize("/*[local-name()='Invoice' and namespace-uri()='urn:example:a''b']");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_address_outside_the_grammar() {
        let expected = Err(Error::Malformed("/ubl:Invoice/text()".to_owned()));

        let actual = Iso.normalize("/ubl:Invoice/text()");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_wildcard_of_an_unwritten_namespace() {
        let expected = Err(Error::Malformed("/*:Invoice".to_owned()));

        let actual = Iso.normalize("/*:Invoice");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_unclosed_predicate() {
        let expected = Err(Error::Malformed("/ubl:Invoice[1".to_owned()));

        let actual = Iso.normalize("/ubl:Invoice[1");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_unclosed_literal() {
        let address = format!("/ubl:Invoice[namespace-uri()='{INV}]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Iso.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_predicate_of_an_unknown_kind() {
        let expected = Err(Error::Malformed("/ubl:Invoice[@a='b']".to_owned()));

        let actual = Iso.normalize("/ubl:Invoice[@a='b']");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_predicate_of_nested_brackets() {
        let expected = Err(Error::Malformed("/ubl:Invoice[foo[1]]".to_owned()));

        let actual = Iso.normalize("/ubl:Invoice[foo[1]]");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_position_of_zero() {
        let expected = Err(Error::Malformed("/ubl:Invoice[0]".to_owned()));

        let actual = Iso.normalize("/ubl:Invoice[0]");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_wildcard_without_a_name() {
        let address = format!("/*[namespace-uri()='{INV}']");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Iso.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_named_wildcard_without_a_namespace() {
        let expected = Err(Error::Malformed("/*[local-name()='Invoice']".to_owned()));

        let actual = Iso.normalize("/*[local-name()='Invoice']");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_prefixed_name_of_a_written_namespace() {
        let address = format!("/ubl:Invoice[namespace-uri()='{INV}']");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Iso.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_attribute_of_a_written_namespace() {
        let address = format!("/ubl:Invoice/@schemeID[namespace-uri()='{INV}']");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Iso.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_name_written_twice() {
        let address = format!("/*:Invoice[local-name()='Other' and namespace-uri()='{INV}']");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Iso.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_name_of_a_leading_digit() {
        let expected = Err(Error::Malformed("/ubl:1Invoice".to_owned()));

        let actual = Iso.normalize("/ubl:1Invoice");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_descendant_step() {
        let expected = Err(Error::Malformed("/ubl:Invoice//cbc:ID".to_owned()));

        let actual = Iso.normalize("/ubl:Invoice//cbc:ID");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_trailing_separator() {
        let expected = Err(Error::Malformed("/ubl:Invoice/".to_owned()));

        let actual = Iso.normalize("/ubl:Invoice/");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_address_of_a_bare_separator() {
        let expected = Err(Error::Malformed("/".to_owned()));

        let actual = Iso.normalize("/");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_empty_address() {
        let expected = Err(Error::Malformed(String::new()));

        let actual = Iso.normalize("");

        assert_eq!(actual, expected);
    }
}
