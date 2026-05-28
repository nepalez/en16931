# ADR-0011: Error Surface

## Context

Several library operations fail outside the finding channel (ADR-0007). The core orchestrates the pipeline. Extension crates own the specifics of their steps (ADR-0006, ADR-0008).

## Problem

What shape does the library's error take, and where does each failure mode live?

## Decision

The core exposes a single `Error` enum. Its variants describe failure points in the core's own logic. They do not name extensions as a category.

Each extension trait declares an associated `Error` type bound by `Into<core::Error>`. The extension crate picks the concrete type and writes the conversion. Orphan rules permit it, since the source type is local.

A converted error reaches the core through one of the core-logic variants. The original extension error stays available through `std::error::Error::source()`.

## Alternatives Considered

* **Type-erased extension variant.** `Error::Extension(Box<dyn std::error::Error>)` would absorb any extension failure. It was rejected because the variant names extensions as a core concept.

* **Trait-bounded extension variant.** The same shape with a core trait in place of `std::error::Error`. It was rejected for the same reason. The trait would also need updates for new metadata.

* **Dedicated stage variants.** The enum would carry stage-specific extension failures. It was rejected because new extension failures would force core releases.

* **Type per stage.** Each operation returns its own error type. It was rejected because the surface multiplies. A cross-stage pipeline chains through manual conversions.

## Consequences

### Pros

* New failure modes inside an extension stay inside the extension.
* The core grows its `Error` only when its own logic gains a new failure point.

### Cons

* A consumer that wants the extension cause walks `source()` and downcasts.
* Each extension crate writes its own conversion.
