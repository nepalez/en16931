# en16931

Electronic invoicing for Rust, built on the European standard [EN-16931].

<img src="https://cdn.evilmartians.com/badges/logo-no-label.svg" alt="Evil Martians logo" width="22" height="16" /> <b>en16931</b> is built by <b><a href="https://evilmartians.com/">Evil Martians</a></b>, an American design and engineering consultancy for <b>developer tools, AI, and cybersecurity startups</b>.

The library provides a typed model of an invoice that can be serialized into standards-compliant XML. An incoming XML document parses back into the invoice model. When an **external validator** checks the XML document, its findings are attached to invoice fields.

## Background

Electronic invoicing became mandatory in EU public procurement with [Directive 2014/55/EU]. The common language of this exchange is [EN-16931], which defines what an invoice states: who trades with whom, what is delivered, at what price, and under which taxes. Every statement has a stable code in the standard, such as `BT-1` for the invoice number.

### Vocabularies

An invoice travels as an XML document. Two XML vocabularies (the library calls them **bindings**) are allowed:

* **[UBL]** — the Universal Business Language, maintained by the standards consortium [OASIS]
* **[CII]** — the Cross Industry Invoice, maintained by [UN/CEFACT], the United Nations body for trade facilitation and electronic business

### Profiles

On top of the base European standard, countries (both in the EU and outside) and trade networks define their own **profiles** — the standard calls them Core Invoice Usage Specifications ([CIUS]).

A profile tailors the standard to a country, a trade network, or a sector within them. For example:
* [NLCIUS] narrows it down for the Netherlands,
* [XRechnung] adapts it to one sector of one country — German public procurement,
* [Peppol BIS Billing] serves the international [Peppol] delivery network,

and further profiles cover Australia and New Zealand, Singapore, Japan, Malaysia, and the United Arab Emirates.

Each specification is published in [Schematron], a rule language for XML used for automatic validation.

### Validators

Running these rules is, in practice, a Java affair. The most mature engine for them is [Saxon], and what sets it apart is support rather than code. No other ecosystem has sustained the continuous upkeep that evolving specifications and rule sets demand.

A **bare processor** holds no rules of its own and simply executes whatever rule set it receives. There are also applications that bundle a processor with the official rule sets, properly versioned, and expose the check as a ready-to-run service.

Two well-known examples are the [KoSIT validator], which ships the [XRechnung] configuration, and services built on the [phive] engine, which carry the [Peppol] and [EN-16931] rule sets.

## Getting started

### Install the crates

Add the core plus the extensions that match your validator:

```toml
[dependencies]
en16931-core = "0.1"
en16931-kosit = "0.1"
en16931-iso = "0.1"
```

The crates require Rust 1.85 or newer. The core carries an optional cargo feature, `mime`, which adds conversions between the attachment media type and the `Mime` type of the [mime] crate.

### Deploy a validator

Run a validator next to your application. The repository ships a ready `docker-compose.yml` with two of them: the [KoSIT] tool and a [phive]-based service.

## Examples

### Issuing an invoice

From business facts to a validated document:

```rust
use en16931_core::*;
use en16931_iso::Iso;
use en16931_kosit::Kosit;
use CountryCode::*;
use Currency::*;
use time::Month::*;

// 1. Describe the business facts
let invoice = Invoice {
    number: "INV-2026-001".parse()?,
    issue_date: Date::from_calendar_date(2026, January, 15)?,
    type_code: "380".parse()?,
    currency: EUR,
    seller: Seller {
        name: "Seller Official Name".parse()?,
        vat: Some("DE123456789".parse()?),
        address: PostalAddress {
            country: DE,
            // ...
        },
        // ...
    },
    // ...
};

// 2. Stage the invoice for one profile and one binding.
//    The XML is rendered under the hood as part of the document.
let document = DocumentBuilder {
    invoice,
    profile: Profile::XRechnung30,
    binding: Binding::Ubl,
    business_process: None,
}.try_into()?;

// 3. Send `document.xml()` to your validator. 
//    The transport is yours: any HTTP client will do.
let answer = post(validator_url, document.xml())?;

// 4. Read the answer, telling the parser:
//    * which service provided the response (`Kosit`),
//    * which spelling of the XML paths it uses (`Iso`).
let report = RawReport::parse(&answer, &Kosit, &Iso)?;

match document.check(report)? {
    Ok(valid) => {
        // The invoice passed, possibly with warnings in valid.report().
    }
    Err(invalid) => {
        for problem in &invalid.report().problems {
            // The context is a path to the field from the root of the model,
            // such as `seller.vat`.
            eprintln!("{} at {}: {}", problem.severity, problem.context, problem.text);
        }
    }
}
```

