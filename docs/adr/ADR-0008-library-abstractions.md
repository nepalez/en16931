# ADR-0008: Library Abstractions

## Context

The library converts the model to and from XML for both bindings. It accepts a parsed validator output (ADR-0006) and binds report locations to typed terms.

## Problem

What abstractions does the library offer to a consumer?

## Decision

> The library separates the business entity, the form the consumer fills, and the public artifact. Several types model the lifecycle.

`Invoice` is the business entity. It carries every business fact — parties, lines, dates, amounts. The amounts are inputs only, since serialization computes the derived totals. It is the superset model of all profiles (ADR-0003). Every field but the type code is optional, so it implements `Default`. Regulatory-flow fields (like `BT-23`) do not live here.

`DocumentBuilder<P>` is the staging form that serialization reads. It is a public struct with the `invoice` and the regulatory-flow data. The parameter `P` names the profile and its binding, so neither takes a field. No validation runs here.

`Serializable` and `Deserializable` are the core (de)serializers (ADR-0005). They are traits of an invoice type, parameterized by the binding and the namespace set. Each one fills the document in place, computing the derived amounts and the dictionary in lockstep.

`Document<P>` is the public artifact. It holds a private `builder`, the `xml`, and the `dictionary` from record-form paths to `Context`-s. `TryFrom<DocumentBuilder<P>>` serializes into it, and `Document::parse_as::<P>` reconstructs it. That parse rejects a document whose `BT-24` belongs to another profile. It converts into the `Invoice` through `From<Document<P>>`. A received document must become a business object. An `Invoice` never parses from XML alone.

`Target<P>` is what a validator needs to pick a rule set. It carries the document kind, while its parameter names the profile and the binding. A `Document<P>` yields one, and an extension turns it into the identifier of that service (ADR-0002).

`RawReport` is the normalized validator output. The core defines its shape and parsing pipeline, through `Wrapper` and `Normalizer` (ADR-0006). Every entry carries a severity, the rule identifier, the message text, and the address of the reported node. The severity tells an error from a warning. The entry keeps the address twice, as the processor wrote it and in the normalized form. Neither form is resolved yet, which `Document::check` supplies (ADR-0004). A user can build one too.

`Report` is the typed list of problems bound to `Context`-s, without references to XML. `Document::check` extends it from a `RawReport`.

`Document::check` binds each location of the `RawReport` to a `Context` through the dictionary. It returns `Result<Result<ValidDocument<P>, InvalidDocument<P>>, Error>`. A separate `Error` covers resolution failures (ADR-0007).

`ValidDocument<P>` and `InvalidDocument<P>` are newtypes over a `Document<P>` and a bound `Report`. A `ValidDocument<P>` carries a `Report` without errors, which may still hold warnings. Both convert back to a `Document<P>` or an `Invoice`, dropping the report.

`Profile` is a core trait (ADR-0006). A profile type stamps `BT-24` per document kind, and omits it for an uncovered kind. It also drops the terms it forbids, taken from a declarative set on the type. The type carries the default `BT-23` value, applied when the builder leaves that field empty. Its invoice type converts from and into the `Invoice`.

## Alternatives Considered

* **A separate internal structured form.** A distinct type sits between the builder and the artifact for (de)serialization. Rejected because the builder already carries every field, while its type names the profile and the binding. The parse path reconstructs the same builder, so a second type only duplicates it.

* **Single enum with state variants over an inner `Invoice`.** The shape compacts the surface but loses the type-level state constraint.

* **Self-referential storage via `Pin`.** Forces `Pin<Box<...>>` through every API that handles a checked artifact.

* **Profile stored inside `Invoice`.** Pollutes the business layer with a field that only the reporting form needs.

## Consequences

### Pros

* The `DocumentBuilder<P>` doubles as the structured form, so no second type duplicates its fields.
* Function signatures can demand `ValidDocument<P>` or `InvalidDocument<P>` specifically.
* The dictionary lets the report reference model fields by static paths.
* `Invoice` stays free of reporting concerns.

### Cons

* `Document<P>` carries the builder, the xml, and the dictionary together, which is heavier through FFI.
* The surface carries more types, with delegation boilerplate for the newtypes.
