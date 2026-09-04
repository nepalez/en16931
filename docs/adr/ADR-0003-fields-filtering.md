# ADR-0003: Fields Filtering

## Context

[EN-16931] defines a core invoice. Country and sector profiles restrict it. A profile forbids some terms and requires others.

An extension of the standard adds terms the core model does not carry.

One business invoice may target several profiles (for example, during cross-border operations).

## Problem

How does the library hold the data for every profile at once?

How does it keep a profile-forbidden term out of the output?

## Decision

> The core holds one unified model.

It is the superset of every term in [EN-16931] and in its supported [CIUS] profiles. Every term is optional, except the type code `BT-3`, which selects the rule set. The consumer fills every term it knows without tracking per-profile rules.

The model checks the type of a value, not the presence of a term. A missing term is a finding of the validator, not a library error.

A profile version is another supported profile. The superset only grows. An optional term is added, never removed, while any supported version needs it.

The superset is not the only form of an invoice. An extension declares an invoice form of its own, which carries terms beyond the standard. That form converts into the core invoice, dropping those terms.

Serialization targets one profile skipping terms it forbids. The library runs no other checks, leaving them to a service (ADR-0001).

An invoice serves several profiles through separate serializations. Each carries its own `BT-24` value. Each is validated on its own.

## Alternatives Considered

* **Consumer-side filtering.** The caller removes forbidden terms before serialization. Rejected because it spreads per-profile knowledge across every consumer.

* **Document kind derived from the sign of the amounts.** Rejected because `BT-3` is a code, not a kind, and a negative invoice is legal.

## Consequences

### Pros

* The consumer fills every known term, and serialization drops what a profile forbids.
* One invoice reaches several profiles without a conversion step.
* A new profile adds its restrictions without touching the shared model.
* An extension carries its own terms without growing the shared model.
* A document with a missing term parses, and the validator reports it.

### Cons

* The model carries terms that a given profile could never emit.
* Any error except for forbidden fields surfaces only at the service, not at serialization.
* A conversion to the core invoice drops the terms of an extension.
* The model admits an incomplete invoice.

## Examples

A Dutch seller issues under [NLCIUS]. The buyer requires [Peppol BIS]. One document holds the shared content. It carries `BT-23`, which [Peppol BIS] requires and [NLCIUS] allows. It omits `BT-21`, which [Peppol BIS] forbids.

The seller serializes the document under [NLCIUS] and validates it locally. The seller then serializes the same invoice under [Peppol BIS] and sends that. The buyer validates the received document under [Peppol BIS].

## References

[CIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108937/CIUS+and+Extension+-+What+is+allowed
[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[NLCIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108895/eInvoicing+in+The+Netherlands
[Peppol BIS]: https://docs.peppol.eu/poacc/billing/3.0/bis/
