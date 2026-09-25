//! The reading machinery shared by the bindings: a cursor over the resolved tokens
//! and the trace that rebuilds the dictionary in lockstep, behind one set of reading primitives.

use super::{Token, Trace};
use crate::prelude::{NonZeroUsize, PhantomData};
use crate::{Dictionary, Error, NonEmptyString};
use crate::{Format, Namespace};

/// The stateful reader of the binding `F`, handed to a `Deserializable` walk.
///
/// It reads the resolved tokens of a document through a cursor and records every read node
/// into the dictionary in the same pass, so a finding of a validator later binds
/// to the model context of the node it addresses. The walk reads the document
/// through the reading primitives only, from the root element down.
pub struct Parser<F: Format, N: Namespace + From<F::Namespace>> {
    pub(crate) tokens: Vec<Token<N>>,
    pub(crate) cursor: usize,
    pub(crate) trace: Trace<N>,
    format: PhantomData<F>,
}

impl<F: Format, N: Namespace + From<F::Namespace>> Parser<F, N> {
    pub(crate) fn new(tokens: Vec<Token<N>>) -> Self {
        Self {
            tokens,
            cursor: 0,
            trace: Trace::new(),
            format: PhantomData,
        }
    }

    pub(crate) fn finish(self) -> Dictionary<N> {
        self.trace.into_dictionary()
    }

    // ---- reading primitives ---------------------------------------------

    // Reads a leaf mapped to a model field, returning its text.
    pub(crate) fn leaf(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        Ok(self.leaf_attr(namespace, name, field)?.1)
    }

    // Reads a leaf mapped to a model field, returning its attributes and text.
    pub(crate) fn leaf_attr(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
    ) -> Result<(Vec<(String, String)>, String), Error> {
        let namespace = namespace.into();
        let attributes = self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_field(field);
        self.trace.record_context();
        let text = self.take_text();
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok((attributes, text))
    }

