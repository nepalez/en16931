# ADR-0007: Findings versus Errors

## Context

A validator reports findings against an invoice. The library binds each report location to a model node (ADR-0004).

The `Binding` fills the dictionary as it (de)serializes, one key per node. So the dictionary covers every node of the document, on either path.

## Problem

Which outcomes are reportable validation findings, and which are errors of the library itself?

## Decision

> One unbound location MUST fail the whole pass, dropping any findings.

A validation finding is a report entry bound to a model node.

The complete dictionary MUST cover every node, so a location the validator emits on our document MUST bind. An unbound location is therefore not about our document.

The very presence of a mismatch is unrecoverable. It discredits the whole report, so no finding from that pass survives. The library fails the operation as a whole, carrying the original location. It is never dropped silently, and never turned into a finding.

## Alternatives Considered

* **Unbound location as a finding.** Keep the raw location among the findings. Rejected because a dictionary miss is a library fault, not invoice content.

* **Skip the unbound location.** Drop it, keep the rest. Rejected because the mismatch discredits the whole report, not one entry.

## Consequences

### Pros

* Findings and library errors stay in separate channels.
* A binding or dialect mismatch surfaces loudly, not as a silent gap.
