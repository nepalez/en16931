use super::Trace;
use crate::prelude::{BytesEnd, BytesStart, BytesText, Event, NonZeroUsize, VariantArray, Writer};
use crate::{Abbreviations, Dictionary, Format, Namespace, Term};

/// The stateful writer of the binding `F`, handed to a `Serializable` walk.
///
/// It renders the XML and records every written node into the dictionary in the same pass,
/// so a finding of a validator later binds to the model context of the node it addresses.
/// The walk writes the document through the writing primitives only, from the root element down,
/// and never touches the XML or the dictionary directly.
pub struct Serializer<F: Format> {
    inner: Writer<Vec<u8>>,
    pub(crate) trace: Trace<F::Namespace>,
    abbreviations: Abbreviations<F::Namespace>,
    forbidden: &'static [Term],
}

impl<F: Format> Serializer<F> {
    pub(crate) fn new(forbidden: &'static [Term]) -> Self {
        Self {
            inner: Writer::new(Vec::new()),
            trace: Trace::new(),
            abbreviations: F::Namespace::default_abbreviations(),
            forbidden,
        }
    }

    pub(crate) fn finish(
        self,
    ) -> (
        String,
        Dictionary<F::Namespace>,
        Abbreviations<F::Namespace>,
    ) {
        let xml = String::from_utf8(self.inner.into_inner()).expect("quick-xml emits valid UTF-8");
        (xml, self.trace.into_dictionary(), self.abbreviations)
    }

    // Whether the profile forbids the term of a node about to be written.
    pub(crate) fn forbids(&self, term: Term) -> bool {
        self.forbidden.contains(&term)
    }

    // Writes the root element of the binding, declaring every namespace of its set,
    // and records it at the root context.
    pub(crate) fn root(&mut self, body: impl FnOnce(&mut Self)) {
        let declarations: Vec<(String, &'static str)> = F::Namespace::VARIANTS
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
        let root = BytesStart::new(qname(F::root_namespace(), F::ROOT_ELEMENT))
            .with_attributes(declarations.iter().map(|(key, uri)| (key.as_str(), *uri)));
        self.write(Event::Start(root));
        for namespace in F::Namespace::VARIANTS {
            self.abbreviations
                .declare(namespace.prefix(), *namespace)
                .expect("the writer binds each abbreviation to one namespace");
        }
        self.trace.enter(F::root_namespace(), F::ROOT_ELEMENT);
        self.trace.record_root();
        body(self);
        self.trace.leave();
        self.write(Event::End(BytesEnd::new(qname(
            F::root_namespace(),
            F::ROOT_ELEMENT,
        ))));
    }

    // Writes a term-bearing leaf, skipped when the profile forbids the term.
    pub(crate) fn leaf(
        &mut self,
        namespace: F::Namespace,
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
    pub(crate) fn leaf_attr(
        &mut self,
        namespace: F::Namespace,
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
    pub(crate) fn field_leaf(
        &mut self,
        namespace: F::Namespace,
        name: &str,
        field: &'static str,
        value: &str,
    ) {
        self.write_field_leaf(namespace, name, field, &[], value);
    }

    // Writes a leaf with attributes mapped to a model field, never filtered.
    pub(crate) fn field_leaf_attr(
        &mut self,
        namespace: F::Namespace,
        name: &str,
        field: &'static str,
        attributes: &[(&str, &str)],
        value: &str,
    ) {
        self.write_field_leaf(namespace, name, field, attributes, value);
    }

    pub(crate) fn write_field_leaf(
        &mut self,
        namespace: F::Namespace,
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
    pub(crate) fn repeatable_leaf(
        &mut self,
        namespace: F::Namespace,
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
    pub(crate) fn rooted(
        &mut self,
        namespace: F::Namespace,
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
    pub(crate) fn derived(
        &mut self,
        namespace: F::Namespace,
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
    pub(crate) fn group(
        &mut self,
        namespace: F::Namespace,
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

    // Writes a group mapped to a model field, never filtered (its term is never forbidden).
    pub(crate) fn field_group(
        &mut self,
        namespace: F::Namespace,
        name: &str,
        field: &'static str,
        body: impl FnOnce(&mut Self),
    ) {
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
    pub(crate) fn repeatable(
        &mut self,
        namespace: F::Namespace,
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
    pub(crate) fn structural(
        &mut self,
        namespace: F::Namespace,
        name: &str,
        body: impl FnOnce(&mut Self),
    ) {
        self.trace.enter(namespace, name);
        self.trace.record_root();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    // Writes a wrapper mapped to the enclosing context, without a new segment.
    pub(crate) fn nested(
        &mut self,
        namespace: F::Namespace,
        name: &str,
        body: impl FnOnce(&mut Self),
    ) {
        self.trace.enter(namespace, name);
        self.trace.record_context();
        self.write_start(namespace, name);
        body(self);
        self.write_end(namespace, name);
        self.trace.leave();
    }

    // Writes a whole element with its attributes and text, never recorded.
    pub(crate) fn write_element(
        &mut self,
        namespace: F::Namespace,
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

    pub(crate) fn write_start(&mut self, namespace: F::Namespace, name: &str) {
        self.write(Event::Start(BytesStart::new(qname(namespace, name))));
    }

    pub(crate) fn write_end(&mut self, namespace: F::Namespace, name: &str) {
        self.write(Event::End(BytesEnd::new(qname(namespace, name))));
    }

    fn write(&mut self, event: Event<'_>) {
        self.inner
            .write_event(event)
            .expect("writing XML to an in-memory buffer never fails");
    }
}

// The record-form qualified name of an element, prefixed for its namespace.
fn qname<N: Namespace>(namespace: N, name: &str) -> String {
    let prefix = namespace.prefix();
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}:{name}")
    }
}
