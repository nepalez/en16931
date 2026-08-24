# ADR-0008: Library Abstractions

## Context

The library converts the model to and from XML for both bindings. It accepts a parsed validator output (ADR-0006) and binds report locations to typed terms.

## Problem

What abstractions does the library offer to a consumer?

## Decision

> The library separates the business entity, the form the consumer fills, and the public artifact. Several types model the lifecycle.

`Invoice` is the business entity. It carries every business fact — parties, lines, dates, amounts. The amounts are inputs only, since the binding computes the derived totals. It is the superset model of all profiles (ADR-0003), where non-core fields are optional. Regulatory-flow fields (like `BT-23`) do not live here.

`DocumentBuilder` is the staging form the binding (de)serializes. It is a public struct with the `invoice`, the `profile`, the target `binding`, and the regulatory-flow data. The consumer fills it, and the binding reads it. No validation runs here.

`Binding` is the core (de)serializer (ADR-0005). It is a closed enum of the two bindings, not a trait. It serializes a `DocumentBuilder` to XML and parses XML back to one. It computes the derived amounts and fills the dictionary in lockstep.

`Document` is the public artifact. It holds a private `builder`, the `xml`, and the `dictionary` from record-form paths to `Context`-s. `TryFrom<DocumentBuilder>` serializes into it, and `Document::parse` detects the binding and reconstructs it. It converts into the `Invoice` through `From<Document>`. A received document must become a business object. An `Invoice` never parses from XML alone.

`Target` is what a validator needs to pick a rule set. It carries the profile, the binding, and the document kind. A `Document` yields one, and an extension turns it into the identifier of that service (ADR-0002).

`RawReport` is the normalized validator output. The core defines its shape and parsing pipeline, through `Wrapper` and `Normalizer` (ADR-0006). Every entry carries a severity, the rule identifier, the message text, and the address of the reported node. The severity tells an error from a warning. The entry keeps the address twice, as the processor wrote it and in the normalized form. Neither form is resolved yet, which `Document::check` supplies (ADR-0004). A user can build one too.

`Report` is the typed list of problems bound to `Context`-s, without references to XML. `Document::check` extends it from a `RawReport`.

`Document::check` binds each location of the `RawReport` to a `Context` through the dictionary. It returns `Result<Result<ValidDocument, InvalidDocument>, Error>`. A separate `Error` covers resolution failures (ADR-0007).

`ValidDocument` and `InvalidDocument` are newtypes over a `Document` and a bound `Report`. A `ValidDocument` carries a `Report` without errors, which may still hold warnings. Both convert back to a `Document` or an `Invoice`, dropping the report.

`Profile` is a core enum (ADR-0006). It stamps `BT-24` and drops the terms it forbids. Those terms come from a declarative set on each variant. It parses from a `BT-24` string. Like `Binding`, its closed core-only set needs no trait.

## Alternatives Considered

* **A separate internal structured form.** A distinct type sits between the builder and the artifact for (de)serialization. Rejected because the builder already carries every field, including the profile and binding. The parse path reconstructs the same builder, so a second type only duplicates it.

* **Single enum with state variants over an inner `Invoice`.** The shape compacts the surface but loses the type-level state constraint.

* **Self-referential storage via `Pin`.** Forces `Pin<Box<...>>` through every API that handles a checked artifact.

* **Profile stored inside `Invoice`.** Pollutes the business layer with a field that only the reporting form needs.

## Consequences

### Pros

* The `DocumentBuilder` doubles as the structured form, so no second type duplicates its fields.
* Function signatures can demand `ValidDocument` or `InvalidDocument` specifically.
* The dictionary lets the report reference model fields by static paths.
* `Invoice` stays free of reporting concerns.

### Cons

* `Document` carries the builder, the xml, and the dictionary together, which is heavier through FFI.
* The surface carries more types, with delegation boilerplate for the newtypes.
