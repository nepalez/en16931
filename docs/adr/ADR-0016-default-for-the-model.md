# ADR-0016: Default for the Model

## Context

Every field of the model is optional, except the type code `BT-3` (ADR-0003). A consumer fills `Invoice` and its groups through struct literals. A literal spells every optional field, even when the value is `None`.

Domain types already check values at the construction boundary (ADR-0012). Cross-field rules stay with the external validator (ADR-0001).

## Problem

How can the library shorten model construction?

## Decision

The model structs implement `Default`. `InvoiceType` defaults to code 380, the commercial invoice, so `Invoice` derives `Default` too. A consumer spells the fields it fills and closes the literal with `..Default::default()`.

The default adds no validation of its own. Relational and conditional constraints remain a duty of the validator.

## Alternatives Considered

* **Generated typestate builder.** The [bon] crate derives a builder that checks required fields at compile time. Rejected because one required field guards nothing, and the derive costs a macro per struct.

* **A body struct of the optional fields.** `Invoice` holds the type code and a `Body` with `Default`. Rejected because every walk and every `Context` gains one more level for no benefit.

## Consequences

### Pros

* An omitted field costs no code and no attribute at the call site.
* The model carries no procedural macro output.

### Cons

* A forgotten type code yields a valid invoice of the wrong kind, which no validator detects.

## Examples

```rust
let contact = Contact {
    name: Some("Anna Seller".parse()?),
    email: Some("anna@example.de".parse()?),
    ..Default::default()
};
```

## References

[bon]: https://docs.rs/bon
