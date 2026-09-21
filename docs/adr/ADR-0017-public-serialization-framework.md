# ADR-0017: Public Serialization Framework

## Context

A conformant extension of [EN-16931] adds content the core model does not carry (ADR-0003). A survey of the published extensions shows two classes of such content.

The first class holds native syntax elements outside the semantic model. Examples are sublines of an invoice line, a second payment group, and a withholding total.

The second class holds elements of a namespace of its own. Every found case writes them in the document-level `ext:UBLExtensions` block.

No found extension writes a whole document from scratch. None changes a formula of the standard either. Each one adds nodes to the document the core writes.

## Problem

What does the core publish to a third-party crate? The crate must write and parse the content of its extension.

## Decision

> The core publishes the writing framework as a stable public API.

The write events, the trace, and the dictionary registration are public. The base walk keeps the order of the schema, and asks the extension at every position. One generic hook carries that question, and its default body is empty.

A `Slot` names a position in the terms of the record form (ADR-0004). One form points inside a group, the other after that group. The addressing is complete by construction, so any future position is expressible.

A `Scope` writes into a slot. It takes single nodes, nested subtrees, namespace declarations, and dictionary entries (ADR-0009).

The parse is symmetric. An element unknown to the base walk reaches the extension with its slot.

## Alternatives Considered

* **A full event API.** The extension writes the whole document through published events. Rationale: it admits any layout an extension may need. Rejection: no found extension writes a document from scratch, and each one would duplicate the base walk.

* **A list of named hooks.** The core names one method per known attachment point. Rationale: each point reads clearly at the call site. Rejection: the list is an enum growing inside the core, and it misses the places no survey foresaw.

* **A soak inside the core before the publication.** The framework serves the core alone until a release proves it. Rationale: a published API is hard to revise. Rejection: the extension crates would wait for a whole release cycle.

## Consequences

### Pros

* An extension attaches at any position, including one no survey foresaw.
* The framework guarantees the order and the nesting of the nodes it writes.
* An extension crate lands without a core release.

### Cons

* The core owes stability to the framework it publishes.
* A slot reads less clearly than a named attachment point.

## References

[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
