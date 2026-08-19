# ADR-0004: Binding and Dialect

## Context

An external [Schematron] validator (ADR-0001) emits an [SVRL] report whose failed assertions carry [XPath] locations. The library matches each location to a model field. It matches only against the document whose XML was checked. So the report and that document share one binding.

The standard ships two XML bindings, [UBL] and [CII], for the same content. Validators differ in how their processor writes a location.

So a location varies along two independent axes.

## Problem

How should the library treat an [XPath] location, given two bindings and many processor dialects? Should it reduce every location to one cross-binding form?

## Decision

An [XPath] location has two independent variables:
* the **binding** is its element vocabulary, fixed by the document;
* the **dialect** is its surface syntax, fixed by the validator's processor.

The library records every path in one form. It is namespace-resolved, dialect-free, and carries the positional index of every step. The names stay as URIs, so the vocabulary remains the binding's.

> A [UBL] path and a [CII] path never unify. The record form is binding-specific, not a cross-binding canonical [XPath]. Only the dialect is normalized away.

A per-dialect step rewrites a processor's location into this form. It reads names from the location alone. It needs neither the schema nor the binding, so one such step serves every binding.

A dialect names the namespace of a step in one of two ways. It writes the full URI, which the step resolves on its own. Or it writes a short abbreviation, which the step resolves through a table.

Such an abbreviation has two origins. One processor copies the abbreviation of the checked document. Another writes the abbreviation of its own rule set. So the binding builds one table per document, holding both origins.

An abbreviation that stands for two namespaces at once is a hard failure of that document.

The binding enters only when a path is written. The dialect enters only when a report is read.

## Alternatives Considered

* **Binding-independent canonical form.** Both [UBL] and [CII] locations reduce to one shared [XPath] for a single match path. Rejected because the two paths never unify. The shared form hides a wrong-binding location instead of failing it.

* **Dialect folded into the binding I/O.** The binding-aware writer also absorbs the processor dialect, since the document fixes the binding. Rejected because the dialect comes from the validator, not the binding. Folding it multiplies binding handlers by every dialect.

* **Declarations of the report as the only source.** A report declares the namespaces it uses, so a step could read them. Rejected because a processor may copy the abbreviation of the document, which the report never declares.

* **A fixed table inside every normalizer.** The rule sets fix `cac`, `cbc`, `ram`, and `rsm`, so no input is needed. Rejected because a document may declare an abbreviation of its own, which such a table misses.

## Consequences

### Pros

* One normalizing step per dialect serves every binding, because it never reads the schema.
* The binding and the dialect recombine freely, with neither leaking into the other.
* A wrong-binding location fails to match, instead of resolving to the wrong field.
* Both origins of an abbreviation resolve through a single table.

### Cons

* Every supported processor dialect needs its own normalizing step.
* The abbreviation table travels from the document to the step that reads a report.
* A document that binds a standard abbreviation to another namespace is rejected.

## Examples

The normalizer applies one mechanical rewrite. It reads names from the location, and namespaces from the location or the abbreviation table:

* `*[local-name()='N' and namespace-uri()='U'][i]` → `Q{U}N[i]`;
* `p:N[i]` → `Q{U}N[i]`, where `p` resolves to `U` through the abbreviation table;
* a step with no index → index `[1]`.

Namespace URIs are abbreviated: UBL by `INV`, `CAC`, `CBC`. CII by `RSM`, `RAM`.

Take `BT-126`, the identifier of the second invoice line in a UBL document. Two validators write its location in different dialects:

```text
dialect A  /*[local-name()='Invoice' and namespace-uri()='INV'][1]
           /*[local-name()='InvoiceLine' and namespace-uri()='CAC'][2]
           /*[local-name()='ID' and namespace-uri()='CBC'][1]

dialect B  /ubl:Invoice/cac:InvoiceLine[2]/cbc:ID
```

The rule reduces both to one record form:

```text
/Q{INV}Invoice[1]/Q{CAC}InvoiceLine[2]/Q{CBC}ID[1]
```

In a CII document the same rule yields the CII record form:

```text
/Q{RSM}CrossIndustryInvoice[1]/Q{RSM}SupplyChainTradeTransaction[1]
/Q{RAM}IncludedSupplyChainTradeLineItem[2]
/Q{RAM}AssociatedDocumentLineDocument[1]/Q{RAM}LineID[1]
```

The rewrite never inspects a name to tell [UBL] from [CII]. It copies `N`, `U`, and `i` from the input. So one normalizing step serves both formats, and the two record forms stay distinct.

## References

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[Schematron]: https://schematron.com/
[SVRL]: https://schematron.com/document/3427.html
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[XPath]: https://www.w3.org/TR/xpath-31/
