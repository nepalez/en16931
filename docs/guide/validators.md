# Validators

This chapter explains how to build a request for each supported validation service and which pair of extensions reads its answer.

## One Answer, Two Extensions

The answer of a validator is read by a pair of extensions, each shipped as a crate of its own:

* a `Wrapper` describes how the validator packages its findings,
* a `Normalizer` describes the **dialect** of spelling [XPath] addresses in error references.

Both ready services, the [KoSIT validator] and [phive], answer in the `Iso` dialect. `Schxslt` and `Schxslt2` can be used in [SVRL] assemblies of your own.

## The `KoSIT` Validator

The [KoSIT validator] applies the rule configuration baked into its deployment. The [XRechnung] configuration, for example, holds the base [EN-16931] rules plus the German [CIUS], for both [UBL] and [CII]. Another profile requires another deployment.

The request is the bare invoice XML. The service picks the rules by inspecting the document itself.

## The `phive` Services

A [phive] service, such as [phorm], carries many rule sets, and the caller names one in the request. `Wrapper::vendor_id` derives that name from the `Target` of the document:

```rust
let rules = Phive.vendor_id(document.target())?; // "eu.peppol.bis3:invoice"
let url = format!("{base}/api/validate/{rules}:latest");
```

The invoice XML goes to that URL with the service token in the `X-Token` header. For a profile the service does not bundle, `vendor_id` fails.

## Bare SVRL

The ready services do not cover every standard. For the rest, assemble a service of your own from a [Schematron] processor and the rule set of your profile.

Such an assembly differs from the ready services in two ways:

* The answer is a raw [SVRL] report without any envelope. The `Svrl` wrapper reads it as is.
* The processor bundles no rule sets. You should compile the rule set into the stylesheet it executes.

Three compilers exist for that compilation, and each spells addresses in a dialect of its own. Pick the normalizer by your compiler:

* `Iso` for the [ISO Schematron skeleton],
* `Schxslt` for [SchXslt],
* `Schxslt2` for [SchXslt 2].

## Choosing the Pair

Add the two crates of your service:

* the [KoSIT validator] — `en16931-kosit` with `en16931-iso`,
* a [phive] service — `en16931-phive` with `en16931-iso`,
* your own assembly — `en16931-svrl` with the normalizer depending on the compiler you use: `en16931-iso`, `en16931-schxslt`, or `en16931-schxslt2`.

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[CIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108937/CIUS+and+Extension+-+What+is+allowed
[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[ISO Schematron skeleton]: https://github.com/Schematron/schematron
[KoSIT validator]: https://github.com/itplr-kosit/validator
[phive]: https://github.com/phax/phive
[phorm]: https://github.com/phax/phorm
[SchXslt]: https://codeberg.org/schxslt/schxslt
[SchXslt 2]: https://codeberg.org/schxslt/schxslt2
[Schematron]: https://schematron.com/
[SVRL]: https://schematron.com/document/3427.html
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[XPath]: https://www.w3.org/TR/xpath-31/
[XRechnung]: https://xeinkauf.de/xrechnung/
