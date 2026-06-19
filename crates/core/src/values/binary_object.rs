//! A binary object: an embedded document with its media type and filename (`BT-125`).

use crate::{Error, MimeCode, NonEmptyString};

/// A binary object (`BT-125`): an embedded document, its media type, and its filename.
///
/// The content holds the raw bytes and is base64-encoded only at serialization. It must be
/// non-empty. The media type and filename are mandatory whenever a binary object is present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryObject {
    content: Vec<u8>,
    mime_code: MimeCode,
    filename: NonEmptyString,
}

impl BinaryObject {
    /// Builds a binary object from its content, media type, and filename.
    /// Rejects empty content and a blank filename.
    pub fn new(content: Vec<u8>, mime_code: MimeCode, filename: &str) -> Result<Self, Error> {
        if content.is_empty() {
            return Err(Error::InvalidValue("an empty binary object".to_owned()));
        }
        Ok(Self {
            content,
            mime_code,
            filename: filename.parse()?,
        })
    }

    /// The raw document content.
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// The media type of the content.
    pub fn mime_code(&self) -> MimeCode {
        self.mime_code
    }

    /// The document filename.
    pub fn filename(&self) -> &str {
        self.filename.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builds_from_content_mime_and_filename() {
        let object = BinaryObject::new(vec![1, 2, 3], MimeCode::Pdf, "invoice.pdf")
            .expect("a non-empty binary object builds");

        assert_eq!(object.content(), &[1, 2, 3]);
        assert_eq!(object.mime_code(), MimeCode::Pdf);
        assert_eq!(object.filename(), "invoice.pdf");
    }

    #[test]
    fn rejects_empty_content() {
        assert!(matches!(
            BinaryObject::new(vec![], MimeCode::Pdf, "invoice.pdf"),
            Err(Error::InvalidValue(_))
        ));
    }

    #[test]
    fn rejects_a_blank_filename() {
        assert!(BinaryObject::new(vec![1], MimeCode::Pdf, "   ").is_err());
    }
}