    // Reads an optional leaf mapped to a model field.
    pub(crate) fn optional_leaf(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
    ) -> Result<Option<NonEmptyString>, Error> {
        let namespace = namespace.into();
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf(namespace, name, field)?.parse()?))
        } else {
            Ok(None)
        }
    }

    // Reads the text of an optional leaf mapped to a model field.
    pub(crate) fn optional_text(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
    ) -> Result<Option<String>, Error> {
        let namespace = namespace.into();
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf(namespace, name, field)?))
        } else {
            Ok(None)
        }
    }

    // Reads an optional leaf mapped to a model field, returning its attributes and text.
    #[allow(clippy::type_complexity)]
    pub(crate) fn optional_leaf_attr(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
    ) -> Result<Option<(Vec<(String, String)>, String)>, Error> {
        let namespace = namespace.into();
        if self.is_open(namespace, name) {
            Ok(Some(self.leaf_attr(namespace, name, field)?))
        } else {
            Ok(None)
        }
    }

    // Reads an optional derived leaf with no model field, returning its attributes and text.
    #[allow(clippy::type_complexity)]
    pub(crate) fn optional_derived(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
    ) -> Result<Option<(Vec<(String, String)>, String)>, Error> {
        let namespace = namespace.into();
        if self.is_open(namespace, name) {
            Ok(Some(self.derived(namespace, name)?))
        } else {
            Ok(None)
        }
    }

    // Reads a repeatable single-value element.
    pub(crate) fn repeatable_leaf(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
        instance: NonZeroUsize,
    ) -> Result<String, Error> {
        let namespace = namespace.into();
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_instance(field, instance);
        self.trace.record_context();
        let text = self.take_text();
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok(text)
    }

    // Reads a leaf mapped to a field of the invoice itself from inside a group,
    // returning its text.
    pub(crate) fn header_leaf(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
    ) -> Result<String, Error> {
        let namespace = namespace.into();
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_header(field);
        let text = self.take_text();
        self.take_close()?;
        self.trace.leave();
        Ok(text)
    }

    // Reads a regulatory leaf recorded at the root context.
    pub(crate) fn rooted(&mut self, namespace: impl Into<N>, name: &str) -> Result<String, Error> {
        let namespace = namespace.into();
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_root();
        let text = self.take_text();
        self.take_close()?;
        self.trace.leave();
        Ok(text)
    }

    // Reads a derived leaf with no model field, returning its attributes and text.
    pub(crate) fn derived(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
    ) -> Result<(Vec<(String, String)>, String), Error> {
        let namespace = namespace.into();
        let attributes = self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_context();
        let text = self.take_text();
        self.take_close()?;
        self.trace.leave();
        Ok((attributes, text))
    }

    pub(crate) fn enter_structural(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
    ) -> Result<(), Error> {
        let namespace = namespace.into();
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_root();
        Ok(())
    }

    pub(crate) fn leave_structural(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.leave();
        Ok(())
    }

    pub(crate) fn enter_nested(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
    ) -> Result<(), Error> {
        let namespace = namespace.into();
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.record_context();
        Ok(())
    }

    pub(crate) fn leave_nested(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.leave();
        Ok(())
    }

    pub(crate) fn enter_group(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
    ) -> Result<(), Error> {
        let namespace = namespace.into();
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_field(field);
        self.trace.record_context();
        Ok(())
    }

    pub(crate) fn leave_group(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok(())
    }

    pub(crate) fn enter_repeatable(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
        field: &'static str,
        instance: NonZeroUsize,
    ) -> Result<(), Error> {
        let namespace = namespace.into();
        self.take_open(namespace, name)?;
        self.trace.enter(namespace, name);
        self.trace.push_instance(field, instance);
        self.trace.record_context();
        Ok(())
    }

    pub(crate) fn leave_repeatable(&mut self) -> Result<(), Error> {
        self.take_close()?;
        self.trace.pop_context();
        self.trace.leave();
        Ok(())
    }

    // ---- token cursor ---------------------------------------------------

    // Peeks the text of a direct child element by local name, without consuming.
    pub(crate) fn peek_child_text(&self, child: &str) -> Option<String> {
        let mut depth = 0usize;
        let mut cursor = self.cursor;
        while let Some(token) = self.tokens.get(cursor) {
            match token {
                Token::Open { name, .. } => {
                    if depth == 1 && name == child {
                        if let Some(Token::Text(text)) = self.tokens.get(cursor + 1) {
                            return Some(text.trim().to_owned());
                        }
                    }
                    depth += 1;
                }
                Token::Close => {
                    if depth <= 1 {
                        return None;
                    }
                    depth -= 1;
                }
                Token::Text(_) => {}
            }
            cursor += 1;
        }
        None
    }

    pub(crate) fn head(&self) -> Result<(N, String), Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace, name, ..
            }) => Ok((*namespace, name.clone())),
            _ => Err(Error::malformed_xml("expected an element")),
        }
    }

    pub(crate) fn is_open(&self, namespace: impl Into<N>, name: &str) -> bool {
        let namespace = namespace.into();
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, name: local, .. }) if *found == namespace && local == name
        )
    }

    pub(crate) fn is_open_namespace(&self, namespace: impl Into<N>) -> bool {
        let namespace = namespace.into();
        matches!(
            self.tokens.get(self.cursor),
            Some(Token::Open { namespace: found, .. }) if *found == namespace
        )
    }

    pub(crate) fn take_open(
        &mut self,
        namespace: impl Into<N>,
        name: &str,
    ) -> Result<Vec<(String, String)>, Error> {
        let namespace = namespace.into();
        match self.tokens.get(self.cursor) {
            Some(Token::Open {
                namespace: found,
                name: local,
                attributes,
            }) if *found == namespace && local == name => {
                let attributes = attributes.clone();
                self.cursor += 1;
                Ok(attributes)
            }
            _ => Err(Error::malformed_xml(format!("expected <{name}>"))),
        }
    }

    pub(crate) fn take_close(&mut self) -> Result<(), Error> {
        match self.tokens.get(self.cursor) {
            Some(Token::Close) => {
                self.cursor += 1;
                Ok(())
            }
            _ => Err(Error::malformed_xml("expected an end tag")),
        }
    }

    pub(crate) fn take_text(&mut self) -> String {
        if let Some(Token::Text(text)) = self.tokens.get(self.cursor) {
            let text = text.clone();
            self.cursor += 1;
            text
        } else {
            String::new()
        }
    }
}
