use crate::Error;
use crate::prelude::*;

// The position of a step whose address never carries one.
const FIRST: NonZeroUsize = NonZeroUsize::new(1).expect("a positive index");

// The step `fn:path` writes for the default namespace node.
const UNNAMED_NAMESPACE: &str = "*[Q{http://www.w3.org/2005/xpath-functions}local-name()=\"\"]";

/// Rewrite an address of the SchXslt2 dialect into the normalized form.
///
/// The second SchXslt version leaves the location to the standard `fn:path`
/// function, unless a deployment supplies its own.
/// The function writes one step per node
/// on the way from the document root down to the reported one.
///
/// 1. An element step spells the namespace URI and the position,
///    `Q{urn:...}Invoice[1]`, and neither of them is ever omitted.
///    An element outside every namespace keeps the empty URI, `Q{}Invoice[1]`.
/// 2. An attribute closes the address as `@Q{urn:...}local` in a namespace,
///    or as a bare `@local` outside of one, without a position.
/// 3. A text node, a comment, or a processing instruction closes it as
///    `text()[1]`, `comment()[1]`, or `processing-instruction(name)[1]`.
/// 4. A namespace node closes it as `namespace::prefix`,
///    or as `namespace::*[...local-name()=""]` for the default namespace,
///    without a position.
///
/// A validated document is always rooted at a document node,
/// so an address always begins with a slash.
///
/// The reader strips that syntax and nothing else.
/// A step no element of the model answers, a closing step among them,
/// stays in place, since the match against a document is the one that drops it.
///
/// An address outside the grammar fails the whole report.
#[derive(Debug)]
pub struct Schxslt2;

impl Normalizer for Schxslt2 {
    type Error = Error;

    fn normalize(&self, location: &str) -> Result<Location, Self::Error> {
        let address = location
            .trim()
            .strip_prefix('/')
            .filter(|rest| !rest.is_empty())
            .ok_or_else(|| Error::Malformed(location.to_owned()))?;
        let sources = split(address);
        let closing = sources.len() - 1;
        let steps = sources
            .into_iter()
            .enumerate()
            .map(|(at, source)| step(source, at == closing))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Error::Malformed(location.to_owned()))?;
        Ok(Location { steps })
    }
}

