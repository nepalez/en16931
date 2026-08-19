# ADR-0009: Binding Correspondence

## Context

The library must resolve [SVRL] report locations into typed model fields. A failed assertion carries one location. An incompatible location matches nothing (ADR-0004).

The location is not arbitrary. Mainstream validators compile rules through the standard [Schematron] skeleton. Its `schematron-get-full-path` function numbers each step by counting preceding siblings of the same name. So every step carries a positional index, never a value predicate.

Serialization and parsing already know the correspondence between `Invoice` fields and XML elements. The XML binding fixes it. SVRL resolution is open.

## Problem

How does the library resolve a positional [SVRL] location into a typed model field?

What if a location matches no node?

## Decision

> The `Binding` builds the dictionary, one entry per node it handles.

It enters every node, even one that carries no model term. Serialization of a `DocumentBuilder` drives it, and `Document::parse` drives the inverse path. The key is the node's path in record form (ADR-0004). The value is a `Context`. Knowledge of the binding enters only here, through `Binding`.

The same pass yields the abbreviation table of the document (ADR-0004). The keys keep the namespace itself, so a path stays comparable across documents.

The dictionary serves one purpose. It matches a rule violation to a model field following the location where the rule fired. So the dictionary needs only those entries that the rules can refer to. The [CII] datatype namespaces `udt` and `qdt` carry values, like `udt:DateTimeString`, but no rules ever reference them. The `Binding` writes such a wrapper into the XML, yet stores no entry in the dictionary.

`Document::check` resolves each location against the dictionary by a bounded lookup. It never evaluates [XPath]. The location and the keys share the positional record form. So each positional predicate matches the stored index. An incompatible location stays unresolved, which is a library error, not a finding (ADR-0007).

## Alternatives Considered

* **Value-predicate resolution.** Store each node's value and match a filter predicate against it. Rejected because the skeleton never emits a value predicate in a location. The stored value would serve no input.

* **XML evaluation.** Evaluate each location against the document's stored XML to find the node. Rejected because positional locations need no engine. It would reintroduce an [XPath] interpreter the project avoids (ADR-0001).

* **Static pattern registry.** A declarative manifest of [XPath] templates with index variables maps onto `Context`. Rejected because the manifest needs maintenance alongside every binding and schema update. The dictionary instead derives from the `Binding` pass.

* **Opaque [XPath] string in `Report`.** Each entry keeps the raw location and offers no model-side path. Rejected because it drops the typed-path promise of ADR-0008. It pushes the resolution problem onto every consumer.

* **Abbreviations as the keys.** A location would match a key literally, with no translation at all. Rejected because a dialect that writes the URI would then need the reverse translation. Such a key would also turn specific to one document.

* **Local names and indexes as the keys.** The namespace would be dropped, and a lookup would compare names alone. Rejected because a location of the wrong binding would then match a node of the right name.

## Consequences

### Pros

* The dictionary derives from the `Binding` pass, with no external manifest to maintain.
* Each entry yields a typed `Context`, so the consumer never parses a raw location.

### Cons

* The dictionary holds an entry per node, growing with document size.
* `Document` carries the dictionary alongside its `xml`, which complicates its passage through FFI.
* A location that names an abbreviation resolves it before the lookup can start.

## References

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[Schematron]: https://schematron.com/
[Schematron skeleton]: https://github.com/Schematron/schematron/blob/master/trunk/schematron/code/iso_schematron_skeleton_for_saxon.xsl
[SVRL]: https://schematron.com/document/3427.html
[XPath]: https://www.w3.org/TR/xpath-31/
