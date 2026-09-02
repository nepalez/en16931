# ADR-0016: Generated Builder for the Model

## Context

The model is the superset of every profile (ADR-0003). Most of its fields are optional. A consumer fills `Invoice` and its groups through struct literals. A literal spells every optional field, even when the value is `None`. Fixtures and examples repeat dozens of `None` values and empty vectors.

Domain types already check values at the construction boundary (ADR-0012). Cross-field rules stay with the external validator (ADR-0001).

## Problem

How can the library shorten model construction? Required fields must stay checked at compile time.

## Decision

The core derives a generated typestate builder on the model structs with optional fields. A struct whose every field is required gains no builder. Its literal spells nothing absent, so a builder would only add noise. The [bon] crate provides the derive. Public fields stay in place. The builder is an additional construction path. The change is backward compatible and ships as a minor core release.

An `Option<T>` field becomes an optional setter without a per-field attribute. A repeatable group, held as a `Vec` field, defaults to an empty list. A missing required field fails compilation through the typestate. `build()` returns the struct without a `Result`.

The builder adds no validation of its own. Relational and conditional constraints remain a duty of the validator. Parsing of string values into domain types remains a duty of the caller.

## Alternatives Considered

* **Hand-written builders.** They give full control over the API surface. Rejected because the repetitive code must be written and maintained by hand.

* **[derive_builder].** It is the most widely used generator. Rejected because its `build()` returns a `Result`. A missing required field surfaces only at runtime.

* **[typed-builder].** It checks required fields at compile time. Rejected because an `Option<T>` field turns optional only through a per-field attribute.

* **Constructors of the required minimum.** A `new(...)` per struct covers the mandatory fields. Rejected because positional arguments scale poorly. Optional fields would still need mutation afterwards.

## Consequences

### Pros

* An omitted optional field costs no code and no attribute at the call site.
* A missing required field fails compilation, not a test.
* The builder code is generated, with no handwritten boilerplate to maintain.

### Cons

* The core gains a procedural-macro dependency and its compile-time cost.
* Each derived struct carries the macro output and generated builder types.

## Examples

```rust
let contact = Contact::builder()
    .name("Anna Seller".parse()?)
    .email("anna@example.de".parse()?)
    .build();
```

The `telephone` field stays `None` without being spelled.

## References

[bon]: https://docs.rs/bon
[derive_builder]: https://docs.rs/derive_builder
[typed-builder]: https://docs.rs/typed-builder
