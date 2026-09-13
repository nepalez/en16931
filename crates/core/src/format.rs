use crate::prelude::*;
use crate::{Abbreviations, Binding, Context, Error, Namespace, Path};

pub(crate) mod trace;

#[cfg(test)]
pub(crate) mod test_helpers;

pub mod cii;
pub mod ubl;

pub use cii::Cii;
pub use ubl::Ubl;

/// An XML binding of the standard, represented by a marker type.
#[allow(private_bounds)]
pub trait Format: Sealed {
    /// The base set of record-form namespaces this binding writes.
    type Namespace: Namespace;

    /// The local name of the root element of a document in this binding.
    const ROOT_ELEMENT: &'static str;

    /// The binding this marker stands for, as a validator's target names it.
    const BINDING: Binding;

    /// The namespace of the root element of a document in this binding.
    fn root_namespace() -> Self::Namespace;

    /// Reads a document into resolved, owned tokens,
    /// collecting the abbreviations it declares and dropping insignificant whitespace.
    ///
    /// A malformed document, an element without a namespace,
    /// or an element of a namespace outside the set yield `Error::MalformedXml`.
    #[allow(private_interfaces)]
    fn tokenize(xml: &str) -> Result<Tokenized<Self::Namespace>, Error> {
        let mut reader = NsReader::from_str(xml);
        let mut tokens = Vec::new();
        let mut abbreviations = Self::Namespace::default_abbreviations();
        loop {
            let (resolved, event) = reader.read_resolved_event()?;
            match event {
                Event::Start(start) => {
                    tokens.push(open_token::<Self>(resolved, &start, &mut abbreviations)?);
                }
                Event::Empty(start) => {
                    tokens.push(open_token::<Self>(resolved, &start, &mut abbreviations)?);
                    tokens.push(Token::Close);
                }
                Event::End(_) => tokens.push(Token::Close),
                Event::Text(value) => {
                    let bytes = value.into_inner();
                    let text = String::from_utf8_lossy(&bytes);
                    if !text.trim().is_empty() {
                        tokens.push(Token::Text(text.into_owned()));
                    }
                }
                Event::Eof => return Ok((tokens, abbreviations)),
                _ => {}
            }
        }
    }
}

pub(crate) trait Sealed {}

pub(crate) type Tokenized<N> = (Vec<Token<N>>, Abbreviations<N>);

pub(crate) enum Token<N: Namespace> {
    Open {
        namespace: N,
        name: String,
        attributes: Vec<(String, String)>,
    },
    Text(String),
    Close,
}

// Builds an `Open` token from a start tag, resolving its namespace and reading
// its attributes. A namespace declaration binds an abbreviation instead.
fn open_token<F: Format + ?Sized>(
    resolved: ResolveResult<'_>,
    start: &BytesStart<'_>,
    abbreviations: &mut Abbreviations<F::Namespace>,
) -> Result<Token<F::Namespace>, Error> {
    let ResolveResult::Bound(uri) = resolved else {
        return Err(Error::malformed_xml("an element has no namespace"));
    };
    let uri = String::from_utf8_lossy(uri.into_inner());
    let namespace = F::Namespace::from_uri(&uri)
        .ok_or_else(|| Error::malformed_xml(format!("unknown namespace: {uri}")))?;
    let name = String::from_utf8_lossy(start.local_name().as_ref()).into_owned();
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        if key == b"xmlns" || key.starts_with(b"xmlns:") {
            let abbreviation = String::from_utf8_lossy(key.strip_prefix(b"xmlns:").unwrap_or(b""));
            let uri = String::from_utf8_lossy(&attribute.value);
            if let Some(namespace) = F::Namespace::from_uri(&uri) {
                abbreviations.declare(&abbreviation, namespace)?;
            }
            continue;
        }
        let key = String::from_utf8_lossy(attribute.key.local_name().as_ref()).into_owned();
        let value = String::from_utf8_lossy(&attribute.value).into_owned();
        attributes.push((key, value));
    }
    Ok(Token::Open {
        namespace,
        name,
        attributes,
    })
}

/// The dictionary to bind nodes of the XML into the document's ones.
///
/// The key is the node's record-form `Path` over the namespace set `N`.
/// The value is the `Context` for a consumer to highlight.
pub type Dictionary<N> = HashMap<Path<N>, Context>;
