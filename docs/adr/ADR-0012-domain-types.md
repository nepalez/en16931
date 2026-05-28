# ADR-0012: Domain Types for Model Fields

## Context

The library's public surface centers on `Invoice` (ADR-0008). Each [EN-16931] term carries a constraint. Examples are a length cap, a code list, or a currency type.

## Problem

What type does a model field take, and where is its constraint enforced?

## Decision

A model field never holds a base primitive (`String`, `&str`, `i32`, `f64`). It holds a type that names the term and carries its constraint.

A constraint with no existing crate becomes a newtype around the primitive. The newtype implements `TryFrom` from the primitive, or `FromStr` for strings. That conversion is the single place where the constraint is checked.

A supported profile version may loosen a constraint. The newtype then accepts the loosest value across versions. A stricter version rejects it at the validator (ADR-0001).

A constraint already modeled by a mature crate reuses that crate's type. An [ISO 4217] currency code is one example.

## Alternatives Considered

* **Primitives on fields.** The model carries raw `String` and `i32`. The external validator (ADR-0001) reports any bad value. Rejected because invalid state can sit in the model unchecked. The field type also gives no signal about the term's constraint.

## Consequences

### Pros

* The field type names the term and its constraint in the public API.
* A constructed value is valid. No struct holds an unchecked term.
* Each constraint is checked once at the conversion boundary.
* Mature crates carry domain logic the library does not re-implement.

### Cons

* The public surface grows one type per constrained term.
* The consumer converts at the boundary, often through `try_into`.

## References

[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[ISO 4217]: https://www.iso.org/iso-4217-currency-codes.html
