# ADR-0010: Resolution Context

## Context

The library binds each `RawReport` entry to a model field (ADR-0009) to highlight it in the UI.

A failed assertion carries one location, one rule identifier, and a message. The message names the rule and the business terms, like `BT-131`. ADR-0008 introduced `Context` but left its payload open.

## Problem

What does a `Context` carry? Does it repeat the profile identity that the report already states? How many does one report entry resolve to? What if a location names a node that maps to no term?

## Decision

A `Context` points to one model node for the UI to highlight. The target is a leaf field, a repeatable-group instance, or the `Document` root. The path carries the concrete index of every group it crosses.

It omits the profile identity. The rule identifier and the business terms stay in the report entry text.

A report entry resolves to exactly one `Context`.

A failed assertion in [SVRL] carries one location:
* a stored `BT` rule yields its leaf field;
* a computed `BT` rule yields its owning group, or the root;
* a `BG` group yields that group instance as a single node;
* a document-wide rule yields the `Document` root;
* a node with no assigned term yields the root.

A structural binding element nests around terms, yet [EN-16931] assigns it no `BT` or `BG`. The [CII] `SupplyChainTradeTransaction` is one example. The `Binding` alone knows it, owning the binding-to-term correspondence (ADR-0009). It records each term-less node, structural or met on parse, with the root `Context`. Resolution reads the stored target and inspects no node.

This keeps a node with no term apart from an unbound location. Such a node is a dictionary hit bound to the root, a finding. An unbound location is a dictionary miss from a binding or dialect mismatch. So it is a library error (ADR-0007), never a finding.

## Alternatives Considered

* **A `Context` per inner field of the group.** A `BG`-group location would expand to one `Context` for every leaf inside the group instance. It was rejected because one reported node maps to one node, and the consumer can drill into the group itself.

* **Profile identity inside `Context`.** A consumer could read the rule from the field alone. It was rejected because the entry already carries the rule identifier and terms.

## Consequences

### Pros

* The UI highlights the exact reported node, including its repeatable-group index.
* A `BG` group of any rank resolves through the same single lookup as a leaf field.
* `Context` stays free of profile identity, so a new profile adds no field.

### Cons

* The consumer reads the rule identifier and the terms from the entry text.
* A `BG`-group complaint yields only the group node, so a consumer that wants the inner fields expands the group itself.
* A complaint on a structural binding element highlights the whole document, mapping to the root.

## References

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[SVRL]: https://schematron.com/document/3427.html