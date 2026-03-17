# ADR-0001: Project Scope

## Context

The library exposes [EN-16931] electronic invoices as a Rust semantic model. Applications build invoices, exchange them as XML, and check their conformance to the standard. The standard defines two XML bindings, [UBL] and [CII], for the same invoice content. Each binding ships an XSD schema for structural validation.

Country and sector profiles such as [XRechnung] or [Peppol BIS] extend the base standard. The standard calls them [CIUS], or Core Invoice Usage Specifications. Both the base and its profiles ship their rules as [Schematron] documents.

[Schematron] rules use [XPath] to navigate the invoice XML tree. A [Schematron] processor compiles these rules to XSLT. Then it runs the resulting XSLT over the XML document and emits an [SVRL] report.

The industry runs these rules through the [Saxon] XSLT engine. Mature validators either wrap [Saxon] directly ([KoSIT], [phive]) or expose a [Saxon]-backed service over HTTP. None reimplement [Schematron] themselves.

No mature Rust engine for [XPath 2.0] or [XSLT 3.0] exists. The [xee] crate is still early, and other Rust XPath crates stop at version 1.0. Mature engines either run on the JVM ([Saxon]) or ship as native binaries. The native build, [SaxonC], is produced from the same Java code via GraalVM. Calling it from Rust requires an FFI. No such wrapper exists in the registry.

Embedding the engine locally requires more than the binary. The library would also carry compiled XSLT artifacts per profile. These come from [Schematron] sources via the [SchXslt] transpiler. They need refreshing on every standard update.

## Problem

How should [EN-16931] invoices be validated?
What is the library's role in that process that avoids reinventing the wheel?

## Decision

An external service handles validation (both XSD and [Schematron] in one pass). A typical deployment runs it in a Docker container next to the application. The application chooses the service, orchestrates the validation flow, and provides its transport.

A supported service owns its rules, or runs behind a proxy that holds them. The library carries no stylesheets.

> The library's role is to adapt Rust code to the service interface (XML-serialized invoices and [SVRL] reports).

As such it:
* provides a handwritten semantic invoice model for all profiles, with no code generation;
* serializes the model to [UBL] and [CII] XML and parses it back;
* parses [SVRL] reports and maps [XPath] locations to model fields.

## Alternatives Considered

* **Native Rust interpreter over the semantic model.** The approach offers full type safety on field references. It was rejected because the new surface is large. The library would carry its own expression language, code generator, manifest, and interpreter. It would reimplement [Schematron] without community testing.

* **Local [Schematron] engine through [SaxonC] FFI.** The native binary keeps validation inside the library. It was rejected because every language binding inflates with the bundled engine. Engine maintenance also moves onto the project.

* **Hybrid: semantic model plus local [SaxonC] engine.** This option combines a Rust-friendly API with normative validation. It was rejected for the same FFI cost as the local engine. It also adds round-trip concerns between the model and its XML form.

## Consequences

### Pros

* The cost of additional infrastructure is negligible compared to the cost of reimplementing a mature engine.
* Engine code lives outside the project, with fixes from upstream.
* The deployed service is battle-tested, unlike any code we would write.
* The library stays free of FFI and its memory-safety risks.
* A new [CIUS] profile adds at most a small config crate, never engine code or a stylesheet.
* The path to bringing validation into the library stays open.

### Cons

* Two artifacts must be deployed together: the library and the service.
* Transport implementation falls on the application, not the library.
* End-to-end testing falls on the application, not the library.
* A bare engine needs an external proxy to hold the rules.

## References

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[CIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108937/CIUS+and+Extension+-+What+is+allowed
[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[KoSIT]: https://github.com/itplr-kosit/validator
[Peppol BIS]: https://docs.peppol.eu/poacc/billing/3.0/
[phive]: https://github.com/phax/phive
[Saxon]: https://www.saxonica.com/
[SaxonC]: https://www.saxonica.com/saxon-c/index.xml
[Schematron]: https://schematron.com/
[SchXslt]: https://codeberg.org/schxslt/schxslt2
[SVRL]: https://schematron.com/document/3427.html
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[xee]: https://github.com/Paligo/xee
[XPath]: https://www.w3.org/TR/xpath-31/
[XPath 2.0]: https://www.w3.org/TR/xpath20/
[XRechnung]: https://xeinkauf.de/xrechnung/
[XSLT 3.0]: https://www.w3.org/TR/xslt-30/
