# ADR-0006: Project Extendability

## Context

The library extends along several axes. Mature validators differ both in envelope and in the [XPath] dialect of `location` (ADR-0004). Each axis varies across products and across configurations of the same product. The library must bound the contract between the core and any extension.

The core holds the invoice interfaces and the [UBL] and [CII] walks (ADR-0005). Several extension concerns surround it, each revised upstream on its own schedule:
* per-format validator-output parsers,
* profiles of countries and sectors, and further extensions (ADR-0001).

## Problem

How is the system decomposed into crates and what versioning policy should be followed to provide extendability?

Which contract binds the core to an extension, and which concerns grow along it?

## Decision

A virtual manifest at the root holds the workspace. Every crate lives under `crates/`, grouped into a per-axis subfolder. Each folder name describes the axis, while the crate name carries the `en16931-` prefix:

```text
en16931/
└── crates/
    ├── core/                  — en16931-core: interfaces, bindings, profile trait, extension traits
    ├── dialects/{iso, ...}    — en16931-{iso, ...}: xpath dialects
    ├── envelopes/{svrl, ...}  — en16931-{svrl, ...}: validator-output envelopes
    └── extensions/{cius, ...} — en16931-{cius, ...}: invoice types and profiles
```

> The invoice interfaces and the two bindings are not extension axes. A profile is one.

The invoice interfaces and the bindings live in the core (ADR-0005), unified across standards. The base invoice types and the [CIUS] profiles live in `cius`. A [CIUS] profile stamps `BT-24` and forbids terms of the base types (ADR-0003). A profile of an extension also brings terms the base types lack. The volatile rules of a profile are [Schematron], owned by the validator or its proxy (ADR-0002). So a new profile needs no core release, and the core declares `Profile` as a trait.

A profile type names its binding, its namespace set, and its invoice type. It stamps `BT-24` per document kind, as a credit note may differ there. A binding is a marker type of a sealed trait. The sealing keeps the binding set closed to the core (ADR-0005).

The core defines five extension traits, and each of them is open:
* `Serializable` and `Deserializable` write an invoice type into a binding and parse it back,
* a hook trait lets an extension write and parse its own nodes,
* `Envelope` unwraps the validator-specific answer,
* `Dialect` rewrites a processor location into the record form (ADR-0004).

Sealing any of them would admit growth inside the core alone, which defeats the axis.

A validator service declares a trait of its own. It derives the vendor id of a rule set from a profile type and a document kind (ADR-0002). A service trait that covers the [CIUS] profiles depends on `cius`.

Extensions are plain Cargo dependencies, not feature flags. The consumer composes them at the call site, pairing an `Envelope` with a `Dialect`. Together they turn a validator artifact into the record form (ADR-0004).

Core follows semver. Each extension crate carries its own version. It declares a compatibility range of core in `Cargo.toml`. A breaking change in a trait, or a newly supported standard, bumps the core. Extensions update their range and re-release.

## Alternatives Considered

* **Typed edges over an erased middle.** A profile type appears at the entry and the exit only. The artifact in between holds the base invoice. Rationale: the type parameter then stays out of the middle types. Rejection: the parse must restore the content of an extension. The erased middle cannot hold it.

* **Dispatch on the `BT-24` string.** A registry maps the identifier of a document onto a profile. Rationale: the parse then recognizes the profile on its own. Rejection: an open set of types needs an external registry. Its entries are checked at runtime, not by the compiler.

* **Conversion-driven extension contract.** Each extension exports a newtype and a `TryFrom` into a single report type. Rejected because envelope and [XPath] dialect conflate inside one type. Switching dialect within one service requires a new extension crate instead of recombining two existing ones.

* **Lock-step semver across the workspace.** One version on every crate, bumped together. Rejected because every new extension would force a major core release.

* **Independent semver without compat range.** Each crate versions on its own track, with no declared range of core. Rejected because consumers cannot tell which extension version pairs with which core.

* **Flat crate folders named after the crates.** Every crate sits directly under `crates/`, each folder named exactly as its crate. Rationale: the common convention eases navigation and renames. Rejection: a shared `en16931-` prefix forces long flat names. The per-axis grouping then disappears.

## Consequences

### Pros

* A parsed document keeps the content of its extension.
* A new profile ships in any crate, and the compiler checks its bindings.
* Both dialect and envelope recombine without new extension crates.
* Each axis releases on its own track.
* Consumers see exact compatibility windows.
* The per-axis grouping coexists with a shared `en16931-` crate prefix.

### Cons

* A consumer combines two extension crates instead of one.
* The parse needs the profile in advance, since no registry recognizes it.

## References

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[CIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108937/CIUS+and+Extension+-+What+is+allowed
[Schematron]: https://schematron.com/
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[XPath]: https://www.w3.org/TR/xpath-31/
