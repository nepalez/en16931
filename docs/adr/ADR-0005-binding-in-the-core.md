# ADR-0005: Binding in the Core

## Context

The library (de)serializes invoices to the two [EN-16931] XML bindings, [UBL] and [CII] (ADR-0001).

Around the core, several concerns extend the library on their own upstream schedules:
* Country and sector profiles restrict the model (ADR-0001).
* Validator envelopes wrap the output.
* Processor dialects vary the location syntax (ADR-0004).

Each grows as countries and validators multiply.

A binding looks like one more such concern, since it too has more than one form. Its place must be fixed before the extension axes are drawn.

## Problem

Is an XML binding an extension axis, like a profile or a dialect, or does it belong to the core?

## Decision

> Both bindings live in the core. The core carries the [UBL] and [CII] walks as public functions. A binding is not an extension axis.

The binding set is closed and standards-controlled. [Directive 2014/55/EU] mandates a limited list of compliant syntaxes, and [EN-16931] admits exactly these two. A third would need a [CEN] revision and a new binding, not a downstream choice. The current direction, [ViDA], consolidates rather than expands.

So a new binding is far less probable than a new profile, envelope, or dialect that can be added without touching the standard.

The consumer states the binding before writing or parsing a document. The core keeps both bindings behind one closed set, which no downstream crate extends. Each walk is generic over the invoice traits. A contract for a concrete type calls the walk. No blanket implementation exists, since it would seal an open trait.

A binding marker carries its namespace set as a type of its own and names the root namespace. The [UBL] root and its namespace depend on the document kind. An invoice and a credit note are different [UBL] documents, while [CII] keeps one root.

## Alternatives Considered

**Binding as an extension axis.**

Each binding ships as its own crate, composed at the call site like an envelope or a dialect. Rationale: every varying concern gets the same treatment.

Rejected because the binding set is closed, so the extension cost stays unjustified. A consumer-supplied binding would also need a registry to reach the parser.

**Detection of the binding from the content.**

The parser reads the root element and picks the binding itself. Rationale: an inbound document arrives from a counterparty, so its binding stays outside the consumer's control.

Rejected because the consumer knows the binding before parsing. A contract or an exchange channel fixes it.

## Consequences

### Pros

* The binding is fixed before the parse begins.
* The closed binding set carries no per-crate versioning or call-site composition.

### Cons

* A new binding, however rare, requires a core release.
* The core grows to carry both bindings' (de)serialization.
* A consumer that accepts both bindings drives two separate paths.

## References

[CEN]: https://en.wikipedia.org/wiki/European_Committee_for_Standardization
[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[Directive 2014/55/EU]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108867/European+legislation+on+eInvoicing
[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[ViDA]: https://taxation-customs.ec.europa.eu/taxation/vat/vat-digital-age-vida_en
