# ADR-0014: Invoice Invariants

## Context

The semantic model holds the monetary amounts of an invoice. [EN-16931] ties them by the `BR-CO` calculation rules. A small set of inputs determines the rest. Quantities, prices, rates, and allowance bases are free. Line nets, sums, totals, and the VAT breakdown follow by arithmetic.

The XML form must carry every amount, including the derived ones. The report binding (ADR-0009, ADR-0010) maps each [SVRL] location to a model node.

The domain folds already break a one-to-one match between fields and terms. A folded VAT treatment spans several terms. The model shape and the term layout diverge.

## Problem

Should the model enforce the invariants of an invoice, or leave them to the validator?

Does the model store the derived amounts, or compute them?

How does a report locate a derived term with no stored field?

## Decision

> The model holds the business inputs only. The serialization computes every derived amount.

`Invoice` carries the free inputs. The inputs are quantities, prices, rates, allowance and charge bases, paid and rounding amounts. It holds no line net, no document total, and no VAT breakdown.

The serialization applies the `BR-CO` rules when it writes the document. The rounding follows [EN-16931], half up per VAT category. Every term reaches the XML from a model node or from a computed value. A derived amount is written only when every input is present. Otherwise the element is omitted, and the validator reports it.

The dictionary binds a derived term to its nearest stored node. A line net resolves to its line. A document total resolves to the `Document` root. The serialization records these targets on its pass (ADR-0009).

## Alternatives Considered

* **Materialized amounts.** The model stores every term, and a build step fills the derived ones. Rejected because the redundant fields admit inconsistent input and drift from the domain.

* **Caller-supplied amounts.** The model carries every amount as plain input without computation. Rejected because the caller must compute the whole graph and keep it consistent.

## Consequences

### Pros

* The model states the domain truth once, with no derivable field.
* A consumer supplies the inputs, and the library produces consistent amounts.

### Cons

* The core carries the calculation logic and its rounding rules.
* A parsed invoice loses the sender's stated amounts, since the serialization recomputes them.
* A derived-term finding resolves to a group or the root, not to a dedicated field.

## References

[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[SVRL]: https://schematron.com/document/3427.html
