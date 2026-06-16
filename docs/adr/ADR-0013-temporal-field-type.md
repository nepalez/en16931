# ADR-0013: Temporal Field Type

## Context

The temporal terms of [EN 16931] need a type.

## Problem

Does the library need a time-of-day type anywhere, or a date type is enough?

## Decision

The library takes `Date` from the [time] crate because no time-of_day or datetime type is necessary:

* [EN 16931] fixes a closed set of ten semantic data types, among which the `Date` is the only temporal one.
* A CIUS profile constrains terms and adds no data type.
* The library serializes only the invoice model. The validator output is parsed, never serialized.

So no path carries a time-of-day value.

## Consequences

### Pros

* The library depends on the minimalistic [time] crate only.
* A field value cannot carry a time-of-day outside the standard's scope.

### Cons

* A later term with a time-of-day would force a new temporal type.

## References

[EN 16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[time]: https://crates.io/crates/time