fn split(address: &str) -> Vec<&str> {
    let mut steps = Vec::new();
    let mut inside_uri = false;
    let mut start = 0usize;
    for (at, symbol) in address.char_indices() {
        match symbol {
            '{' => inside_uri = true,
            '}' => inside_uri = false,
            '/' if !inside_uri => {
                steps.push(&address[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    steps.push(&address[start..]);
    steps
}

fn step(source: &str, closing: bool) -> Option<LocationStep> {
    if let Some(body) = source.strip_prefix('@') {
        return closing.then_some(body).and_then(attribute);
    }
    if let Some(prefix) = source.strip_prefix("namespace::") {
        return closing.then_some(prefix).and_then(namespace_node);
    }
    if let Some(step) = node(source) {
        return closing.then_some(step);
    }
    element(source)
}

// Reads an element step: the namespace URI, the local name, and the position.
fn element(source: &str) -> Option<LocationStep> {
    let (uri, rest) = uri(source)?;
    let (name, index) = indexed(rest)?;
    Some(LocationStep {
        namespace: Some(RawNamespace::Uri(uri.to_owned())),
        name: named(name)?,
        index,
    })
}

// Reads the closing step of an attribute, dropping the namespace when it spells one.
fn attribute(body: &str) -> Option<LocationStep> {
    let name = match uri(body) {
        Some((_, local)) => local,
        None if body.starts_with("Q{") => return None,
        None => body,
    };
    Some(LocationStep {
        namespace: None,
        name: named(name)?,
        index: FIRST,
    })
}

// Reads the closing step of a text node, a comment, or a processing instruction.
fn node(source: &str) -> Option<LocationStep> {
    let (head, index) = indexed(source)?;
    let known = head == "text()"
        || head == "comment()"
        || head
            .strip_prefix("processing-instruction(")
            .and_then(|rest| rest.strip_suffix(')'))
            .and_then(named)
            .is_some();
    known.then(|| LocationStep {
        namespace: None,
        name: head.to_owned(),
        index,
    })
}

// Reads the closing step of a namespace node, named by its prefix or unnamed.
fn namespace_node(prefix: &str) -> Option<LocationStep> {
    (prefix == UNNAMED_NAMESPACE || named(prefix).is_some()).then(|| LocationStep {
        namespace: None,
        name: format!("namespace::{prefix}"),
        index: FIRST,
    })
}

// Splits the `Q{U}` head of a step, returning the URI and the tail after it.
fn uri(source: &str) -> Option<(&str, &str)> {
    source.strip_prefix("Q{")?.split_once('}')
}

// Splits the trailing `[i]` position of a step, returning the head and the index.
fn indexed(source: &str) -> Option<(&str, NonZeroUsize)> {
    let (head, digits) = source.strip_suffix(']')?.rsplit_once('[')?;
    Some((head, digits.parse().ok()?))
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

    fn resolved(uri: &str, name: &str, index: usize) -> LocationStep {
        LocationStep {
            namespace: Some(RawNamespace::Uri(uri.to_owned())),
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    fn unmatched(name: &str, index: usize) -> LocationStep {
        LocationStep {
            namespace: None,
            name: name.to_owned(),
            index: NonZeroUsize::new(index).expect("a positive index"),
        }
    }

    #[test]
    fn reads_an_address_of_the_ubl_binding() {
        let expected = Ok(Location {
            steps: vec![
                resolved(INV, "Invoice", 1),
                resolved(CAC, "InvoiceLine", 2),
                resolved(CBC, "ID", 1),
            ],
        });

        let actual = Schxslt2.normalize(&format!(
            "/Q{{{INV}}}Invoice[1]/Q{{{CAC}}}InvoiceLine[2]/Q{{{CBC}}}ID[1]"
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

        let actual = Schxslt2.normalize(&format!(
            "/Q{{{RSM}}}CrossIndustryInvoice[1]\
             /Q{{{RSM}}}SupplyChainTradeTransaction[1]\
             /Q{{{RAM}}}IncludedSupplyChainTradeLineItem[2]\
             /Q{{{RAM}}}AssociatedDocumentLineDocument[1]\
             /Q{{{RAM}}}LineID[1]"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_closing_step_of_a_bare_attribute() {
        let expected = Ok(Location {
            steps: vec![
                resolved(INV, "Invoice", 1),
                resolved(CBC, "ID", 1),
                unmatched("schemeID", 1),
            ],
        });

        let actual =
            Schxslt2.normalize(&format!("/Q{{{INV}}}Invoice[1]/Q{{{CBC}}}ID[1]/@schemeID"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_closing_step_of_a_namespaced_attribute() {
        let expected = Ok(Location {
            steps: vec![resolved(INV, "Invoice", 1), unmatched("type", 1)],
        });

        let actual = Schxslt2.normalize(&format!(
            "/Q{{{INV}}}Invoice[1]/@Q{{http://www.w3.org/2001/XMLSchema-instance}}type"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_closing_step_of_a_text_node() {
        let expected = Ok(Location {
            steps: vec![
                resolved(INV, "Invoice", 1),
                resolved(CBC, "Note", 1),
                unmatched("text()", 2),
            ],
        });

        let actual = Schxslt2.normalize(&format!(
            "/Q{{{INV}}}Invoice[1]/Q{{{CBC}}}Note[1]/text()[2]"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_closing_step_of_a_processing_instruction() {
        let expected = Ok(Location {
            steps: vec![
                resolved(INV, "Invoice", 1),
                unmatched("processing-instruction(xml-stylesheet)", 1),
            ],
        });

        let actual = Schxslt2.normalize(&format!(
            "/Q{{{INV}}}Invoice[1]/processing-instruction(xml-stylesheet)[1]"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_closing_step_of_a_named_namespace_node() {
        let expected = Ok(Location {
            steps: vec![resolved(INV, "Invoice", 1), unmatched("namespace::cbc", 1)],
        });

        let actual = Schxslt2.normalize(&format!("/Q{{{INV}}}Invoice[1]/namespace::cbc"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_closing_step_of_the_default_namespace_node() {
        let expected = Ok(Location {
            steps: vec![
                resolved(INV, "Invoice", 1),
                unmatched(&format!("namespace::{UNNAMED_NAMESPACE}"), 1),
            ],
        });

        let actual = Schxslt2.normalize(&format!(
            "/Q{{{INV}}}Invoice[1]/namespace::{UNNAMED_NAMESPACE}"
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_an_element_of_an_empty_namespace() {
        let expected = Ok(Location {
            steps: vec![resolved("", "Invoice", 1)],
        });

        let actual = Schxslt2.normalize("/Q{}Invoice[1]");

        assert_eq!(actual, expected);
    }

    #[test]
    fn reads_a_namespace_that_carries_separators() {
        let expected = Ok(Location {
            steps: vec![resolved("http://example.org/ns/one", "Invoice", 1)],
        });

        let actual = Schxslt2.normalize("/Q{http://example.org/ns/one}Invoice[1]");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_element_without_a_position() {
        let address = format!("/Q{{{INV}}}Invoice");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_unclosed_namespace() {
        let address = format!("/Q{{{INV}Invoice[1]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_step_without_a_name() {
        let address = format!("/Q{{{INV}}}[1]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_position_of_zero() {
        let address = format!("/Q{{{INV}}}Invoice[0]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_address_of_a_prefixed_name() {
        let expected = Err(Error::Malformed("/ubl:Invoice[1]".to_owned()));

        let actual = Schxslt2.normalize("/ubl:Invoice[1]");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_attribute_of_a_position() {
        let address = format!("/Q{{{INV}}}Invoice[1]/@schemeID[1]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_attribute_before_the_closing_step() {
        let address = format!("/Q{{{INV}}}Invoice[1]/@id/Q{{{CBC}}}ID[1]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_processing_instruction_of_a_quoted_target() {
        let address = format!("/Q{{{INV}}}Invoice[1]/processing-instruction(\"x\")[1]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_address_of_no_leading_separator() {
        let address = format!("Q{{{INV}}}Invoice[1]");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_a_trailing_separator() {
        let address = format!("/Q{{{INV}}}Invoice[1]/");
        let expected = Err(Error::Malformed(address.clone()));

        let actual = Schxslt2.normalize(&address);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_address_of_a_bare_separator() {
        let expected = Err(Error::Malformed("/".to_owned()));

        let actual = Schxslt2.normalize("/");

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_an_empty_address() {
        let expected = Err(Error::Malformed(String::new()));

        let actual = Schxslt2.normalize("");

        assert_eq!(actual, expected);
    }
}
