# ADR-0002: Validator Interface

## Context

The library is an adapter to an external validator (ADR-0001).

Validators differ in how they get their rules:
* [KoSIT] and [phive] bundle the compiled [Schematron] and apply it to the received invoice;
* a bare [Saxon] engine owns no rules and needs them in every request.

These rules change regularly, about twice a year plus minor versions.

## Problem

What does the library hand a validator? Who owns the rules and their versions?

## Decision

The library produces request parts, not the wire request. It exposes:
* the serialized invoice;
* the target of the check.

The application builds the request and routes it, owning the transport (ADR-0001).

> The library carries no stylesheets. A supported validator owns its rules, or runs behind a proxy that holds them.

The target names the profile and the document kind. An invoice and a credit note need different rule sets, so the kind belongs there. The profile already fixes the binding, which therefore takes no part of its own.

The target is always selected outside the invoice body. Each validator takes it in its own form:
* [phive] takes a vendor id, which its own extension derives from the profile;
* [KoSIT] takes the bare invoice, routed to the image built for that profile;
* [Saxon] engine needs a proxy holding the per-profile stylesheets, and isn't supported directly.

The validator deployment pins the rule version, not the library.

New profiles are frequent, and new validators are rare. So each service derives its own identifiers, and the library holds no list of them.

## Alternatives Considered

* **A single profile-and-binding identifier.** The library folds the profile and the binding into one opaque id. Rejected because the id form is validator-specific. The neutral parts let each extension derive its own id.

* **A uniform wire shape built by the library.** The library assembles the whole request, so the application writes no transport. Rejected because validator-specific transport then leaks into the library. The application already owns the transport (ADR-0001).

* **A closed list of profiles inside the service.** The service crate maps every supported profile to its identifier. Rejected because each new profile then needs a release of that crate. A third-party profile stays unsupported.

* **Dispatch on the `BT-24` string.** The service matches the identifier of the profile at run time. Rejected because an unsupported profile then fails during the exchange. A misspelled identifier passes unnoticed.

## Consequences

### Pros

* The library carries no stylesheets, and tracks no [XSLT] releases.
* The neutral parts let each validator derive its own request form.
* The application keeps the transport it already owns.
* Each service states the profiles it supports, and a new profile needs no library release.

### Cons

* A bare engine needs an external proxy to hold its rules.
* The application assembles and routes each request itself.
* A new service derives an identifier for every profile it supports.

## References

[KoSIT]: https://github.com/itplr-kosit/validator
[phive]: https://github.com/phax/phive
[Saxon]: https://www.saxonica.com/
[Schematron]: https://schematron.com/
[XSLT]: https://www.w3.org/TR/xslt-30/
