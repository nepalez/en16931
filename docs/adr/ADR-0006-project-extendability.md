# ADR-0006: Project Extendability

## Context

The library extends along several axes. Mature validators differ both in envelope and in the [XPath] dialect of `location` (ADR-0004). Each axis varies across products and across configurations of the same product. The library must bound the contract between the core and any extension.

The core holds the semantic model and the [UBL] and [CII] (de)serializers (ADR-0005). Several extension concerns surround it, each revised upstream on its own schedule:
* per-format validator-output parsers,
* further extensions (mentioned in ADR-0001).

## Problem

How is the system decomposed into crates and what versioning policy should be followed to provide extendability?

## Decision

A virtual manifest at the root holds the workspace. Every crate lives under `crates/`, grouped into a per-axis subfolder. Each folder name describes the axis, while the crate name carries the `en16931-` prefix:

```text
en16931/
└── crates/
    ├── core/                  — en16931-core: model, bindings, profiles, two extension traits
    ├── envelopes/{svrl, ...}  — en16931-{svrl, ...}: validator-output wrappers
    └── xpath/{iso, ...}       — en16931-{iso, ...}: dialect xpath normalizers
```

> The semantic model, the two bindings, and the profiles are not extension axes. 

They live in the core (ADR-0005), unified across standards. A `Profile` only stamps `BT-24` and forbids terms of the core superset model (ADR-0003). So it supplies no vocabulary the core lacks. A profile that needs a new term grows the core model first. Its volatile rules are [Schematron], owned by the validator or its proxy (ADR-0002). So a profile belongs to the core, like a binding.

The core defines the `Binding` and `Profile` enums, and ships their variants. `Binding` carries [UBL] and [CII] (ADR-0005). `Profile` enumerates the per-standard profiles. Both are closed, core-only sets, so neither is a trait. The core also defines two extension traits:
* `Wrapper` unwraps the validator-specific envelope around the artifact,
* `Normalizer` rewrites a processor dialect into the record form (ADR-0004).

Extensions are plain Cargo dependencies, not feature flags. The consumer composes them at the call site, pairing a `Wrapper` with a `Normalizer`. Together they turn a validator artifact into the record form (ADR-0004).

Core follows semver. Each extension crate carries its own version. It declares a compatibility range of core in `Cargo.toml`. A breaking change in a trait, or a newly supported standard, bumps the core. Extensions update their range and re-release.

## Alternatives Considered

* **Conversion-driven extension contract.** Each extension exports a newtype and a `TryFrom` into a single report type. Rejected because envelope and [XPath] dialect conflate inside one type. Switching dialect within one service requires a new extension crate instead of recombining two existing ones.

* **Lock-step semver across the workspace.** One version on every crate, bumped together. Rejected because every new extension would force a major core release.

* **Independent semver without compat range.** Each crate versions on its own track, with no declared range of core. Rejected because consumers cannot tell which extension version pairs with which core.

* **Flat crate folders named after the crates.** Every crate sits directly under `crates/`, each folder named exactly as its crate. Rationale: the common convention eases navigation and renames. Rejection: a shared `en16931-` prefix forces long flat names. The per-axis grouping then disappears.

## Consequences

### Pros

* Both dialect and envelope recombine without new extension crates.
* Each axis releases on its own track.
* Consumers see exact compatibility windows.
* The per-axis grouping coexists with a shared `en16931-` crate prefix.

### Cons

* A consumer combines two extension crates instead of one.

## References

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[Schematron]: https://schematron.com/
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[XPath]: https://www.w3.org/TR/xpath-31/
