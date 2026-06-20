# ADR-0014: Invoice Invariants

## Context

The semantic model holds an invoice's monetary amounts. [EN-16931] ties them by `BR-CO` calculation rules. A small set of inputs determines the rest. Quantities, prices, rates, and allowance bases are free. Line nets, sums, totals, and the VAT breakdown follow by arithmetic.

The XML form must carry every amount, including the derived ones. The report binding (ADR-0009, ADR-0010) maps each [SVRL] location to a model node.

The domain folds already break a one-to-one match between fields and terms. A folded VAT treatment spans several terms. The model shape and the term layout diverge.

## Problem

Should the model enforce invariants of an invoice or leaves it to a customer/valiador?

Does the model store the derived amounts, or compute them?

How does a report locate a derived term with no stored field?

## Decision

> The model holds the business inputs only. The binding computes every derived amount.

`Invoice` carries the free inputs. The inputs are quantities, prices, rates, allowance and charge bases, paid and rounding amounts. It holds no line net, no document total, and no VAT breakdown.

The `Binding` computes the derived amounts on serialization. It applies the `BR-CO` rules and the round-half-up convention per VAT category. The `Binding` maps each term to a model node or a computed value.

The dictionary binds a derived term to its nearest stored node. A line net resolves to its line. A document total resolves to the `Document` root. The `Binding` records these targets on its pass (ADR-0009).

## Alternatives Considered

* **Materialized amounts.** The model stores every term, and a build step fills the derived ones. Rejected because the redundant fields admit inconsistent input and drift from the domain.

* **Caller-supplied amounts.** The model carries every amount as plain input without computation. Rejected because the caller must compute the whole graph and keep it consistent.

## Consequences

### Pros

* The model states the domain truth once, with no derivable field.
* A consumer supplies the inputs, and the library produces consistent amounts.

### Cons

* The `Binding` carries the calculation logic and its rounding rules.
* A parsed invoice loses the sender's stated amounts, since the binding recomputes them.
* A derived-term finding resolves to a group or the root, not to a dedicated field.

## References

[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[SVRL]: https://schematron.com/document/3427.html