### Receiving an invoice

Receiving works in the opposite direction:

```rust
// 1. Parse the received XML: the binding is recognized automatically.
let document = Document::parse(&xml)?;

// 2. Check the document by an external validator, as in the first example.
let answer = post(validator_url, document.xml())?;
let report = RawReport::parse(&answer, &Kosit, &Iso)?;

// 3. On success, take the business object out of the document.
if let Ok(valid_report) = document.check(report)? {
    let invoice = Invoice::from(valid_report);
    // ...
}
```

## Principles

### Validation stays external

No validation logic is reimplemented here: the library prepares what a validator needs and makes sense of what it returns. When the rules change, you redeploy the validator and leave your code alone.

This choice also keeps the library pure Rust: no `unsafe`, no FFI to a bundled engine.

### One model covers every profile

The crate defines a **single model** for all invoices and credit notes. Fill it with the facts you have, then serialize it for a chosen profile: the serializer stamps the right identifier and omits whatever that profile forbids.

A cross-border seller can thus issue the same invoice under a domestic profile and under the buyer's one without duplicating data. More than thirty profiles are supported, and the list can be extended.

### Findings land on your fields

A raw report addresses problems by XML paths. The library translates each path into a typed pointer into the invoice, so an interface can highlight the item name of line 2 instead of printing the path that led there.

## Crates

Validators disagree on two things: how the response is packaged, and how the location of a finding — the path to the offending XML element — is spelled. The core stays neutral, while small extension crates absorb the differences: a **wrapper** decodes the response of one service, a **normalizer** decodes one location spelling, and any wrapper combines with any normalizer.

The core follows semantic versioning, and each extension evolves on its own line, declaring the range of core versions it is compatible with.

### Core

* `en16931-core` — the invoice model, both bindings, the profiles, and report handling

### Wrappers

One crate per response format:

* `en16931-kosit` — the [KoSIT validator], the [XRechnung] reference tool from the German coordination office for IT standards ([KoSIT])
* `en16931-phive` — services built on [phive], an open-source validation engine
* `en16931-svrl` — a bare report in the Schematron Validation Report Language ([SVRL]), the standard output of [Schematron] tools

The bare-[SVRL] wrapper stands apart. It reads the raw output of a Schematron processor with no service around it, so it works with any rule set — including one the [KoSIT] and [phive] services do not bundle, or your own. One caveat: a rule set belongs to a profile, and the profile shapes the serialized invoice. A new rule set therefore requires a new entry in `Profile` in `en16931-core`.

### Normalizers

One crate per location spelling, which varies with the Schematron processor behind the service:

* `en16931-iso` — the classic [ISO Schematron skeleton], which writes prefixed paths such as `/ubl:Invoice/cac:InvoiceLine[2]/cbc:ID`
* `en16931-schxslt` — the [SchXslt] processor, which spells every namespace out, as in `/Q{urn:...}Invoice[1]`
* `en16931-schxslt2` — the [SchXslt 2] processor, whose spelling differs from its predecessor in small quirks

## Contributing and support

Bug reports, questions, and pull requests are welcome on GitHub.

## License

MIT

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[CIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108937/CIUS+and+Extension+-+What+is+allowed
[Directive 2014/55/EU]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108867/European+legislation+on+eInvoicing
[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[ISO Schematron skeleton]: https://github.com/Schematron/schematron
[KoSIT]: https://www.xoev.de/
[KoSIT validator]: https://github.com/itplr-kosit/validator
[mime]: https://crates.io/crates/mime
[NLCIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108895/eInvoicing+in+The+Netherlands
[OASIS]: https://www.oasis-open.org/
[Peppol]: https://peppol.org/
[Peppol BIS Billing]: https://docs.peppol.eu/poacc/billing/3.0/
[phive]: https://github.com/phax/phive
[Saxon]: https://www.saxonica.com/
[SchXslt]: https://codeberg.org/schxslt/schxslt
[SchXslt 2]: https://codeberg.org/schxslt/schxslt2
[Schematron]: https://schematron.com/
[SVRL]: https://schematron.com/document/3427.html
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[UN/CEFACT]: https://en.wikipedia.org/wiki/UN/CEFACT
[XRechnung]: https://xeinkauf.de/xrechnung/

