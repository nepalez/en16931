# ADR-0015: XML Engine for the Binding Serializer

## Context

The core must (de)serialize a `DocumentBuilder` to both XML bindings, [UBL] and [CII] (ADR-0005). The same pass carries a second duty. As it walks the document, it fills a dictionary (ADR-0009). The dictionary keys each node by its record-form path (ADR-0004). So the serializer needs node-level access to names, namespaces, and positions.

## Problem

The core ships no XML library yet. That choice shapes the whole `Binding` implementation. So the library must be chosen before `Binding` is written.

Which XML library should back the `Binding` (de)serializer?

## Decision

The `Binding` (de)serializer uses [quick-xml] through its event reader and writer.

The event API reads and writes XML and resolves namespaces. It exposes every node during a traversal. So `Binding` tracks each node's namespace, local name, and sibling index in one pass. That pass fills the record-form dictionary as it goes.

A [CII] datatype carrier from the `udt` or `qdt` namespace holds a value only. The writer emits it as a raw event without a dictionary entry (ADR-0009).

`Document::parse` reads the root element namespace to detect the binding (ADR-0005).

## Alternatives Considered

Alternative 1: [quick-xml] with a [Serde] derive.

* Rationale: a derive maps the model declaratively and cuts manual code.
* Rejection: a derive hides the traversal and yields no node path. The dictionary would then need a second pass. The derive also handles namespaces weakly.

Alternative 2: a [xot] tree.

* Rationale: a full tree offers parent access, namespaces, and free traversal.
* Rejection: the crate stays niche and its releases have stalled. Writing still builds the tree node by node.

## Consequences

### Pros

* `Binding` fills the dictionary in the pass that reads or writes the XML.
* Namespace handling stays explicit and matches each binding exactly.
* The dependency is mature and widely adopted.

### Cons

* Each model node needs a handwritten mapping for both bindings.
* The event API exposes XML at a low level.

## References

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[quick-xml]: https://github.com/tafia/quick-xml
[Serde]: https://serde.rs/
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[xot]: https://github.com/faassen/xot
