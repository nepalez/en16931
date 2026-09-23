# ADR-0014: Invoice Invariants

## Context

The semantic model holds the monetary amounts of an invoice. [EN-16931] ties them by the `BR-CO` calculation rules. Quantities, prices, and rates are free inputs. Line nets, totals, and the VAT breakdown follow from them by arithmetic.

The arithmetic involves rounding, and the document carries no rounding strategy. A receiver cannot reproduce the amounts of a sender by recomputation.

The rule sets check the sums for exact equality over two-decimal amounts. They check the products with a tolerance of one currency unit.

The report binding (ADR-0009, ADR-0010) maps each [SVRL] location to a model node.

## Problem

Should the model enforce the invariants of an invoice, or leave them to the validator?

Does the model store the derived amounts, or compute them?

## Decision

> The model stores every amount as the issuer states it. The library computes none.

The `Invoice` trait exposes the inputs and the derived amounts alike. The types of `cius` store both. The trait covers line nets, document totals, the VAT breakdown, and the net price. A relative allowance or charge holds its amount next to the base and the rate.

The issuer computes the amounts and picks the rounding. The validator checks the `BR-CO` rules. The library reports no inconsistency of its own.

The serialization writes an amount only when its field is filled. It rounds each amount to two decimals, half away from zero. A price keeps the scale the issuer gave it. No other place of the library rounds.

The parsing keeps every amount of the document. Every amount has a field of its own, so a finding binds to that field.

## Alternatives Considered

* **Computed amounts.** The model holds the inputs, and the serialization computes the rest. Rejected because the document carries no rounding strategy. A parsed invoice also loses the amounts the sender stated.

* **An amount type limited to two decimals.** The type rejects a value with more decimals. Rejected because the validator already reports such a value.

* **Materialized amounts.** The model stores every term, and a build step fills the derived ones. Rejected because the build step is the same calculation inside the library.

## Consequences

### Pros

* A parsed document reproduces the amounts of its sender.
* The core carries no calculation logic.
* A finding on an amount binds to a dedicated field.

### Cons

* The consumer computes the whole graph of amounts.
* The model admits inconsistent amounts.
* The independent rounding on serialization may break an exact equality of sums.

## References

[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[SVRL]: https://schematron.com/document/3427.html
